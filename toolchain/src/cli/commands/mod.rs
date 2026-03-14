// CLI command implementations

mod deps;
mod ffi;
mod project;
mod run_module;
mod toolchain;

pub use deps::{cmd_add, cmd_fetch};
pub use ffi::cmd_ffi;
pub use project::{cmd_build, cmd_check, cmd_clean, cmd_new, cmd_package, cmd_run};
pub use run_module::cmd_run_module;
pub use toolchain::{cmd_install, cmd_toolchain_install, cmd_toolchain_list, cmd_toolchain_path, cmd_toolchain_use};
