use std::collections::HashMap;

use crate::ast::syntax::{
    AssignStatement, AssignTarget, BinaryOperator, Expression, ExpressionKind, Identifier,
    Literal, StructLiteralField, TypeSyntax, UnaryOperator,
};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};
use crate::runtime::Value;

use super::types::{is_numeric_type, LocalBinding};
use super::utils::{require_same_equatable_types, require_same_numeric_types};

pub fn lower_expression(
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
            lower_array_literal(elements, chunk, locals, types, signatures, expected_type)
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
            lower_index_expression(target, index, chunk, locals, types, signatures)
        }
        ExpressionKind::Call { callee, arguments } => {
            lower_call_expression(callee, arguments, chunk, locals, types, signatures)
        }
        ExpressionKind::Range { .. } => Err(CompileError(
            "range expressions can only be used as `for` loop iterables".to_string(),
        )),
        ExpressionKind::Cast { target, expr } => {
            lower_cast_expression(target, expr, chunk, locals, types, signatures)
        }
        ExpressionKind::Unary { op, expr } => {
            lower_unary_expression(op, expr, chunk, locals, types, signatures)
        }
        ExpressionKind::Binary { left, op, right } => {
            let left_type = lower_expression(left, chunk, locals, types, signatures, None)?;
            let right_type = lower_expression(right, chunk, locals, types, signatures, None)?;
            lower_binary_operator(chunk, types, *op, left_type, right_type)
        }
    }
}

fn lower_array_literal(
    elements: &[Expression],
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    expected_type: Option<TypeId>,
) -> Result<TypeId, CompileError> {
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

pub fn lower_struct_literal(
    name: &TypeSyntax,
    fields: &[StructLiteralField],
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

pub fn lower_member_expression(
    target: &Expression,
    field: &Identifier,
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

fn lower_index_expression(
    target: &Expression,
    index: &Expression,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
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

fn lower_call_expression(
    callee: &Expression,
    arguments: &[Expression],
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
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

pub fn lower_assignment(
    statement: &AssignStatement,
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

fn lower_cast_expression(
    target: &TypeSyntax,
    expr: &Expression,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
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

fn lower_unary_expression(
    op: &UnaryOperator,
    expr: &Expression,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
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

fn lower_binary_operator(
    chunk: &mut Chunk,
    types: &TypeSystem,
    op: BinaryOperator,
    left: TypeId,
    right: TypeId,
) -> Result<TypeId, CompileError> {
    match op {
        BinaryOperator::Add => {
            let result_type = require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::Add);
            Ok(result_type)
        }
        BinaryOperator::Subtract => {
            let result_type = require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::Subtract);
            Ok(result_type)
        }
        BinaryOperator::Multiply => {
            let result_type = require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::Multiply);
            Ok(result_type)
        }
        BinaryOperator::Divide => {
            let result_type = require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::Divide);
            Ok(result_type)
        }
        BinaryOperator::Modulo => {
            if left == types.int() && right == types.int() {
                chunk.instructions.push(Instruction::Modulo);
                Ok(types.int())
            } else {
                Err(CompileError(
                    "modulo operator requires `int` operands".to_string(),
                ))
            }
        }
        BinaryOperator::Less => {
            require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::Less);
            Ok(types.bool())
        }
        BinaryOperator::Greater => {
            require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::Greater);
            Ok(types.bool())
        }
        BinaryOperator::LessEqual => {
            require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::LessEqual);
            Ok(types.bool())
        }
        BinaryOperator::GreaterEqual => {
            require_same_numeric_types(left, right, types)?;
            chunk.instructions.push(Instruction::GreaterEqual);
            Ok(types.bool())
        }
        BinaryOperator::Equal => {
            require_same_equatable_types(left, right, types)?;
            chunk.instructions.push(Instruction::Equal);
            Ok(types.bool())
        }
        BinaryOperator::NotEqual => {
            require_same_equatable_types(left, right, types)?;
            chunk.instructions.push(Instruction::NotEqual);
            Ok(types.bool())
        }
    }
}

fn direct_callee_name(callee: &Expression) -> Result<String, CompileError> {
    match &callee.kind {
        ExpressionKind::Variable(identifier) => Ok(identifier.name.clone()),
        ExpressionKind::Member { target, field } => {
            let ExpressionKind::Variable(module) = &target.kind else {
                return Err(CompileError(
                    "qualified calls must be in the form `module.function`".to_string(),
                ));
            };
            Ok(format!("{}.{}", module.name, field.name))
        }
        _ => Err(CompileError(
            "function calls must use a direct function name or qualified name".to_string(),
        )),
    }
}
