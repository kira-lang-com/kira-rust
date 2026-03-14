use std::collections::HashMap;

use crate::ast::syntax::{Expression, ExpressionKind, Identifier, Literal};
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::calls::analyze_call_expression;
use super::literals::{analyze_array_literal, analyze_struct_literal};
use super::operators::{analyze_binary_result_type, analyze_cast_expression};
use super::types::{is_numeric_type, type_is_native_eligible, ExpressionProfile, LocalBinding};

pub use super::assignments::analyze_assignment;

pub fn analyze_expression(
    expression: &Expression,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    expected_type: Option<TypeId>,
) -> Result<ExpressionProfile, CompileError> {
    match &expression.kind {
        ExpressionKind::Literal(Literal::Bool(_)) => Ok(ExpressionProfile {
            type_id: types.bool(),
            native_eligible: true,
        }),
        ExpressionKind::Literal(Literal::Integer(_)) => Ok(ExpressionProfile {
            type_id: types.int(),
            native_eligible: true,
        }),
        ExpressionKind::Literal(Literal::Float(_)) => Ok(ExpressionProfile {
            type_id: types.float(),
            native_eligible: true,
        }),
        ExpressionKind::Literal(Literal::String(_)) => Ok(ExpressionProfile {
            type_id: types
                .resolve_named("string")
                .expect("built-in string type must exist"),
            native_eligible: true,
        }),
        ExpressionKind::ArrayLiteral(elements) => {
            analyze_array_literal(elements, locals, types, signatures, builtins, expected_type)
        }
        ExpressionKind::StructLiteral { name, fields } => {
            analyze_struct_literal(name, fields, locals, types, signatures, builtins)
        }
        ExpressionKind::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .ok_or_else(|| CompileError(format!("unknown variable `{}`", identifier.name)))?;
            Ok(ExpressionProfile {
                type_id: binding.type_id,
                native_eligible: type_is_native_eligible(types, binding.type_id),
            })
        }
        ExpressionKind::Member { target, field } => {
            analyze_member_expression(target, field, locals, types, signatures, builtins)
        }
        ExpressionKind::Index { target, index } => {
            analyze_index_expression(target, index, locals, types, signatures, builtins)
        }
        ExpressionKind::Call { callee, arguments } => {
            analyze_call_expression(callee, arguments, locals, types, signatures, builtins)
        }
        ExpressionKind::Range { .. } => Err(CompileError(
            "range expressions can only be used as `for` loop iterables".to_string(),
        )),
        ExpressionKind::Cast { target, expr } => {
            analyze_cast_expression(target, expr, locals, types, signatures, builtins)
        }
        ExpressionKind::Unary { expr, .. } => {
            let profile = analyze_expression(expr, locals, types, signatures, builtins, None)?;
            if !is_numeric_type(types, profile.type_id) {
                return Err(CompileError(
                    "unary negation currently requires an `int` or `float` operand".to_string(),
                ));
            }

            Ok(ExpressionProfile {
                type_id: profile.type_id,
                native_eligible: profile.native_eligible,
            })
        }
        ExpressionKind::Binary { left, right, .. } => {
            let left = analyze_expression(left, locals, types, signatures, builtins, None)?;
            let right = analyze_expression(right, locals, types, signatures, builtins, None)?;
            let type_id =
                analyze_binary_result_type(types, left.type_id, right.type_id, expression)?;
            Ok(ExpressionProfile {
                type_id,
                native_eligible: left.native_eligible
                    && right.native_eligible
                    && type_is_native_eligible(types, type_id),
            })
        }
    }
}

fn analyze_index_expression(
    target: &Expression,
    index: &Expression,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
    let target_profile =
        analyze_expression(target, locals, types, signatures, builtins, None)?;
    let index_profile = analyze_expression(
        index,
        locals,
        types,
        signatures,
        builtins,
        Some(types.int()),
    )?;
    if index_profile.type_id != types.int() {
        return Err(CompileError("array indices must be `int`".to_string()));
    }
    match types.get(target_profile.type_id) {
        KiraType::Array(element_type) => Ok(ExpressionProfile {
            type_id: *element_type,
            native_eligible: target_profile.native_eligible
                && index_profile.native_eligible
                && type_is_native_eligible(types, *element_type),
        }),
        _ => Err(CompileError("indexing requires an array value".to_string())),
    }
}

pub fn analyze_member_expression(
    target: &Expression,
    field: &Identifier,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
    let target_profile = analyze_expression(target, locals, types, signatures, builtins, None)?;

    match types.get(target_profile.type_id) {
        KiraType::Array(_) if field.name == "length" => Ok(ExpressionProfile {
            type_id: types.int(),
            native_eligible: target_profile.native_eligible,
        }),
        KiraType::Array(_) => Err(CompileError(format!(
            "unknown array member `{}`",
            field.name
        ))),
        KiraType::Struct(struct_type) => {
            let (_, field_type) = types.struct_field(target_profile.type_id, &field.name).ok_or_else(|| {
                CompileError(format!("{} has no field '{}'", struct_type.name, field.name))
            })?;
            Ok(ExpressionProfile {
                type_id: field_type,
                native_eligible: target_profile.native_eligible
                    && type_is_native_eligible(types, field_type),
            })
        }
        _ => Err(CompileError(format!(
            "type `{}` has no field `{}`",
            types.type_name(target_profile.type_id),
            field.name
        ))),
    }
}
