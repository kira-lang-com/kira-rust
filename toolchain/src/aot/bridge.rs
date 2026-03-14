use crate::compiler::{BackendKind, CompiledModule, FunctionSignature, Instruction};

use super::error::AotError;
use super::types::{rust_abi_type_name, unwrap_value_result, wrap_arg_as_value};
use super::utils::mangle_ident;

#[derive(Clone)]
pub struct BridgeSpec {
    pub callee: String,
    pub symbol: String,
    pub signature: FunctionSignature,
}

pub fn collect_runtime_bridges(module: &CompiledModule) -> Result<Vec<BridgeSpec>, AotError> {
    let mut bridges = Vec::new();

    for function in module.functions.values() {
        if function.selected_backend != BackendKind::Native {
            continue;
        }
        let Some(chunk) = function.artifacts.bytecode.as_ref() else {
            continue;
        };
        for instruction in &chunk.instructions {
            let Instruction::Call {
                function: callee, ..
            } = instruction
            else {
                continue;
            };
            if module.ffi.functions.contains_key(callee) {
                continue;
            }
            if matches!(
                module.functions.get(callee),
                Some(target) if target.selected_backend == BackendKind::Native
            ) {
                continue;
            }
            let signature = module
                .builtins
                .get(callee)
                .map(|builtin| builtin.signature.clone())
                .or_else(|| {
                    module
                        .functions
                        .get(callee)
                        .map(|function| function.signature.clone())
                })
                .ok_or_else(|| {
                    AotError(format!("missing signature for bridge callee `{callee}`"))
                })?;
            let symbol = format!("kira_bridge_{}", mangle_ident(callee));
            if bridges
                .iter()
                .any(|bridge: &BridgeSpec| bridge.symbol == symbol)
            {
                continue;
            }
            bridges.push(BridgeSpec {
                callee: callee.clone(),
                symbol,
                signature,
            });
        }
    }

    Ok(bridges)
}

pub fn generate_bridge_function(
    module: &CompiledModule,
    bridge: &BridgeSpec,
) -> Result<String, AotError> {
    let args = bridge
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| {
            rust_abi_type_name(module, *type_id).map(|name| format!("arg{index}: {name}"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    let params = if args.is_empty() {
        "ctx: *mut c_void".to_string()
    } else {
        format!("ctx: *mut c_void, {args}")
    };
    let ret = rust_abi_type_name(module, bridge.signature.return_type)?;
    let ret_decl = if ret == "()" {
        "".to_string()
    } else {
        format!(" -> {ret}")
    };

    let arg_values = bridge
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| wrap_arg_as_value(module, &format!("arg{index}"), *type_id))
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");

    let unwrap = if ret == "()" {
        "    match vm.run_function(module, CALLEE, args) {\n        Ok(Value::Unit) | Ok(_) => {}\n        Err(error) => panic!(\"bridge call failed: {}\", error),\n    }\n".to_string()
    } else {
        format!(
            "    match vm.run_function(module, CALLEE, args) {{\n        Ok(result) => {},\n        Err(error) => panic!(\"bridge call failed: {{}}\", error),\n    }}\n",
            unwrap_value_result(module, "result", bridge.signature.return_type)?
        )
    };

    Ok(format!(
        "#[no_mangle]\npub extern \"C\" fn {symbol}({params}){ret_decl} {{\n    const CALLEE: &str = \"{callee}\";\n    let ctx = unsafe {{ &mut *(ctx as *mut NativeRuntimeContext) }};\n    let vm = unsafe {{ &mut *ctx.vm }};\n    let module = unsafe {{ &*ctx.module }};\n    let args = vec![{arg_values}];\n{unwrap}}}\n\n",
        symbol = bridge.symbol,
        params = params,
        ret_decl = ret_decl,
        callee = bridge.callee,
        arg_values = arg_values,
        unwrap = unwrap
    ))
}
