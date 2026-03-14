use std::collections::HashMap;

use crate::ast::syntax::{Expression, TypeSyntax, UnaryOperator, BinaryOperator};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{TypeId, TypeSystem};

use super::expressions::lower_expression;
use super::types::{is_numeric_type, LocalBinding};
use super::utils::{require_same_equatable_types, require_same_numeric_types};

pub fn lower_cast_expression(
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

pub fn lower_unary_expression(
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

pub fn lower_binary_operator(
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
