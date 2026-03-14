use std::collections::HashMap;

use crate::ast::syntax::{
    AssignTarget, BinaryOperator, Expression, ExpressionKind, ForStatement, FunctionDefinition,
    Literal, Statement, UnaryOperator,
};
use crate::runtime::{
    type_system::{KiraType, TypeId, TypeSystem},
    Value,
};

use super::{calls::direct_callee_name, Chunk, CompileError, FunctionSignature, Instruction};

#[derive(Clone, Copy)]
struct LocalBinding {
    slot: usize,
    type_id: TypeId,
}

#[derive(Default)]
struct LoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
}

pub(super) fn lower_function_body(
    function: &FunctionDefinition,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<Chunk, CompileError> {
    let signature = signatures
        .get(&function.name.name)
        .ok_or_else(|| CompileError(format!("missing signature for `{}`", function.name.name)))?;

    let mut chunk = Chunk {
        instructions: Vec::new(),
        constants: Vec::new(),
        local_count: function.params.len(),
        local_types: signature.params.clone(),
    };
    let mut locals = HashMap::new();

    for (slot, parameter) in function.params.iter().enumerate() {
        locals.insert(
            parameter.name.name.clone(),
            LocalBinding {
                slot,
                type_id: signature.params[slot],
            },
        );
    }

    lower_block(
        &function.body.statements,
        &mut chunk,
        &mut locals,
        signature.return_type,
        types,
        signatures,
        &function.name.name,
        &mut Vec::new(),
    )?;

    if !matches!(chunk.instructions.last(), Some(Instruction::Return)) {
        if signature.return_type != types.unit() {
            let unit = chunk.push_constant(Value::Unit);
            chunk.instructions.push(Instruction::LoadConst(unit));
        }
        chunk.instructions.push(Instruction::Return);
    }

    Ok(chunk)
}

fn lower_block(
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

fn lower_for_loop(
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

fn lower_expression(
    expression: &Expression,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    expected_type: Option<TypeId>,
) -> Result<TypeId, CompileError> {
    match &expression.kind {
        ExpressionKind::Literal(Literal::Bool(value)) => {
            let index = chunk.push_constant(Value::Bool(*value));
            chunk.instructions.push(Instruction::LoadConst(index));
            Ok(types.bool())
        }
        ExpressionKind::Literal(Literal::Integer(value)) => {
            let index = chunk.push_constant(Value::Int(*value));
            chunk.instructions.push(Instruction::LoadConst(index));
            Ok(types.int())
        }
        ExpressionKind::Literal(Literal::Float(value)) => {
            let index = chunk.push_constant(Value::Float(*value));
            chunk.instructions.push(Instruction::LoadConst(index));
            Ok(types.float())
        }
        ExpressionKind::Literal(Literal::String(value)) => {
            let index = chunk.push_constant(Value::String(value.clone()));
            chunk.instructions.push(Instruction::LoadConst(index));
            Ok(types
                .resolve_named("string")
                .expect("built-in string type must exist"))
        }
        ExpressionKind::ArrayLiteral(elements) => {
            let expected_element = expected_type.and_then(|type_id| match types.get(type_id) {
                KiraType::Array(element) => Some(*element),
                _ => None,
            });

            let mut element_type = expected_element;
            for element in elements {
                let current_type =
                    lower_expression(element, chunk, locals, types, signatures, element_type)?;
                match element_type {
                    Some(expected_element) => {
                        if !types.is_assignable(expected_element, current_type) {
                            return Err(CompileError(format!(
                                "array literal element has type {:?}, expected {:?}",
                                types.get(current_type),
                                types.get(expected_element)
                            )));
                        }
                    }
                    None => element_type = Some(current_type),
                }
            }

            let Some(element_type) = element_type else {
                return Err(CompileError(
                    "cannot infer type of empty array literal".to_string(),
                ));
            };

            let array_type = types.register_array(element_type);
            chunk.instructions.push(Instruction::BuildArray {
                type_id: array_type,
                element_count: elements.len(),
            });
            Ok(array_type)
        }
        ExpressionKind::StructLiteral { name, fields } => {
            lower_struct_literal(name, fields, chunk, locals, types, signatures)
        }
        ExpressionKind::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .ok_or_else(|| CompileError(format!("unknown variable `{}`", identifier.name)))?;
            chunk
                .instructions
                .push(Instruction::LoadLocal(binding.slot));
            Ok(binding.type_id)
        }
        ExpressionKind::Member { target, field } => {
            lower_member_expression(target, field, chunk, locals, types, signatures)
        }
        ExpressionKind::Index { target, index } => {
            let target_type = lower_expression(target, chunk, locals, types, signatures, None)?;
            let index_type =
                lower_expression(index, chunk, locals, types, signatures, Some(types.int()))?;

            if index_type != types.int() {
                return Err(CompileError("array indices must be `int`".to_string()));
            }

            match types.get(target_type) {
                KiraType::Array(element_type) => {
                    chunk.instructions.push(Instruction::ArrayIndex);
                    Ok(*element_type)
                }
                _ => Err(CompileError("indexing requires an array value".to_string())),
            }
        }
        ExpressionKind::Call { callee, arguments } => {
            if let Some(return_type) =
                lower_special_call(callee, arguments, chunk, locals, types, signatures)?
            {
                return Ok(return_type);
            }

            let callee_name = direct_callee_name(callee)?;
            let signature = signatures
                .get(&callee_name)
                .ok_or_else(|| CompileError(format!("unknown function `{callee_name}`")))?;

            if arguments.len() != signature.params.len() {
                return Err(CompileError(format!(
                    "function `{callee_name}` expects {} arguments but got {}",
                    signature.params.len(),
                    arguments.len()
                )));
            }

            for (index, argument) in arguments.iter().enumerate() {
                let expected = signature.params[index];
                let arg_type =
                    lower_expression(argument, chunk, locals, types, signatures, Some(expected))?;
                if !types.is_assignable(expected, arg_type) {
                    return Err(CompileError(format!(
                        "argument {} for `{}` has type {:?}, expected {:?}",
                        index,
                        callee_name,
                        types.get(arg_type),
                        types.get(expected)
                    )));
                }
            }

            chunk.instructions.push(Instruction::Call {
                function: callee_name,
                arg_count: arguments.len(),
            });
            Ok(signature.return_type)
        }
        ExpressionKind::Range { .. } => Err(CompileError(
            "range expressions can only be used as `for` loop iterables".to_string(),
        )),
        ExpressionKind::Cast { target, expr } => {
            let value_type = lower_expression(expr, chunk, locals, types, signatures, None)?;
            let target_type = types
                .resolve_named(&target.name)
                .ok_or_else(|| CompileError(format!("unknown cast target `{}`", target.name)))?;

            if target_type != types.float() {
                return Err(CompileError(format!(
                    "unsupported cast target `{}`",
                    target.name
                )));
            }

            if value_type == types.float() {
                return Ok(types.float());
            }

            if value_type != types.int() {
                return Err(CompileError(
                    "`float()` can only convert from `int` or `float`".to_string(),
                ));
            }

            chunk.instructions.push(Instruction::CastIntToFloat);
            Ok(types.float())
        }
        ExpressionKind::Unary { op, expr } => {
            let value_type = lower_expression(expr, chunk, locals, types, signatures, None)?;
            if !is_numeric_type(types, value_type) {
                return Err(CompileError(
                    "unary negation currently requires an `int` or `float` operand".to_string(),
                ));
            }

            match op {
                UnaryOperator::Negate => chunk.instructions.push(Instruction::Negate),
            }

            Ok(value_type)
        }
        ExpressionKind::Binary { left, op, right } => {
            let left_type = lower_expression(left, chunk, locals, types, signatures, None)?;
            let right_type = lower_expression(right, chunk, locals, types, signatures, None)?;
            lower_binary_operator(chunk, types, *op, left_type, right_type)
        }
    }
}

fn lower_special_call(
    callee: &Expression,
    arguments: &[Expression],
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<Option<TypeId>, CompileError> {
    let ExpressionKind::Member { target, field } = &callee.kind else {
        return Ok(None);
    };
    let ExpressionKind::Variable(identifier) = &target.kind else {
        return Ok(None);
    };

    let target_name = &identifier.name;
    let member_name = &field.name;
    let Some(binding) = locals.get(target_name).copied() else {
        return Ok(None);
    };

    match (types.get(binding.type_id), member_name.as_str()) {
        (KiraType::Array(element_type), "append") => {
            let element_type = *element_type;
            if arguments.len() != 1 {
                return Err(CompileError(format!(
                    "`{}.append` expects 1 argument but got {}",
                    target_name,
                    arguments.len()
                )));
            }

            let arg_type = lower_expression(
                &arguments[0],
                chunk,
                locals,
                types,
                signatures,
                Some(element_type),
            )?;
            if !types.is_assignable(element_type, arg_type) {
                return Err(CompileError(format!(
                    "array append expected {:?}, got {:?}",
                    types.get(element_type),
                    types.get(arg_type)
                )));
            }

            chunk
                .instructions
                .push(Instruction::ArrayAppendLocal(binding.slot));
            Ok(Some(types.unit()))
        }
        (KiraType::Array(_), _) => Err(CompileError(format!(
            "unknown array method `{}`",
            member_name
        ))),
        _ => Ok(None),
    }
}

fn lower_assignment(
    statement: &crate::ast::syntax::AssignStatement,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<(), CompileError> {
    match &statement.target {
        AssignTarget::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .copied()
                .ok_or_else(|| CompileError(format!("unknown local `{}`", identifier.name)))?;
            let value_type = lower_expression(
                &statement.value,
                chunk,
                locals,
                types,
                signatures,
                Some(binding.type_id),
            )?;
            if !types.is_assignable(binding.type_id, value_type) {
                return Err(CompileError(format!(
                    "cannot assign value of type {:?} to local `{}`",
                    types.get(value_type),
                    identifier.name
                )));
            }
            chunk.instructions.push(Instruction::StoreLocal(binding.slot));
            Ok(())
        }
        AssignTarget::Field { .. } => {
            let (binding, path, field_type) = resolve_assign_target(&statement.target, locals, types)?;
            let value_type = lower_expression(
                &statement.value,
                chunk,
                locals,
                types,
                signatures,
                Some(field_type),
            )?;
            if !types.is_assignable(field_type, value_type) {
                return Err(CompileError(format!(
                    "cannot assign value of type {:?} to field of type {:?}",
                    types.get(value_type),
                    types.get(field_type)
                )));
            }

            chunk.instructions.push(Instruction::StoreLocalField {
                local: binding.slot,
                path,
            });
            Ok(())
        }
    }
}

fn resolve_assign_target(
    target: &AssignTarget,
    locals: &HashMap<String, LocalBinding>,
    types: &TypeSystem,
) -> Result<(LocalBinding, Vec<usize>, TypeId), CompileError> {
    match target {
        AssignTarget::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .copied()
                .ok_or_else(|| CompileError(format!("unknown local `{}`", identifier.name)))?;
            Ok((binding, Vec::new(), binding.type_id))
        }
        AssignTarget::Field { target, field, .. } => {
            let (binding, mut path, owner_type) = resolve_assign_target(target, locals, types)?;
            match types.get(owner_type) {
                KiraType::Struct(struct_type) => {
                    let (field_index, field_type) = types
                        .struct_field(owner_type, &field.name)
                        .ok_or_else(|| {
                            CompileError(format!("{} has no field '{}'", struct_type.name, field.name))
                        })?;
                    path.push(field_index);
                    Ok((binding, path, field_type))
                }
                KiraType::Array(_) => Err(CompileError(format!(
                    "cannot assign to array member `{}`",
                    field.name
                ))),
                _ => Err(CompileError(format!(
                    "type `{}` has no assignable field `{}`",
                    types.type_name(owner_type),
                    field.name
                ))),
            }
        }
    }
}

fn lower_struct_literal(
    name: &crate::ast::syntax::TypeSyntax,
    fields: &[crate::ast::syntax::StructLiteralField],
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
    let struct_type = types
        .resolve_named(&name.name)
        .ok_or_else(|| CompileError(format!("unknown type `{}`", name.name)))?;
    let declared_fields = types
        .struct_fields(struct_type)
        .ok_or_else(|| CompileError(format!("`{}` is not a struct type", name.name)))?
        .to_vec();

    let mut provided = HashMap::new();
    for field in fields {
        if provided.insert(field.name.name.clone(), &field.value).is_some() {
            return Err(CompileError(format!(
                "struct literal for `{}` sets field `{}` more than once",
                name.name, field.name.name
            )));
        }
    }

    for field in fields {
        if !declared_fields.iter().any(|declared| declared.name == field.name.name) {
            return Err(CompileError(format!(
                "{} has no field '{}'",
                name.name, field.name.name
            )));
        }
    }

    for declared in &declared_fields {
        let value = provided.get(&declared.name).ok_or_else(|| {
            CompileError(format!(
                "struct literal for `{}` is missing field `{}`",
                name.name, declared.name
            ))
        })?;
        let value_type = lower_expression(
            value,
            chunk,
            locals,
            types,
            signatures,
            Some(declared.type_id),
        )?;
        if !types.is_assignable(declared.type_id, value_type) {
            return Err(CompileError(format!(
                "field `{}` on `{}` has type {:?}, expected {:?}",
                declared.name,
                name.name,
                types.get(value_type),
                types.get(declared.type_id)
            )));
        }
    }

    chunk.instructions.push(Instruction::BuildStruct {
        type_id: struct_type,
        field_count: declared_fields.len(),
    });
    Ok(struct_type)
}

fn lower_member_expression(
    target: &Expression,
    field: &crate::ast::syntax::Identifier,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
    let target_type = lower_expression(target, chunk, locals, types, signatures, None)?;

    match types.get(target_type) {
        KiraType::Array(_) if field.name == "length" => {
            chunk.instructions.push(Instruction::ArrayLength);
            Ok(types.int())
        }
        KiraType::Array(_) => Err(CompileError(format!(
            "unknown array member `{}`",
            field.name
        ))),
        KiraType::Struct(struct_type) => {
            let (field_index, field_type) = types.struct_field(target_type, &field.name).ok_or_else(|| {
                CompileError(format!("{} has no field '{}'", struct_type.name, field.name))
            })?;
            chunk.instructions.push(Instruction::StructField(field_index));
            Ok(field_type)
        }
        _ => Err(CompileError(format!(
            "type `{}` has no field `{}`",
            types.type_name(target_type),
            field.name
        ))),
    }
}

fn lower_binary_operator(
    chunk: &mut Chunk,
    types: &TypeSystem,
    op: BinaryOperator,
    left_type: TypeId,
    right_type: TypeId,
) -> Result<TypeId, CompileError> {
    match op {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => {
            require_same_numeric_types(types, left_type, right_type)?;
            chunk.instructions.push(match op {
                BinaryOperator::Add => Instruction::Add,
                BinaryOperator::Subtract => Instruction::Subtract,
                BinaryOperator::Multiply => Instruction::Multiply,
                BinaryOperator::Divide => Instruction::Divide,
                _ => unreachable!(),
            });
            Ok(left_type)
        }
        BinaryOperator::Modulo => {
            if left_type != types.int() || right_type != types.int() {
                return Err(CompileError(
                    "modulo currently requires `int` operands".to_string(),
                ));
            }

            chunk.instructions.push(Instruction::Modulo);
            Ok(types.int())
        }
        BinaryOperator::Less | BinaryOperator::Greater => {
            require_same_numeric_types(types, left_type, right_type)?;
            chunk.instructions.push(match op {
                BinaryOperator::Less => Instruction::Less,
                BinaryOperator::Greater => Instruction::Greater,
                _ => unreachable!(),
            });
            Ok(types.bool())
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            require_same_equatable_types(types, left_type, right_type)?;
            chunk.instructions.push(match op {
                BinaryOperator::Equal => Instruction::Equal,
                BinaryOperator::NotEqual => Instruction::NotEqual,
                _ => unreachable!(),
            });
            Ok(types.bool())
        }
        BinaryOperator::LessEqual | BinaryOperator::GreaterEqual => {
            require_same_numeric_types(types, left_type, right_type)?;
            chunk.instructions.push(match op {
                BinaryOperator::LessEqual => Instruction::LessEqual,
                BinaryOperator::GreaterEqual => Instruction::GreaterEqual,
                _ => unreachable!(),
            });
            Ok(types.bool())
        }
    }
}

fn is_numeric_type(types: &TypeSystem, type_id: TypeId) -> bool {
    matches!(types.get(type_id), KiraType::Int | KiraType::Float)
}

fn is_equatable_type(types: &TypeSystem, type_id: TypeId) -> bool {
    matches!(
        types.get(type_id),
        KiraType::Bool
            | KiraType::Int
            | KiraType::Float
            | KiraType::String
            | KiraType::Array(_)
            | KiraType::Struct(_)
    )
}

fn require_same_numeric_types(
    types: &TypeSystem,
    left_type: TypeId,
    right_type: TypeId,
) -> Result<(), CompileError> {
    if left_type != right_type || !is_numeric_type(types, left_type) {
        return Err(CompileError(
            "numeric operations require operands of the same `int` or `float` type".to_string(),
        ));
    }

    Ok(())
}

fn require_same_equatable_types(
    types: &TypeSystem,
    left_type: TypeId,
    right_type: TypeId,
) -> Result<(), CompileError> {
    if left_type != right_type || !is_equatable_type(types, left_type) {
        return Err(CompileError(
            "comparison requires operands of the same comparable type".to_string(),
        ));
    }

    Ok(())
}

fn allocate_local(chunk: &mut Chunk) -> usize {
    let slot = chunk.local_count;
    chunk.local_count += 1;
    if chunk.local_types.len() <= slot {
        chunk.local_types.resize(slot + 1, TypeId(usize::MAX));
    }
    slot
}

fn ensure_local_type(chunk: &mut Chunk, slot: usize, type_id: TypeId) {
    if chunk.local_types.len() <= slot {
        chunk.local_types.resize(slot + 1, TypeId(usize::MAX));
    }
    chunk.local_types[slot] = type_id;
}

fn patch_jump(instructions: &mut [Instruction], index: usize, target: usize) {
    match instructions.get_mut(index) {
        Some(Instruction::JumpIfFalse(slot)) | Some(Instruction::Jump(slot)) => {
            *slot = target;
        }
        _ => unreachable!("jump patch requested for non-jump instruction"),
    }
}

fn patch_jumps(instructions: &mut [Instruction], indexes: &[usize], target: usize) {
    for index in indexes {
        patch_jump(instructions, *index, target);
    }
}
