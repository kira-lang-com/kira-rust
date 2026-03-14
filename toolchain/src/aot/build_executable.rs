use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::compile_project;
use crate::project::load_project;

use super::error::AotError;
use super::runner::{build_runner_project, resolve_output_root, write_runner_project, get_shared_target_dir};

pub fn build_default_project(project_root: &Path, out_root: &Path) -> Result<PathBuf, AotError> {
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
        bincode::serialize(&module)
            .map_err(|error| AotError(format!("module serialization failed: {error}")))?,
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

    // Generate runner project (Rust code that embeds VM and bytecode)
    let runner_dir = out_root.join(".kira-build").join(&project.manifest.name).join("runner");
    
    // Note: native_archive is not used for bytecode-only builds, pass a dummy path
    let dummy_archive = runner_dir.join("dummy.a");
    write_runner_project(
        &runner_dir,
        &project.manifest.name,
        &module,
        &module_path,
        &dummy_archive,
        &project.entry_symbol,
    )?;

    // Build the runner project with Cargo
    build_runner_project(&runner_dir)?;

    // Copy the compiled binary to target/debug/projectname
    let shared_target = get_shared_target_dir()?;
    let runner_binary = shared_target.join("release").join(&project.manifest.name);
    let final_binary = debug_dir.join(&project.manifest.name);
    
    fs::copy(&runner_binary, &final_binary).map_err(|error| {
        AotError(format!(
            "failed to copy runner binary from `{}` to `{}`: {}",
            runner_binary.display(),
            final_binary.display(),
            error
        ))
    })?;

    Ok(final_binary)
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
