// Runner project generation and utilities

mod dylib;
mod c_runner;
mod c_abi;
mod c_bridges;
mod c_source;
mod c_wrappers;
mod utils;

pub use dylib::{link_shared_library, shared_lib_extension};
pub use c_runner::build_c_runner_executable;
pub use utils::{mangle_ident, resolve_output_root, write_if_changed};
