// Native code generation for Kira functions

mod calls;
mod context;
mod control_flow;
mod emit_dispatch;
mod emit_function;
mod instructions;
mod runtime;
mod types;
mod values;

pub use context::NativeCodegen;
