use std::fs;
use std::path::Path;
use std::process;

use crate::compiler::CompiledModule;
use crate::runtime::vm::Vm;
use crate::runtime::ffi_loader::FfiLoader;

pub fn cmd_run_module(module_path: &Path) {
    // Read and deserialize the module
    let module_bytes = match fs::read(module_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error: failed to read module `{}`: {}", module_path.display(), e);
            process::exit(1);
        }
    };

    let module: CompiledModule = match bincode::deserialize(&module_bytes) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: failed to deserialize module: {}", e);
            process::exit(1);
        }
    };

    // Find the entry point (main function)
    let entry = module.functions.keys()
        .find(|name| name.as_str() == "main")
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("error: no main function found in module");
            process::exit(1);
        });

    let mut vm = Vm::default();

    // Load FFI libraries if present
    if !module.ffi.functions.is_empty() || !module.ffi.links.is_empty() {
        let project_root = module_path.parent().unwrap_or_else(|| Path::new("."));
        let mut ffi_loader = FfiLoader::new();
        if let Err(e) = ffi_loader.load_ffi_metadata(&module.ffi, project_root) {
            eprintln!("error: failed to load FFI libraries: {}", e);
            process::exit(1);
        }
        vm.load_ffi(ffi_loader);
    }

    // Run the module
    match vm.run_entry(&module, &entry) {
        Ok(_) => {
            for line in vm.output() {
                println!("{}", line);
            }
        }
        Err(error) => {
            eprintln!("Runtime Error:\n{}", error);
            process::exit(1);
        }
    }
}
