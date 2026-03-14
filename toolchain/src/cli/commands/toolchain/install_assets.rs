// Install-time asset copying helpers

use std::fs;
use std::path::Path;

pub fn copy_toolchain_sources(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.join("Cargo.toml").is_file() {
        return Err(format!(
            "toolchain source directory `{}` does not look like a Rust crate",
            src.display()
        ));
    }
    fs::create_dir_all(dst).map_err(|e| format!("failed to create `{}`: {}", dst.display(), e))?;
    copy_dir_recursive_filtered(src, dst)
}

pub fn copy_runtime_staticlib(src: &Path, dst_dir: &Path) -> Result<(), String> {
    if !src.is_file() {
        return Err(format!(
            "runtime static library `{}` does not exist",
            src.display()
        ));
    }
    fs::create_dir_all(dst_dir)
        .map_err(|e| format!("failed to create `{}`: {}", dst_dir.display(), e))?;
    let dest_path = dst_dir.join(runtime_lib_name());
    fs::copy(src, &dest_path)
        .map_err(|e| format!("failed to copy `{}`: {}", src.display(), e))?;
    Ok(())
}

pub fn runtime_lib_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "kira_runtime.lib"
    } else {
        "libkira_runtime.a"
    }
}

fn copy_dir_recursive_filtered(src: &Path, dst: &Path) -> Result<(), String> {
    let entries = fs::read_dir(src)
        .map_err(|e| format!("failed to read `{}`: {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read dir entry: {e}"))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name == "target" || name == ".git" {
            continue;
        }

        let file_type = entry
            .file_type()
            .map_err(|e| format!("failed to stat `{}`: {}", path.display(), e))?;

        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)
                .map_err(|e| format!("failed to create `{}`: {}", dest_path.display(), e))?;
            copy_dir_recursive_filtered(&path, &dest_path)?;
        } else if file_type.is_file() {
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("failed to copy `{}`: {}", path.display(), e))?;
        }
    }
    Ok(())
}
