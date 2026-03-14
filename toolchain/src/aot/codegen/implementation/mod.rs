// Main codegen implementation module
// Coordinates LLVM code generation for Kira functions

mod context;
mod emit;
mod helpers;
mod runtime;
mod types;
mod values;

pub use context::NativeCodegen;
