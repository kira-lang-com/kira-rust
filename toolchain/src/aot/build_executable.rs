use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::compile_project;
use crate::project::load_project;

use super::error::AotError;
use super::runner::{build_c_runner_executable, resolve_output_root};
use crate::compiler::serialize_module;

pub fn build_default_project(project_root: &Path, out_root: &Path) -> Result<PathBuf, AotError> {
    use crate::compiler::build_all_native_dependencies;
    use super::library::build_native_archive;
    
    let project = load_project(project_root).map_err(|error| AotError(error.to_string()))?;
    
    // Build native dependencies
    let native_deps: Vec<(String, PathBuf)> = project.program.links.iter()
        .map(|link| (link.library.clone(), PathBuf::from(&link.header)))
        .collect();
    
    if !native_deps.is_empty() {
        build_all_native_dependencies(project_root, &native_deps)
            .map_err(|e| AotError(format!("Failed to build native dependencies: {}", e)))?;
    }
    
    let module = compile_project(&project.program, project_root)
        .map_err(|error| AotError(error.to_string()))?;

    let out_root = resolve_output_root(out_root)?;
    fs::create_dir_all(&out_root).map_err(|error| {
        AotError(format!(
            "failed to create output directory `{}`: {}",
            out_root.display(),
            error
        ))
    })?;

    // Serialize the compiled module to target/compiled_module.bin
    let module_path = out_root.join("compiled_module.bin");
    fs::write(
        &module_path,
        serialize_module(&module).map_err(AotError)?,
    )
    .map_err(|error| {
        AotError(format!(
            "failed to write `{}`: {}",
            module_path.display(),
            error
        ))
    })?;

    // Create debug directory for the executable
    let debug_dir = out_root.join("debug");
    fs::create_dir_all(&debug_dir).map_err(|error| {
        AotError(format!(
            "failed to create debug directory `{}`: {}",
            debug_dir.display(),
            error
        ))
    })?;

    // Build native archive for AOT-compiled functions
    let staging_dir = out_root.join(".kira-build").join(&project.manifest.name).join("staging");
    fs::create_dir_all(&staging_dir).map_err(|error| {
        AotError(format!(
            "failed to create staging directory `{}`: {}",
            staging_dir.display(),
            error
        ))
    })?;
    let native_archive = build_native_archive(&project.manifest.name, &module, &staging_dir)?;

    let final_binary = debug_dir.join(exe_name(&project.manifest.name));

    // Generate and build the standalone runner (C + clang, no Cargo)
    let runner_dir = out_root.join(".kira-build").join(&project.manifest.name).join("runner");
    build_c_runner_executable(
        &runner_dir,
        &project.manifest.name,
        &module,
        &module_path,
        &native_archive,
        &project.entry_symbol,
        &final_binary,
    )?;

    // Copy the compiled module to the same directory as the executable
    let final_module = debug_dir.join("compiled_module.bin");
    fs::copy(&module_path, &final_module).map_err(|error| {
        AotError(format!(
            "failed to copy compiled module from `{}` to `{}`: {}",
            module_path.display(),
            final_module.display(),
            error
        ))
    })?;

    Ok(final_binary)
}

fn exe_name(base: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

pub fn run_default_project(project_root: &Path, _out_root: &Path) -> Result<i32, AotError> {
    use crate::runtime::vm::Vm;
    use crate::runtime::ffi_loader::FfiLoader;
    use crate::compiler::build_all_native_dependencies;
    
    let project = load_project(project_root).map_err(|error| AotError(error.to_string()))?;
    
    // Build native dependencies
    let native_deps: Vec<(String, PathBuf)> = project.program.links.iter()
        .map(|link| (link.library.clone(), PathBuf::from(&link.header)))
        .collect();
    
    if !native_deps.is_empty() {
        build_all_native_dependencies(project_root, &native_deps)
            .map_err(|e| AotError(format!("Failed to build native dependencies: {}", e)))?;
    }
    
    let module = compile_project(&project.program, project_root)
        .map_err(|error| AotError(error.to_string()))?;
    
    let mut vm = Vm::default();
    
    // Load FFI libraries if present
    if !module.ffi.functions.is_empty() || !module.ffi.links.is_empty() {
        let mut ffi_loader = FfiLoader::new();
        ffi_loader.load_ffi_metadata(&module.ffi, project_root)
            .map_err(|e| AotError(format!("Failed to load FFI libraries: {}", e)))?;
        vm.load_ffi(ffi_loader);
    }
    
    match vm.run_entry(&module, &project.entry_symbol) {
        Ok(_) => {
            for line in vm.output() {
                println!("{}", line);
            }
            Ok(0)
        }
        Err(error) => {
            eprintln!("Runtime Error:\n{}", error);
            Ok(1)
        }
    }
}
