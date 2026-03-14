use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::compile_project;
use crate::project::load_project;

use super::error::AotError;
use super::runner::{remove_path_if_exists, resolve_output_root};

pub fn build_default_project(project_root: &Path, out_root: &Path) -> Result<PathBuf, AotError> {
    use std::env;
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

    // Find the kira binary
    let kira_binary = env::current_exe()
        .map_err(|e| AotError(format!("Failed to get current executable path: {}", e)))?;
    
    // Create executable at target/debug/projectname
    let binary_name = project.manifest.name.clone();
    let final_binary = debug_dir.join(&binary_name);
    
    create_wrapper_script(&final_binary, &kira_binary, &module_path)?;

    Ok(final_binary)
}

fn create_wrapper_script(
    wrapper_path: &Path,
    kira_binary: &Path,
    module_path: &Path,
) -> Result<(), AotError> {
    #[cfg(unix)]
    {
        // Create a shell script that runs the compiled module via the Kira VM
        let script = format!(
            "#!/bin/sh\nexec \"{}\" run-module \"{}\" \"$@\"\n",
            kira_binary.display(),
            module_path.display()
        );
        fs::write(wrapper_path, script).map_err(|e| {
            AotError(format!(
                "failed to write executable `{}`: {}",
                wrapper_path.display(),
                e
            ))
        })?;
        
        // Make it executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(wrapper_path)
            .map_err(|e| AotError(format!("failed to get permissions: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(wrapper_path, perms)
            .map_err(|e| AotError(format!("failed to set permissions: {}", e)))?;
    }
    
    #[cfg(windows)]
    {
        // Create a batch file that runs the compiled module via the Kira VM
        let script = format!(
            "@echo off\r\n\"{}\" run-module \"{}\" %*\r\n",
            kira_binary.display(),
            module_path.display()
        );
        let bat_path = wrapper_path.with_extension("bat");
        fs::write(&bat_path, script).map_err(|e| {
            AotError(format!(
                "failed to write executable `{}`: {}",
                bat_path.display(),
                e
            ))
        })?;
    }
    
    Ok(())
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
