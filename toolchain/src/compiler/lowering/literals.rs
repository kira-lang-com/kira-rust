use std::collections::HashMap;

use crate::ast::{Expression, StructLiteralField, TypeSyntax};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};
use crate::runtime::Value;

use super::expressions::lower_expression;
use super::types::LocalBinding;

pub fn lower_array_literal(
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
