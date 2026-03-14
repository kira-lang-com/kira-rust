use std::collections::HashMap;

use crate::ast::syntax::{Expression, ExpressionKind};
use crate::compiler::{calls::direct_callee_name, BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::expressions::analyze_expression;
use super::types::{type_is_native_eligible, ExpressionProfile, LocalBinding};

pub fn analyze_call_expression(
    callee: &Expression,
    arguments: &[Expression],
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
    if let Some(profile) =
        analyze_special_call(callee, arguments, locals, types, signatures, builtins)?
    {
        return Ok(profile);
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

    let builtin_native = builtins
        .get(&callee_name)
        .map(|builtin| builtin.backend == super::super::BackendKind::Native)
        .unwrap_or(true);

    let mut native_eligible = builtin_native && type_is_native_eligible(types, signature.return_type);
    if callee_name != "printIn" {
        native_eligible &= signature
            .params
            .iter()
            .copied()
            .all(|type_id| type_is_native_eligible(types, type_id));
    }

    for (index, argument) in arguments.iter().enumerate() {
        let expected = signature.params[index];
        let profile = analyze_expression(
            argument,
            locals,
            types,
            signatures,
            builtins,
            Some(expected),
        )?;
        if !types.is_assignable(expected, profile.type_id) {
            return Err(CompileError(format!(
                "argument {} for `{}` has type {:?}, expected {:?}",
                index,
                callee_name,
                types.get(profile.type_id),
                types.get(expected)
            )));
        }
        native_eligible &= profile.native_eligible;
    }

    Ok(ExpressionProfile {
        type_id: signature.return_type,
        native_eligible,
    })
}

fn analyze_special_call(
    callee: &Expression,
    arguments: &[Expression],
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<Option<ExpressionProfile>, CompileError> {
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
            let profile = analyze_expression(
                &arguments[0],
                locals,
                types,
                signatures,
                builtins,
                Some(element_type),
            )?;
            if !types.is_assignable(element_type, profile.type_id) {
                return Err(CompileError(format!(
                    "array append expected {:?}, got {:?}",
                    types.get(element_type),
                    types.get(profile.type_id)
                )));
            }
            Ok(Some(ExpressionProfile {
                type_id: types.unit(),
                native_eligible: profile.native_eligible,
            }))
        }
        (KiraType::Array(_), _) => Err(CompileError(format!(
            "unknown array method `{}`",
            member_name
        ))),
        _ => Ok(None),
    }
}
