// Install path resolution helpers

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::state::exe_name;
use super::install::InstallMode;

pub fn find_toolchain_src() -> Result<PathBuf, String> {
    // Try walking up from CWD first.
    if let Ok(cwd) = env::current_dir() {
        let mut dir = cwd.as_path();
        loop {
            let candidate = dir.join("toolchain").join("Cargo.toml");
            if candidate.is_file() {
                return Ok(dir.join("toolchain"));
            }
            let candidate = dir.join("Cargo.toml");
            if candidate.is_file() && dir.file_name().and_then(|n| n.to_str()) == Some("toolchain") {
                return Ok(dir.to_path_buf());
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    // Fallback: try deriving from current executable location (installed or repo builds).
    if let Ok(exe) = env::current_exe() {
        let exe = fs::canonicalize(&exe).unwrap_or(exe);
        if let Some(exe_dir) = exe.parent() {
            // Installed layout: `<install_root>/kira` and `<install_root>/toolchain/Cargo.toml`.
            let candidate = exe_dir.join("toolchain").join("Cargo.toml");
            if candidate.is_file() {
                return Ok(exe_dir.join("toolchain"));
            }

            // Repo layout: `toolchain/target/<profile>/kira`.
            if let Some(dir) = exe_dir.parent().and_then(|p| p.parent()) {
                if dir.file_name().and_then(|n| n.to_str()) == Some("toolchain")
                    && dir.join("Cargo.toml").is_file()
                {
                    return Ok(dir.to_path_buf());
                }
            }
        }
    }

    Err("could not locate `toolchain/Cargo.toml`".to_string())
}

pub fn read_toolchain_version(toolchain_src: &Path) -> Result<String, String> {
    let toml_path = toolchain_src.join("Cargo.toml");
    let text = fs::read_to_string(&toml_path)
        .map_err(|e| format!("failed to read `{}`: {}", toml_path.display(), e))?;

    let mut in_package = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() != "version" {
                continue;
            }
            let v = v.trim();
            if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
                return Ok(v[1..v.len() - 1].to_string());
            }
        }
    }

    Err("could not parse toolchain version from toolchain/Cargo.toml".to_string())
}

pub fn built_kira_binary_path(toolchain_src: &Path, mode: InstallMode) -> PathBuf {
    toolchain_src
        .join("target")
        .join(profile_name(mode))
        .join(exe_name("kira"))
}

pub fn profile_name(mode: InstallMode) -> &'static str {
    if mode == InstallMode::Release {
        "release"
    } else {
        "debug"
    }
}

pub fn runtime_staticlib_source_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "toolchain.lib"
    } else {
        "libtoolchain.a"
    }
}
