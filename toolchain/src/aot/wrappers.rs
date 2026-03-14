use crate::compiler::{CompiledFunction, CompiledModule};
use crate::runtime::type_system::{KiraType, TypeId};

use super::error::AotError;
use super::types::{rust_abi_type_name, wrap_arg_as_value, wrap_rust_result};
use super::utils::{indent, mangle_ident};

pub fn generate_extern_decl(
    module: &CompiledModule,
    function: &CompiledFunction,
) -> Result<String, AotError> {
    let params = function
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| {
            rust_abi_type_name(module, *type_id).map(|name| format!("arg{index}: {name}"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    let params = if params.is_empty() {
        "ctx: *mut c_void".to_string()
    } else {
        format!("ctx: *mut c_void, {params}")
    };

    let return_type = rust_abi_type_name(module, function.signature.return_type)?;
    let ret = if return_type == "()" {
        "".to_string()
    } else {
        format!(" -> {return_type}")
    };

    Ok(format!(
        "unsafe extern \"C\" {{\n    fn {}({}){};\n}}\n\n",
        function
            .artifacts
            .aot
            .as_ref()
            .map(|artifact| artifact.symbol.as_str())
            .unwrap_or("missing_symbol"),
        params,
        ret
    ))
}

pub fn generate_native_wrapper(
    module: &CompiledModule,
    function: &CompiledFunction,
) -> Result<String, AotError> {
    let wrapper_name = format!("wrap_{}", mangle_ident(&function.name));
    let arg_extracts = function
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| generate_value_extract(module, index, *type_id))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    let native_args = (0..function.signature.params.len())
        .map(|index| format!("arg{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let call_args = if native_args.is_empty() {
        "&mut ctx as *mut _ as *mut c_void".to_string()
    } else {
        format!("&mut ctx as *mut _ as *mut c_void, {native_args}")
    };
    let call = if rust_abi_type_name(module, function.signature.return_type)? == "()" {
        format!(
            "    unsafe {{ {}({}); }}\n    Ok(Value::Unit)",
            function.artifacts.aot.as_ref().unwrap().symbol,
            call_args
        )
    } else {
        format!(
            "    let result = unsafe {{ {}({}) }};\n    Ok({})",
            function.artifacts.aot.as_ref().unwrap().symbol,
            call_args,
            wrap_rust_result(module, "result", function.signature.return_type)?
        )
    };

    Ok(format!(
        "fn {wrapper_name}(vm: &mut Vm, module: &CompiledModule, args: Vec<Value>) -> Result<Value, RuntimeError> {{\n    if args.len() != {argc} {{\n        return Err(RuntimeError(format!(\"native function `{name}` expects {argc} arguments but got {{}}\", args.len())));\n    }}\n{arg_extracts}\n    let mut ctx = NativeRuntimeContext {{ vm, module }};\n{call}\n}}\n\n",
        wrapper_name = wrapper_name,
        argc = function.signature.params.len(),
        name = function.name,
        arg_extracts = indent(&arg_extracts, 4),
        call = indent(&call, 4),
    ))
}

fn generate_value_extract(
    module: &CompiledModule,
    index: usize,
    type_id: TypeId,
) -> Result<String, AotError> {
    let source = match module.types.get(type_id) {
        KiraType::Bool => format!(
            "let arg{index} = match &args[{index}] {{\n    Value::Bool(value) => *value,\n    other => return Err(RuntimeError(format!(\"expected bool argument {index}, got {{:?}}\", other))),\n}};"
        ),
        KiraType::Int => format!(
            "let arg{index} = match &args[{index}] {{\n    Value::Int(value) => *value,\n    other => return Err(RuntimeError(format!(\"expected int argument {index}, got {{:?}}\", other))),\n}};"
        ),
        KiraType::Float => format!(
            "let arg{index} = match &args[{index}] {{\n    Value::Float(value) => value.0,\n    other => return Err(RuntimeError(format!(\"expected float argument {index}, got {{:?}}\", other))),\n}};"
        ),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => format!(
            "let arg{index} = Box::into_raw(Box::new(args[{index}].clone())) as *mut c_void;"
        ),
        other => {
            return Err(AotError(format!(
                "runner argument extraction does not yet support {:?}",
                other
            )))
        }
    };
    Ok(source)
}
