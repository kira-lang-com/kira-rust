use std::collections::HashMap;

use crate::ast::{Expression, ExpressionKind, ForStatement};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};
use crate::runtime::Value;

use super::expressions::lower_expression;
use super::statements::lower_block;
use super::types::{LocalBinding, LoopContext};
use super::chunk_helpers::{allocate_local, patch_jump, patch_jumps};

pub fn lower_for_loop(
    statement: &ForStatement,
    chunk: &mut Chunk,
    locals: &mut HashMap<String, LocalBinding>,
    return_type: TypeId,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    function_name: &str,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<(), CompileError> {
    match &statement.iterable.kind {
        ExpressionKind::Range {
            start,
            end,
            inclusive,
        } => lower_range_for_loop(
            statement,
            start,
            end,
            *inclusive,
            chunk,
            locals,
            return_type,
            types,
            signatures,
            function_name,
            loop_stack,
        ),
        _ => lower_array_for_loop(
            statement,
            chunk,
            locals,
            return_type,
            types,
            signatures,
            function_name,
            loop_stack,
        ),
    }
}

fn lower_range_for_loop(
    statement: &ForStatement,
    start: &Expression,
    end: &Expression,
    inclusive: bool,
    chunk: &mut Chunk,
    locals: &mut HashMap<String, LocalBinding>,
    return_type: TypeId,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    function_name: &str,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<(), CompileError> {
    let int_type = types.int();

    let start_type = lower_expression(start, chunk, locals, types, signatures, Some(int_type))?;
    if start_type != int_type {
        return Err(CompileError("range start must be `int`".to_string()));
    }
    let index_slot = allocate_local(chunk);
    chunk.instructions.push(Instruction::StoreLocal(index_slot));

    let end_type = lower_expression(end, chunk, locals, types, signatures, Some(int_type))?;
    if end_type != int_type {
        return Err(CompileError("range end must be `int`".to_string()));
    }
    let end_slot = allocate_local(chunk);
    chunk.instructions.push(Instruction::StoreLocal(end_slot));

    let binding_slot = allocate_local(chunk);
    let loop_start = chunk.instructions.len();
    chunk.instructions.push(Instruction::LoadLocal(index_slot));
    chunk.instructions.push(Instruction::LoadLocal(end_slot));
    chunk.instructions.push(if inclusive {
        Instruction::LessEqual
    } else {
        Instruction::Less
    });
    let exit_jump = chunk.instructions.len();
    chunk
        .instructions
        .push(Instruction::JumpIfFalse(usize::MAX));

    chunk.instructions.push(Instruction::LoadLocal(index_slot));
    chunk
        .instructions
        .push(Instruction::StoreLocal(binding_slot));

    loop_stack.push(LoopContext::default());
    let mut body_locals = locals.clone();
    body_locals.insert(
        statement.binding.name.clone(),
        LocalBinding {
            slot: binding_slot,
            type_id: int_type,
        },
    );
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

    let continue_target = chunk.instructions.len();
    patch_jumps(
        &mut chunk.instructions,
        &loop_context.continue_jumps,
        continue_target,
    );
    let one = chunk.push_constant(Value::Int(1));
    chunk.instructions.push(Instruction::LoadLocal(index_slot));
    chunk.instructions.push(Instruction::LoadConst(one));
    chunk.instructions.push(Instruction::Add);
    chunk.instructions.push(Instruction::StoreLocal(index_slot));
    chunk.instructions.push(Instruction::Jump(loop_start));

    let loop_end = chunk.instructions.len();
    patch_jump(&mut chunk.instructions, exit_jump, loop_end);
    patch_jumps(&mut chunk.instructions, &loop_context.break_jumps, loop_end);
    Ok(())
}

fn lower_array_for_loop(
    statement: &ForStatement,
    chunk: &mut Chunk,
    locals: &mut HashMap<String, LocalBinding>,
    return_type: TypeId,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    function_name: &str,
    loop_stack: &mut Vec<LoopContext>,
) -> Result<(), CompileError> {
    let iterable_type =
        lower_expression(&statement.iterable, chunk, locals, types, signatures, None)?;
    let KiraType::Array(element_type) = types.get(iterable_type) else {
        return Err(CompileError(
            "`for` loops currently require an array or range iterable".to_string(),
        ));
    };

    let array_slot = allocate_local(chunk);
    chunk.instructions.push(Instruction::StoreLocal(array_slot));

    let zero = chunk.push_constant(Value::Int(0));
    let index_slot = allocate_local(chunk);
    chunk.instructions.push(Instruction::LoadConst(zero));
    chunk.instructions.push(Instruction::StoreLocal(index_slot));

    let binding_slot = allocate_local(chunk);
    let loop_start = chunk.instructions.len();
    chunk.instructions.push(Instruction::LoadLocal(index_slot));
    chunk.instructions.push(Instruction::LoadLocal(array_slot));
    chunk.instructions.push(Instruction::ArrayLength);
    chunk.instructions.push(Instruction::Less);
    let exit_jump = chunk.instructions.len();
    chunk
        .instructions
        .push(Instruction::JumpIfFalse(usize::MAX));

    chunk.instructions.push(Instruction::LoadLocal(array_slot));
    chunk.instructions.push(Instruction::LoadLocal(index_slot));
    chunk.instructions.push(Instruction::ArrayIndex);
    chunk
        .instructions
        .push(Instruction::StoreLocal(binding_slot));

    loop_stack.push(LoopContext::default());
    let mut body_locals = locals.clone();
    body_locals.insert(
        statement.binding.name.clone(),
        LocalBinding {
            slot: binding_slot,
            type_id: *element_type,
        },
    );
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

    let continue_target = chunk.instructions.len();
    patch_jumps(
        &mut chunk.instructions,
        &loop_context.continue_jumps,
        continue_target,
    );
    let one = chunk.push_constant(Value::Int(1));
    chunk.instructions.push(Instruction::LoadLocal(index_slot));
    chunk.instructions.push(Instruction::LoadConst(one));
    chunk.instructions.push(Instruction::Add);
    chunk.instructions.push(Instruction::StoreLocal(index_slot));
    chunk.instructions.push(Instruction::Jump(loop_start));

    let loop_end = chunk.instructions.len();
    patch_jump(&mut chunk.instructions, exit_jump, loop_end);
    patch_jumps(&mut chunk.instructions, &loop_context.break_jumps, loop_end);
    Ok(())
}
