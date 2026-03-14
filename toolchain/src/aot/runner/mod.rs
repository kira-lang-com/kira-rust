// Runner project generation and utilities

mod dylib;
mod runner;
mod utils;

pub use dylib::{link_shared_library, shared_lib_extension};
pub use runner::{build_runner_project, get_shared_target_dir, write_runner_project};
pub use utils::{indent, mangle_ident, remove_path_if_exists, resolve_output_root, write_if_changed};
