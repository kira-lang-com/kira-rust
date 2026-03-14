use std::collections::HashMap;

use crate::ast::syntax::{Expression, ExpressionKind, Identifier, Literal};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};
use crate::runtime::Value;

use super::assignments::lower_assignment;
use super::calls::lower_call_expression;
use super::literals::{lower_array_literal, lower_struct_literal};
use super::operators::{lower_binary_operator, lower_cast_expression, lower_unary_expression};
use super::types::LocalBinding;

pub use super::assignments::lower_assignment as pub_lower_assignment;

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

