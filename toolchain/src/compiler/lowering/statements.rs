use std::collections::HashMap;

use crate::ast::syntax::Statement;
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{TypeId, TypeSystem};

use super::assignments::lower_assignment;
use super::expressions::lower_expression;
use super::loops::lower_for_loop;
use super::types::{LocalBinding, LoopContext};
use super::utils::{allocate_local, ensure_local_type, patch_jump, patch_jumps};

pub fn lower_block(
    statements: &[Statement],
    chunk: &mut Chunk,
    locals: &mut HashMap<String, LocalBinding>,
    return_type: TypeId,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    function_name: &str,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<(), CompileError> {
    for statement in statements {
        match statement {
            Statement::Let(statement) => {
                let declared = statement
                    .type_ann
                    .as_ref()
                    .map(|type_name| {
                        types.ensure_named(&type_name.name).ok_or_else(|| {
                            CompileError(format!(
                                "unknown type `{}` on local `{}`",
                                type_name.name, statement.name.name
                            ))
                        })
                    })
                    .transpose()?;

                let inferred =
                    lower_expression(&statement.value, chunk, locals, types, signatures, declared)?;
                let local_type = declared.unwrap_or(inferred);

                if !types.is_assignable(local_type, inferred) {
                    return Err(CompileError(format!(
                        "cannot assign value of type {:?} to local `{}`",
                        types.get(inferred),
                        statement.name.name
                    )));
                }

                let slot = allocate_local(chunk);
                locals.insert(
                    statement.name.name.clone(),
                    LocalBinding {
                        slot,
                        type_id: local_type,
                    },
                );
                ensure_local_type(chunk, slot, local_type);
                chunk.instructions.push(Instruction::StoreLocal(slot));
            }
            Statement::Assign(statement) => {
                lower_assignment(statement, chunk, locals, types, signatures)?;
            }
            Statement::Return(statement) => {
                let value_type = lower_expression(
                    &statement.expression,
                    chunk,
                    locals,
                    types,
                    signatures,
                    Some(return_type),
                )?;
                if !types.is_assignable(return_type, value_type) {
                    return Err(CompileError(format!(
                        "return type mismatch in `{}`: expected {:?}, got {:?}",
                        function_name,
                        types.get(return_type),
                        types.get(value_type)
                    )));
                }

                chunk.instructions.push(Instruction::Return);
            }
            Statement::Expression(statement) => {
                let expression_type = lower_expression(
                    &statement.expression,
                    chunk,
                    locals,
                    types,
                    signatures,
                    None,
                )?;
                if expression_type != types.unit() {
                    chunk.instructions.push(Instruction::Pop);
                }
            }
            Statement::If(statement) => {
                let condition_type = lower_expression(
                    &statement.condition,
                    chunk,
                    locals,
                    types,
                    signatures,
                    Some(types.bool()),
                )?;
                if condition_type != types.bool() {
                    return Err(CompileError(
                        "`if` conditions must evaluate to `bool`".to_string(),
                    ));
                }

                let jump_if_false = chunk.instructions.len();
                chunk
                    .instructions
                    .push(Instruction::JumpIfFalse(usize::MAX));

                let mut then_locals = locals.clone();
                lower_block(
                    &statement.then_block.statements,
                    chunk,
                    &mut then_locals,
                    return_type,
                    types,
                    signatures,
                    function_name,
                    loop_stack,
                )?;

                if let Some(else_block) = &statement.else_block {
                    let jump_after_then = chunk.instructions.len();
                    chunk.instructions.push(Instruction::Jump(usize::MAX));
                    let else_start = chunk.instructions.len();
                    patch_jump(&mut chunk.instructions, jump_if_false, else_start);

                    let mut else_locals = locals.clone();
                    lower_block(
                        &else_block.statements,
                        chunk,
                        &mut else_locals,
                        return_type,
                        types,
                        signatures,
                        function_name,
                        loop_stack,
                    )?;
                    let after_else = chunk.instructions.len();
                    patch_jump(&mut chunk.instructions, jump_after_then, after_else);
                } else {
                    let after_then = chunk.instructions.len();
                    patch_jump(&mut chunk.instructions, jump_if_false, after_then);
                }
            }
            Statement::While(statement) => {
                let loop_start = chunk.instructions.len();
                let condition_type = lower_expression(
                    &statement.condition,
                    chunk,
                    locals,
                    types,
                    signatures,
                    Some(types.bool()),
                )?;
                if condition_type != types.bool() {
                    return Err(CompileError(
                        "`while` conditions must evaluate to `bool`".to_string(),
                    ));
                }

                let exit_jump = chunk.instructions.len();
                chunk
                    .instructions
                    .push(Instruction::JumpIfFalse(usize::MAX));

                loop_stack.push(LoopContext::default());
                let mut body_locals = locals.clone();
                lower_block(
                    &statement.body.statements,
                    chunk,
                    &mut body_locals,
                    return_type,
                    types,
                    signatures,
                    function_name,
                    loop_stack,
                )?;
                let loop_context = loop_stack.pop().expect("loop context should exist");
                patch_jumps(
                    &mut chunk.instructions,
                    &loop_context.continue_jumps,
                    loop_start,
                );

                chunk.instructions.push(Instruction::Jump(loop_start));
                let loop_end = chunk.instructions.len();
                patch_jump(&mut chunk.instructions, exit_jump, loop_end);
                patch_jumps(&mut chunk.instructions, &loop_context.break_jumps, loop_end);
            }
            Statement::For(statement) => {
                lower_for_loop(
                    statement,
                    chunk,
                    locals,
                    return_type,
                    types,
                    signatures,
                    function_name,
                    loop_stack,
                )?;
            }
            Statement::Break(_) => {
                let Some(loop_context) = loop_stack.last_mut() else {
                    return Err(CompileError(
                        "`break` can only be used inside a loop".to_string(),
                    ));
                };
                let jump = chunk.instructions.len();
                chunk.instructions.push(Instruction::Jump(usize::MAX));
                loop_context.break_jumps.push(jump);
            }
            Statement::Continue(_) => {
                let Some(loop_context) = loop_stack.last_mut() else {
                    return Err(CompileError(
                        "`continue` can only be used inside a loop".to_string(),
                    ));
                };
                let jump = chunk.instructions.len();
                chunk.instructions.push(Instruction::Jump(usize::MAX));
                loop_context.continue_jumps.push(jump);
            }
        }
    }

    Ok(())
}
