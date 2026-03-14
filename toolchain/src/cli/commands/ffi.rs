// FFI bindings CLI command

use std::process;

use crate::cli::utils::find_project_root;
use crate::project::generate_ffi_bindings;

pub fn cmd_ffi() {
    let project_root = find_project_root();
    if let Err(error) = generate_ffi_bindings(&project_root) {
        eprintln!("error: {}", error);
        process::exit(1);
    }
    println!("  ✓ Generated FFI bindings");
}
