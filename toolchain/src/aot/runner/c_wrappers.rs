// Native wrapper generation

use crate::aot::error::AotError;
use crate::aot::runner::mangle_ident;
use crate::compiler::{CompiledModule, FunctionSignature};
use crate::runtime::type_system::KiraType;

pub fn generate_native_wrapper(
    module: &CompiledModule,
    function_name: &str,
    symbol: &str,
    signature: &FunctionSignature,
) -> Result<String, AotError> {
    let wrapper_name = format!("wrap_{}", mangle_ident(function_name));
    let argc = signature.params.len();

    let mut body = String::new();
    body.push_str(&format!(
        "static KiraValue* {wrapper}(KiraVm* vm, const KiraModule* module, const KiraValue* const* args, size_t argc, KiraError* err) {{\n",
        wrapper = wrapper_name
    ));
    body.push_str(&format!(
        "    if (argc != {argc}) {{ kira_error_set(err, \"native function `{name}` expects {argc} arguments\"); return NULL; }}\n",
        argc = argc,
        name = function_name
    ));
    for (index, type_id) in signature.params.iter().enumerate() {
        body.push_str(&emit_value_extract(module, index, *type_id)?);
    }

    body.push_str("    NativeRuntimeContext ctx = { vm, module };\n");

    let call_args = if argc == 0 {
        "&ctx".to_string()
    } else {
        let args = (0..argc).map(|i| format!("arg{i}")).collect::<Vec<_>>().join(", ");
        format!("&ctx, {args}")
    };

    let ret_type = module.types.get(signature.return_type);
    match ret_type {
        KiraType::Unit => {
            body.push_str(&format!("    {symbol}({call_args});\n", symbol = symbol, call_args = call_args));
            body.push_str("    return kira_value_unit();\n");
        }
        KiraType::Bool => {
            body.push_str(&format!("    bool result = {symbol}({call_args});\n", symbol = symbol, call_args = call_args));
            body.push_str("    return kira_value_from_bool(result);\n");
        }
        KiraType::Int => {
            body.push_str(&format!("    int64_t result = {symbol}({call_args});\n", symbol = symbol, call_args = call_args));
            body.push_str("    return kira_value_from_int(result);\n");
        }
        KiraType::Float => {
            body.push_str(&format!("    double result = {symbol}({call_args});\n", symbol = symbol, call_args = call_args));
            body.push_str("    return kira_value_from_float(result);\n");
        }
        _ => {
            body.push_str(&format!("    void* result = {symbol}({call_args});\n", symbol = symbol, call_args = call_args));
            body.push_str("    return kira_value_from_handle_take(result);\n");
        }
    }
    body.push_str("}\n\n");
    Ok(body)
}

fn emit_value_extract(
    module: &CompiledModule,
    index: usize,
    type_id: crate::runtime::type_system::TypeId,
) -> Result<String, AotError> {
    let mut out = String::new();
    match module.types.get(type_id) {
        KiraType::Bool => {
            out.push_str(&format!(
                "    bool arg{index} = kira_value_as_bool(args[{index}], err);\n    if (kira_error_has(err)) {{ return NULL; }}\n",
                index = index
            ));
        }
        KiraType::Int => {
            out.push_str(&format!(
                "    int64_t arg{index} = kira_value_as_int(args[{index}], err);\n    if (kira_error_has(err)) {{ return NULL; }}\n",
                index = index
            ));
        }
        KiraType::Float => {
            out.push_str(&format!(
                "    double arg{index} = kira_value_as_float(args[{index}], err);\n    if (kira_error_has(err)) {{ return NULL; }}\n",
                index = index
            ));
        }
        KiraType::String
        | KiraType::Array(_)
        | KiraType::Struct(_)
        | KiraType::Dynamic
        | KiraType::Opaque(_) => {
            out.push_str(&format!(
                "    if (!args[{index}]) {{ kira_error_set(err, \"expected handle argument {index}\"); return NULL; }}\n    void* arg{index} = kira_value_into_handle_clone(args[{index}]);\n",
                index = index
            ));
        }
        KiraType::Unit => {
            return Err(AotError("unit type cannot appear in function parameters".to_string()));
        }
        other => {
            return Err(AotError(format!("unsupported wrapper parameter type {:?}", other)));
        }
    }
    Ok(out)
}
