// Main codegen implementation module
// Coordinates LLVM code generation for Kira functions

mod calls;
mod context;
mod control_flow;
mod emit;
mod instructions;
mod runtime_arrays;
mod runtime_base;
mod runtime_boxing;
mod runtime_print;
mod runtime_structs;
mod types;
mod values;

pub use context::NativeCodegen;
