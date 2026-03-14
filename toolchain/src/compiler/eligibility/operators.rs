use std::collections::HashMap;

use crate::ast::syntax::{BinaryOperator, Expression, ExpressionKind, TypeSyntax};
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{TypeId, TypeSystem};

use super::expressions::analyze_expression;
use super::types::{is_equatable_type, is_numeric_type, ExpressionProfile, LocalBinding};

pub fn analyze_cast_expression(
    target: &TypeSyntax,
    expr: &Expression,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
    let profile = analyze_expression(expr, locals, types, signatures, builtins, None)?;
    let target_type = types
        .resolve_named(&target.name)
        .ok_or_else(|| CompileError(format!("unknown cast target `{}`", target.name)))?;

    if target_type != types.float() {
        return Err(CompileError(format!(
            "unsupported cast target `{}`",
            target.name
        )));
    }

    if profile.type_id != types.int() && profile.type_id != types.float() {
        return Err(CompileError(
            "`float()` can only convert from `int` or `float`".to_string(),
        ));
    }

    Ok(ExpressionProfile {
        type_id: types.float(),
        native_eligible: profile.native_eligible,
    })
}

pub fn analyze_binary_result_type(
    types: &TypeSystem,
    left_type: TypeId,
    right_type: TypeId,
    expression: &Expression,
) -> Result<TypeId, CompileError> {
    let ExpressionKind::Binary { op, .. } = &expression.kind else {
        unreachable!()
    };

    match op {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => {
            if left_type != right_type || !is_numeric_type(types, left_type) {
                return Err(CompileError(
                    "numeric operations require operands of the same `int` or `float` type"
                        .to_string(),
                ));
            }
            Ok(left_type)
        }
        BinaryOperator::Modulo => {
            if left_type != types.int() || right_type != types.int() {
                return Err(CompileError(
                    "modulo currently requires `int` operands".to_string(),
                ));
            }
            Ok(types.int())
        }
        BinaryOperator::Less
        | BinaryOperator::Greater
        | BinaryOperator::LessEqual
        | BinaryOperator::GreaterEqual => {
            if left_type != right_type || !is_numeric_type(types, left_type) {
                return Err(CompileError(
                    "ordered comparison requires operands of the same `int` or `float` type"
                        .to_string(),
                ));
            }
            Ok(types.bool())
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            if left_type != right_type || !is_equatable_type(types, left_type) {
                return Err(CompileError(
                    "comparison requires operands of the same comparable type".to_string(),
                ));
            }
            Ok(types.bool())
        }
    }
}
