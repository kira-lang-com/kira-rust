// Bridge function generation

use crate::aot::bridge::BridgeSpec;
use crate::aot::error::AotError;
use crate::aot::runner::c_abi::{c_param_list, c_return_type};
use crate::compiler::CompiledModule;
use crate::runtime::type_system::KiraType;

pub fn generate_bridge_function(module: &CompiledModule, bridge: &BridgeSpec) -> Result<String, AotError> {
    let mut out = String::new();
    let params = c_param_list(module, &bridge.signature, true)?;
    let ret = c_return_type(module, &bridge.signature)?;

    out.push_str(&format!("{ret} {symbol}({params}) {{\n", ret = ret, symbol = bridge.symbol, params = params));
    out.push_str("    const char* callee = \"");
    out.push_str(&bridge.callee);
    out.push_str("\";\n");
    out.push_str("    NativeRuntimeContext* ctx_ptr = (NativeRuntimeContext*)ctx;\n");
    out.push_str("    KiraVm* vm = ctx_ptr->vm;\n");
    out.push_str("    const KiraModule* module = ctx_ptr->module;\n");

    let argc = bridge.signature.params.len();
    if argc == 0 {
        out.push_str("    KiraValue** args = NULL;\n");
    } else {
        out.push_str(&format!("    KiraValue* args[{argc}];\n", argc = argc));
        for (index, type_id) in bridge.signature.params.iter().enumerate() {
            out.push_str(&emit_bridge_arg(module, index, *type_id)?);
        }
    }

    out.push_str("    KiraValue* result = NULL;\n");
    out.push_str("    KiraError err = {0};\n");
    out.push_str(&format!(
        "    if (!kira_vm_run_function(vm, module, callee, {args_ptr}, {argc}, &result, &err)) {{\n",
        args_ptr = if argc == 0 { "NULL" } else { "args" },
        argc = argc
    ));
    out.push_str("        abort_with_error(\"bridge call failed\", &err);\n");
    out.push_str("    }\n");

    match module.types.get(bridge.signature.return_type) {
        KiraType::Unit => {
            out.push_str("    if (result) { kira_value_free(result); }\n");
            out.push_str("    return;\n");
        }
        KiraType::Bool => {
            out.push_str("    KiraError ret_err = {0};\n");
            out.push_str("    bool value = kira_value_as_bool(result, &ret_err);\n");
            out.push_str("    if (kira_error_has(&ret_err)) { abort_with_error(\"bridge return\", &ret_err); }\n");
            out.push_str("    kira_value_free(result);\n");
            out.push_str("    return value;\n");
        }
        KiraType::Int => {
            out.push_str("    KiraError ret_err = {0};\n");
            out.push_str("    int64_t value = kira_value_as_int(result, &ret_err);\n");
            out.push_str("    if (kira_error_has(&ret_err)) { abort_with_error(\"bridge return\", &ret_err); }\n");
            out.push_str("    kira_value_free(result);\n");
            out.push_str("    return value;\n");
        }
        KiraType::Float => {
            out.push_str("    KiraError ret_err = {0};\n");
            out.push_str("    double value = kira_value_as_float(result, &ret_err);\n");
            out.push_str("    if (kira_error_has(&ret_err)) { abort_with_error(\"bridge return\", &ret_err); }\n");
            out.push_str("    kira_value_free(result);\n");
            out.push_str("    return value;\n");
        }
        _ => {
            out.push_str("    void* handle = kira_value_into_handle(result);\n");
            out.push_str("    return handle;\n");
        }
    }

    out.push_str("}\n\n");
    Ok(out)
}

fn emit_bridge_arg(
    module: &CompiledModule,
    index: usize,
    type_id: crate::runtime::type_system::TypeId,
) -> Result<String, AotError> {
    let mut out = String::new();
    match module.types.get(type_id) {
        KiraType::Bool => {
            out.push_str(&format!(
                "    args[{index}] = kira_value_from_bool(arg{index});\n",
                index = index
            ));
        }
        KiraType::Int => {
            out.push_str(&format!(
                "    args[{index}] = kira_value_from_int(arg{index});\n",
                index = index
            ));
        }
        KiraType::Float => {
            out.push_str(&format!(
                "    args[{index}] = kira_value_from_float(arg{index});\n",
                index = index
            ));
        }
        KiraType::String
        | KiraType::Array(_)
        | KiraType::Struct(_)
        | KiraType::Dynamic
        | KiraType::Opaque(_) => {
            out.push_str(&format!(
                "    args[{index}] = kira_value_from_handle_take(arg{index});\n",
                index = index
            ));
        }
        KiraType::Unit => {
            return Err(AotError("unit type cannot appear in bridge params".to_string()));
        }
        other => {
            return Err(AotError(format!("unsupported bridge param type {:?}", other)));
        }
    }
    Ok(out)
}
