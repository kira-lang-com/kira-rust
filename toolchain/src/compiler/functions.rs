// Function signature building and call helpers

use std::collections::HashMap;

use crate::ast::{Expression, ExternFunctionDefinition, ExpressionKind, FunctionDefinition};
use crate::runtime::type_system::TypeSystem;

use super::{BuiltinFunction, CompileError, CompiledFunction, FunctionSignature};

// Signature building

pub(super) fn collect_signatures(
    builtins: &HashMap<String, BuiltinFunction>,
    functions: &HashMap<String, CompiledFunction>,
) -> HashMap<String, FunctionSignature> {
    let mut signatures = HashMap::new();

    for builtin in builtins.values() {
        signatures.insert(builtin.name.clone(), builtin.signature.clone());
    }

    for function in functions.values() {
        signatures.insert(function.name.clone(), function.signature.clone());
    }

    signatures
}

pub(super) fn build_signature(
    types: &mut TypeSystem,
    function: &FunctionDefinition,
) -> Result<FunctionSignature, CompileError> {
    let mut params = Vec::with_capacity(function.params.len());
    for parameter in &function.params {
        let type_id = types
            .ensure_named(&parameter.type_name.name)
            .ok_or_else(|| {
                CompileError(format!(
                    "unknown parameter type `{}` on function `{}`",
                    parameter.type_name.name, function.name.name
                ))
            })?;
        params.push(type_id);
    }

    let return_type = match &function.return_type {
        Some(type_name) => types.ensure_named(&type_name.name).ok_or_else(|| {
            CompileError(format!(
                "unknown return type `{}` on function `{}`",
                type_name.name, function.name.name
            ))
        })?,
        None => types.unit(),
    };

    let function_type = types.register_function(params.clone(), return_type);

    Ok(FunctionSignature {
        params,
        return_type,
        function_type,
    })
}

pub(super) fn build_extern_signature(
    types: &mut TypeSystem,
    function: &ExternFunctionDefinition,
) -> Result<FunctionSignature, CompileError> {
    let mut params = Vec::with_capacity(function.params.len());
    for parameter in &function.params {
        let type_id = types
            .ensure_named(&parameter.type_name.name)
            .ok_or_else(|| {
                CompileError(format!(
                    "unknown parameter type `{}` on extern function `{}`",
                    parameter.type_name.name, function.name.name
                ))
            })?;
        params.push(type_id);
    }

    let return_type = match &function.return_type {
        Some(type_name) => types.ensure_named(&type_name.name).ok_or_else(|| {
            CompileError(format!(
                "unknown return type `{}` on extern function `{}`",
                type_name.name, function.name.name
            ))
        })?,
        None => types.unit(),
    };

    let function_type = types.register_function(params.clone(), return_type);

    Ok(FunctionSignature {
        params,
        return_type,
        function_type,
    })
}

// Call helpers

pub(super) fn direct_callee_name(callee: &Expression) -> Result<String, CompileError> {
    match &callee.kind {
        ExpressionKind::Variable(identifier) => Ok(identifier.name.clone()),
        _ => Err(CompileError(
            "only direct function calls are supported in the current compiler".to_string(),
        )),
    }
}
