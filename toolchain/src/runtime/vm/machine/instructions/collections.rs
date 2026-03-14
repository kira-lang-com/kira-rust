use crate::compiler::CompiledModule;
use crate::runtime::value::StructValue;
use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

use super::super::stack::store_struct_field;

pub fn execute_build_array(
    stack: &mut Vec<Value>,
    element_count: usize,
) -> Result<Option<usize>, RuntimeError> {
    let mut values = Vec::with_capacity(element_count);
    for _ in 0..element_count {
        values.push(
            stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while building array".to_string()))?,
        );
    }
    values.reverse();
    stack.push(Value::Array(values));
    Ok(None)
}

pub fn execute_build_struct(
    stack: &mut Vec<Value>,
    module: &CompiledModule,
    type_id: crate::runtime::type_system::TypeId,
    field_count: usize,
) -> Result<Option<usize>, RuntimeError> {
    let mut values = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        values.push(
            stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while building struct".to_string()))?,
        );
    }
    values.reverse();

    let crate::runtime::type_system::KiraType::Struct(struct_type) = module.types.get(type_id)
    else {
        return Err(RuntimeError(format!(
            "invalid struct type id {:?} in bytecode",
            type_id
        )));
    };

    if struct_type.fields.len() != values.len() {
        return Err(RuntimeError(format!(
            "struct `{}` expected {} fields but bytecode provided {}",
            struct_type.name,
            struct_type.fields.len(),
            values.len()
        )));
    }

    stack.push(Value::Struct(StructValue {
        type_name: struct_type.name.clone(),
        fields: struct_type
            .fields
            .iter()
            .map(|field| field.name.clone())
            .zip(values.into_iter())
            .collect(),
    }));
    Ok(None)
}

pub fn execute_array_length(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading array length".to_string()))?;
    match value {
        Value::Array(values) => stack.push(Value::Int(values.len() as i64)),
        value => {
            return Err(RuntimeError(format!(
                "expected array for `.length`, got {:?}",
                value
            )))
        }
    }
    Ok(None)
}

pub fn execute_array_index(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let index = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading array index".to_string()))?;
    let target = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading array target".to_string()))?;
    let Value::Int(index) = index else {
        return Err(RuntimeError(format!(
            "array index must be int, got {:?}",
            index
        )));
    };
    let Value::Array(values) = target else {
        return Err(RuntimeError(format!(
            "expected array for indexing, got {:?}",
            target
        )));
    };
    if index < 0 || index as usize >= values.len() {
        return Err(RuntimeError(format!(
            "array index {} out of bounds for length {}",
            index,
            values.len()
        )));
    }
    stack.push(values[index as usize].clone());
    Ok(None)
}

pub fn execute_struct_field(
    stack: &mut Vec<Value>,
    index: usize,
) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading struct field".to_string()))?;
    match value {
        Value::Struct(struct_value) => {
            let (_, field_value) = struct_value.fields.get(index).ok_or_else(|| {
                RuntimeError(format!(
                    "struct `{}` has no field at index {}",
                    struct_value.type_name, index
                ))
            })?;
            stack.push(field_value.clone());
        }
        value => {
            return Err(RuntimeError(format!(
                "expected struct for field access, got {:?}",
                value
            )))
        }
    }
    Ok(None)
}

pub fn execute_store_local_field(
    stack: &mut Vec<Value>,
    locals: &mut Vec<Value>,
    local: usize,
    path: &[usize],
) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while storing struct field".to_string()))?;
    let target = locals
        .get_mut(local)
        .ok_or_else(|| RuntimeError(format!("invalid local index {local}")))?;
    store_struct_field(target, path, value)?;
    Ok(None)
}

pub fn execute_array_append_local(
    stack: &mut Vec<Value>,
    locals: &mut Vec<Value>,
    index: usize,
) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while appending to array".to_string()))?;
    let local = locals
        .get_mut(index)
        .ok_or_else(|| RuntimeError(format!("invalid local index {index}")))?;
    match local {
        Value::Array(values) => {
            values.push(value);
        }
        other => {
            return Err(RuntimeError(format!(
                "expected array local for append, got {:?}",
                other
            )))
        }
    }
    Ok(None)
}
