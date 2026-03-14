use std::collections::HashMap;

use crate::ast::FunctionDefinition;
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::TypeSystem;

use super::statements::analyze_statement;
use super::types::{type_is_native_eligible, LocalBinding};

pub fn is_native_eligible(
    function: &FunctionDefinition,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<bool, CompileError> {
    let signature = signatures
        .get(&function.name.name)
        .ok_or_else(|| CompileError(format!("missing signature for `{}`", function.name.name)))?;

    if !signature
        .params
        .iter()
        .copied()
        .all(|type_id| type_is_native_eligible(types, type_id))
    {
        return Ok(false);
    }

    if !type_is_native_eligible(types, signature.return_type) {
        return Ok(false);
    }

    let mut locals = HashMap::new();
    for (slot, parameter) in function.params.iter().enumerate() {
        let _ = slot;
        locals.insert(
            parameter.name.name.clone(),
            LocalBinding {
                type_id: signature.params[slot],
            },
        );
    }

    for statement in &function.body.statements {
        if !analyze_statement(statement, &mut locals, types, signatures, builtins, 0)? {
            return Ok(false);
        }
    }

    Ok(true)
}
