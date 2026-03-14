use crate::compiler::{BackendKind, CompiledModule, FunctionSignature, Instruction};

use crate::aot::error::AotError;
use crate::aot::runner::mangle_ident;

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
