use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::compiler::{BackendKind, Chunk, CompiledModule};
use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

use super::super::builtins::call_builtin;
use super::execution::execute_chunk;

pub type NativeHandler = fn(&mut Vm, &CompiledModule, Vec<Value>) -> Result<Value, RuntimeError>;

pub struct Vm {
    pub(crate) output: Vec<String>,
    pub(crate) rng_state: u64,
    pub(crate) started_at: Instant,
    native_registry: HashMap<String, NativeHandler>,
}

impl Default for Vm {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x5eed_u64);

        Self {
            output: Vec::new(),
            rng_state: seed ^ 0x9E37_79B9_7F4A_7C15,
            started_at: Instant::now(),
            native_registry: HashMap::new(),
        }
    }
}

impl Vm {
    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn register_native(&mut self, name: impl Into<String>, handler: NativeHandler) {
        self.native_registry.insert(name.into(), handler);
    }

    pub fn run_entry(
        &mut self,
        module: &CompiledModule,
        entry: &str,
    ) -> Result<Value, RuntimeError> {
        self.run_function(module, entry, Vec::new())
    }

    pub fn run_function(
        &mut self,
        module: &CompiledModule,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if let Some(builtin) = module.builtins.get(name) {
            if builtin.signature.params.len() != args.len() {
                return Err(RuntimeError(format!(
                    "`{name}` expects {} arguments but got {}",
                    builtin.signature.params.len(),
                    args.len()
                )));
            }
            return call_builtin(self, name, args);
        }

        let function = module
            .functions
            .get(name)
            .ok_or_else(|| RuntimeError(format!("unknown function `{name}`")))?;

        if function.signature.params.len() != args.len() {
            return Err(RuntimeError(format!(
                "function `{name}` expects {} arguments but got {}",
                function.signature.params.len(),
                args.len()
            )));
        }

        if let Some(chunk) = function.artifacts.bytecode.as_ref() {
            if function.selected_backend == BackendKind::Native {
                if let Some(handler) = self.native_registry.get(name).copied() {
                    return handler(self, module, args);
                }
            }
            return execute_chunk(self, module, chunk, args);
        }

        match function.selected_backend {
            BackendKind::Native => {
                if let Some(handler) = self.native_registry.get(name).copied() {
                    return handler(self, module, args);
                }
                let artifact = function.artifacts.aot.as_ref().ok_or_else(|| {
                    RuntimeError(format!("function `{name}` has no AOT artifact"))
                })?;
                Err(RuntimeError(format!(
                    "function `{name}` is build-time AOT only and has no VM shadow (symbol `{}`)",
                    artifact.symbol
                )))
            }
            BackendKind::Vm => Err(RuntimeError(format!(
                "function `{name}` is missing bytecode"
            ))),
        }
    }
}
