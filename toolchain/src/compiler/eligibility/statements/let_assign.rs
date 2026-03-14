use std::collections::HashMap;

use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::TypeSystem;

use super::super::expressions::analyze_expression;
use super::super::types::{type_is_native_eligible, LocalBinding};

pub fn analyze_let_statement(
    statement: &crate::ast::LetStatement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<bool, CompileError> {
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
    let profile = analyze_expression(
        &statement.value,
        locals,
        types,
        signatures,
        builtins,
        declared,
    )?;
    let local_type = declared.unwrap_or(profile.type_id);

    if !types.is_assignable(local_type, profile.type_id) {
        return Err(CompileError(format!(
            "cannot assign value of type {:?} to local `{}`",
            types.get(profile.type_id),
            statement.name.name
        )));
    }

    if !profile.native_eligible || !type_is_native_eligible(types, local_type) {
        return Ok(false);
    }

    locals.insert(
        statement.name.name.clone(),
        LocalBinding {
            type_id: local_type,
        },
    );
    Ok(true)
}
