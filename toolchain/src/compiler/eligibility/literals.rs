use std::collections::HashMap;

use crate::ast::{Expression, StructLiteralField, TypeSyntax};
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::expressions::analyze_expression;
use super::types::{type_is_native_eligible, ExpressionProfile, LocalBinding};

pub fn analyze_array_literal(
    elements: &[Expression],
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    expected_type: Option<TypeId>,
) -> Result<ExpressionProfile, CompileError> {
    let expected_element = expected_type.and_then(|type_id| match types.get(type_id) {
        KiraType::Array(element) => Some(*element),
        _ => None,
    });

    let mut element_type = expected_element;
    let mut native_eligible = true;
    for element in elements {
        let profile =
            analyze_expression(element, locals, types, signatures, builtins, element_type)?;
        native_eligible &= profile.native_eligible;
        match element_type {
            Some(expected_element) => {
                if !types.is_assignable(expected_element, profile.type_id) {
                    return Err(CompileError(format!(
                        "array literal element has type {:?}, expected {:?}",
                        types.get(profile.type_id),
                        types.get(expected_element)
                    )));
                }
            }
            None => element_type = Some(profile.type_id),
        }
    }

    let Some(element_type) = element_type else {
        return Err(CompileError(
            "cannot infer type of empty array literal".to_string(),
        ));
    };
    let array_type = types.register_array(element_type);
    Ok(ExpressionProfile {
        type_id: array_type,
        native_eligible: native_eligible && type_is_native_eligible(types, array_type),
    })
}

pub fn analyze_struct_literal(
    name: &TypeSyntax,
    fields: &[StructLiteralField],
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
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

    let mut native_eligible = type_is_native_eligible(types, struct_type);
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
        let profile = analyze_expression(
            value,
            locals,
            types,
            signatures,
            builtins,
            Some(declared.type_id),
        )?;
        if !types.is_assignable(declared.type_id, profile.type_id) {
            return Err(CompileError(format!(
                "field `{}` on `{}` has type {:?}, expected {:?}",
                declared.name,
                name.name,
                types.get(profile.type_id),
                types.get(declared.type_id)
            )));
        }
        native_eligible &= profile.native_eligible;
    }

    Ok(ExpressionProfile {
        type_id: struct_type,
        native_eligible,
    })
}
