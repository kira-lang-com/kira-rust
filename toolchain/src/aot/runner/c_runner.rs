// C runner build pipeline

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::aot::error::AotError;
use crate::compiler::CompiledModule;

use super::c_source::generate_c_runner_source;
use super::write_if_changed;

pub fn build_c_runner_executable(
    runner_dir: &Path,
    project_name: &str,
    module: &CompiledModule,
    _module_bin: &Path,
    native_archive: &Path,
    entry_symbol: &str,
    output: &Path,
) -> Result<(), AotError> {
    fs::create_dir_all(runner_dir).map_err(|error| {
        AotError(format!(
            "failed to create runner directory `{}`: {}",
            runner_dir.display(),
            error
        ))
    })?;

    let source = generate_c_runner_source(module, entry_symbol)?;
    let runner_source = runner_dir.join(format!("{project_name}_runner.c"));
    write_if_changed(&runner_source, &source)?;

    let runtime_lib = resolve_runtime_library()?;
    compile_with_clang(
        &runner_source,
        output,
        &runtime_lib,
        native_archive,
        &module.ffi,
    )?;

    Ok(())
}

fn resolve_runtime_library() -> Result<PathBuf, AotError> {
    if let Ok(path) = env::var("KIRA_RUNTIME_LIB") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    if let Ok(exe) = env::current_exe() {
        let exe = fs::canonicalize(&exe).unwrap_or(exe);
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join(runtime_lib_name());
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    if let Ok(crate_root) = resolve_toolchain_root() {
        for profile in ["release", "debug"] {
            let base = crate_root.join("target").join("runtime").join(profile);
            for name in [runtime_lib_name(), toolchain_staticlib_name()] {
                let candidate = base.join(name);
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }

    Err(AotError(
        "could not locate runtime static library; reinstall toolchain".to_string(),
    ))
}

fn resolve_toolchain_root() -> Result<PathBuf, AotError> {
    if let Ok(value) = env::var("KIRA_TOOLCHAIN_SRC") {
        let path = PathBuf::from(value);
        if path.join("Cargo.toml").is_file() {
            return Ok(path);
        }
    }

    if let Ok(exe) = env::current_exe() {
        let exe = fs::canonicalize(&exe).unwrap_or(exe);
        if let Some(exe_dir) = exe.parent() {
            if let Some(parent) = exe_dir.parent() {
                if parent.file_name().and_then(|n| n.to_str()) == Some("target") {
                    if let Some(toolchain_dir) = parent.parent() {
                        if toolchain_dir.join("Cargo.toml").is_file() {
                            return Ok(toolchain_dir.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    Err(AotError("could not resolve toolchain root".to_string()))
}

fn runtime_lib_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "kira_runtime.lib"
    } else {
        "libkira_runtime.a"
    }
}

fn toolchain_staticlib_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "toolchain.lib"
    } else {
        "libtoolchain.a"
    }
}

fn compile_with_clang(
    source: &Path,
    output: &Path,
    runtime_lib: &Path,
    native_archive: &Path,
    ffi: &crate::compiler::FfiMetadata,
) -> Result<(), AotError> {
    let mut cmd = Command::new("clang");
    cmd.arg("-O2")
        .arg("-std=c11")
        .arg(source)
        .arg(native_archive)
        .arg(runtime_lib)
        .arg("-o")
        .arg(output);

    let emit_rpath = !cfg!(target_os = "windows");
    for link in &ffi.links {
        for path in &link.search_paths {
            cmd.arg(format!("-L{path}"));
            if emit_rpath {
                cmd.arg(format!("-Wl,-rpath,{path}"));
            }
        }
        cmd.arg(format!("-l{}", link.library));
    }

    if cfg!(target_os = "linux") {
        cmd.arg("-ldl").arg("-lpthread").arg("-lm");
    }

    let status = cmd
        .status()
        .map_err(|error| AotError(format!("failed to invoke clang: {error}")))?;
    if !status.success() {
        return Err(AotError("clang failed to build standalone runner".to_string()));
    }

    Ok(())
}
