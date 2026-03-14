// Main codegen implementation module
// Coordinates LLVM code generation for Kira functions

mod calls;
mod context;
mod control_flow;
mod emit;
mod instructions;
mod runtime;
mod types;
mod values;

pub use context::NativeCodegen;
