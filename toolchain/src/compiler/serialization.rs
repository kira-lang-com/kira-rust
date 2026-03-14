// Compiled module serialization helpers

use crate::compiler::CompiledModule;

pub fn serialize_module(module: &CompiledModule) -> Result<Vec<u8>, String> {
    bincode::serialize(module).map_err(|error| format!("module serialization failed: {error}"))
}

pub fn deserialize_module(bytes: &[u8]) -> Result<CompiledModule, String> {
    bincode::deserialize(bytes).map_err(|error| format!("module deserialization failed: {error}"))
}
