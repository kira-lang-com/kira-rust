mod archive;
mod bridge;
mod build;
mod c_header;
mod codegen;
mod dylib;
mod error;
mod lib_codegen;
mod runner;
mod stack;
mod types;
mod utils;
mod wrappers;

pub use build::{build_default_project, build_library_project, run_default_project};
pub use error::AotError;
