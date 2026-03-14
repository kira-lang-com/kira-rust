use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn build_native_library(header_path: &Path) -> Result<Option<PathBuf>, String> {
    // Check if there's a C source file next to the header
    let header_dir = header_path.parent().ok_or("Invalid header path")?;
    let header_stem = header_path.file_stem().ok_or("Invalid header filename")?;
    
    let c_source = header_dir.join(format!("{}.c", header_stem.to_string_lossy()));
    
    if !c_source.exists() {
        // No C source file, assume library is pre-built
        return Ok(None);
    }
    
    // Determine output library name based on platform
    let lib_name = if cfg!(target_os = "macos") {
        format!("lib{}.dylib", header_stem.to_string_lossy())
    } else if cfg!(target_os = "windows") {
        format!("{}.dll", header_stem.to_string_lossy())
    } else {
        format!("lib{}.so", header_stem.to_string_lossy())
    };
    
    let output_path = header_dir.join(&lib_name);
    
    // Check if library is up to date
    if output_path.exists() {
        let source_modified = fs::metadata(&c_source)
            .and_then(|m| m.modified())
            .ok();
        let lib_modified = fs::metadata(&output_path)
            .and_then(|m| m.modified())
            .ok();
        
        if let (Some(src_time), Some(lib_time)) = (source_modified, lib_modified) {
            if lib_time > src_time {
                // Library is up to date
                return Ok(Some(output_path));
            }
        }
    }
    
    println!("  Building native library from {}", c_source.display());
    
    // Compile the C source
    let mut cmd = Command::new("clang");
    
    if cfg!(target_os = "macos") {
        cmd.arg("-dynamiclib")
            .arg("-install_name")
            .arg(format!("@rpath/{}", lib_name));
    } else if cfg!(target_os = "windows") {
        cmd.arg("-shared");
    } else {
        cmd.arg("-shared")
            .arg("-fPIC");
    }
    
    cmd.arg("-o")
        .arg(&output_path)
        .arg(&c_source);
    
    let status = cmd.status()
        .map_err(|e| format!("Failed to run clang: {}. Make sure clang is installed.", e))?;
    
    if !status.success() {
        return Err(format!("Failed to compile {}", c_source.display()));
    }
    
    println!("  Built {} in {}", lib_name, header_dir.display());
    
    Ok(Some(output_path))
}

pub fn build_all_native_dependencies(project_root: &Path, dependencies: &[(String, PathBuf)]) -> Result<(), String> {
    for (name, header_path) in dependencies {
        let full_path = if header_path.is_absolute() {
            header_path.clone()
        } else {
            project_root.join(header_path)
        };
        
        if let Some(_lib_path) = build_native_library(&full_path)? {
            // Library was built or is up to date
        }
    }
    
    Ok(())
}
