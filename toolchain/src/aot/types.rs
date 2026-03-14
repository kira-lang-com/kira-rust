use crate::compiler::CompiledModule;
use crate::runtime::type_system::{KiraType, TypeId};

use super::error::AotError;

pub fn rust_abi_type_name(module: &CompiledModule, type_id: TypeId) -> Result<&'static str, AotError> {
    match module.types.get(type_id) {
        KiraType::Unit => Ok("()"),
        KiraType::Bool => Ok("bool"),
        KiraType::Int => Ok("i64"),
        KiraType::Float => Ok("f64"),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            Ok("*mut c_void")
        }
        other => Err(AotError(format!("runner ABI does not yet support type {:?}", other))),
    }
}

pub fn wrap_rust_result(
    module: &CompiledModule,
    name: &str,
    type_id: TypeId,
) -> Result<String, AotError> {
    let value = match module.types.get(type_id) {
        KiraType::Bool => format!("Value::Bool({name})"),
        KiraType::Int => format!("Value::Int({name})"),
        KiraType::Float => format!("Value::Float(ordered_float::OrderedFloat({name}))"),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            format!("unsafe {{ *Box::from_raw({name} as *mut Value) }}")
        }
        other => {
            return Err(AotError(format!(
                "runner result wrapping does not yet support {:?}",
                other
            )))
        }
    };
    Ok(value)
}

pub fn wrap_arg_as_value(
    module: &CompiledModule,
    name: &str,
    type_id: TypeId,
) -> Result<String, AotError> {
    let value = match module.types.get(type_id) {
        KiraType::Bool => format!("Value::Bool({name})"),
        KiraType::Int => format!("Value::Int({name})"),
        KiraType::Float => format!("Value::Float(ordered_float::OrderedFloat({name}))"),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            format!("unsafe {{ *Box::from_raw({name} as *mut Value) }}")
        }
        other => {
            return Err(AotError(format!(
                "runner bridge arguments do not yet support {:?}",
                other
            )))
        }
    };
    Ok(value)
}

pub fn unwrap_value_result(
    module: &CompiledModule,
    name: &str,
    type_id: TypeId,
) -> Result<String, AotError> {
    let value = match module.types.get(type_id) {
        KiraType::Bool => format!(
            "match {name} {{ Value::Bool(value) => value, other => panic!(\"expected bool return value, got {{:?}}\", other) }}"
        ),
        KiraType::Int => format!(
            "match {name} {{ Value::Int(value) => value, other => panic!(\"expected int return value, got {{:?}}\", other) }}"
        ),
        KiraType::Float => format!(
            "match {name} {{ Value::Float(value) => value.0, other => panic!(\"expected float return value, got {{:?}}\", other) }}"
        ),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            format!("Box::into_raw(Box::new({name})) as *mut c_void")
        }
        other => return Err(AotError(format!("runner bridge return does not yet support {:?}", other))),
    };
    Ok(value)
}
