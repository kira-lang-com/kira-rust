// Main codegen implementation module
// Coordinates LLVM code generation for Kira functions

mod calls;
mod context;
mod control_flow;
mod emit_dispatch;
mod emit_function;
mod instructions;
mod runtime_arrays;
mod runtime_base;
mod runtime_boxing;
mod runtime_print;
mod runtime_structs;
mod types;
mod value_arrays;
mod value_boxing;
mod value_constants;
mod value_structs;

pub use context::NativeCodegen;
