// C ABI mapping helpers

use crate::aot::error::AotError;
use crate::compiler::{CompiledModule, FunctionSignature};
use crate::runtime::type_system::KiraType;

pub fn c_return_type(
    module: &CompiledModule,
    signature: &FunctionSignature,
) -> Result<&'static str, AotError> {
    match module.types.get(signature.return_type) {
        KiraType::Unit => Ok("void"),
        _ => c_abi_type_name(module, signature.return_type),
    }
}

pub fn c_param_list(
    module: &CompiledModule,
    signature: &FunctionSignature,
    include_ctx: bool,
) -> Result<String, AotError> {
    let mut params = Vec::new();
    if include_ctx {
        params.push("void* ctx".to_string());
    }
    for (index, type_id) in signature.params.iter().enumerate() {
        let name = c_abi_type_name(module, *type_id)?;
        params.push(format!("{name} arg{index}"));
    }
    Ok(if params.is_empty() {
        "void".to_string()
    } else {
        params.join(", ")
    })
}

pub fn c_abi_type_name(
    module: &CompiledModule,
    type_id: crate::runtime::type_system::TypeId,
) -> Result<&'static str, AotError> {
    match module.types.get(type_id) {
        KiraType::Bool => Ok("bool"),
        KiraType::Int => Ok("int64_t"),
        KiraType::Float => Ok("double"),
        KiraType::String
        | KiraType::Array(_)
        | KiraType::Struct(_)
        | KiraType::Dynamic
        | KiraType::Opaque(_) => Ok("void*"),
        KiraType::Unit => Err(AotError(
            "unit type cannot appear in C ABI parameters".to_string(),
        )),
        other => Err(AotError(format!("unsupported C ABI type {:?}", other))),
    }
}
