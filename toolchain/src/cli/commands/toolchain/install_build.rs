// Install build steps

use std::path::Path;
use std::process;

use super::install::InstallMode;

pub fn build_kira_binary(toolchain_src: &Path, mode: InstallMode) -> Result<(), String> {
    let mut cmd = process::Command::new("cargo");
    cmd.arg("build").arg("--bin").arg("kira");
    if mode == InstallMode::Release {
        cmd.arg("--release");
    }
    cmd.current_dir(toolchain_src);

    let status = cmd
        .status()
        .map_err(|e| format!("failed to run cargo build: {e}"))?;
    if !status.success() {
        return Err("cargo build failed".to_string());
    }
    Ok(())
}

pub fn build_runtime_staticlib(
    toolchain_src: &Path,
    mode: InstallMode,
    target_dir: &Path,
) -> Result<(), String> {
    let mut cmd = process::Command::new("cargo");
    cmd.arg("build").arg("--lib").arg("--no-default-features");
    if mode == InstallMode::Release {
        cmd.arg("--release");
    }

    cmd.env("CARGO_TARGET_DIR", target_dir)
        .current_dir(toolchain_src);

    let status = cmd
        .status()
        .map_err(|e| format!("failed to run cargo build for runtime staticlib: {e}"))?;
    if !status.success() {
        return Err("cargo build for runtime staticlib failed".to_string());
    }
    Ok(())
}
