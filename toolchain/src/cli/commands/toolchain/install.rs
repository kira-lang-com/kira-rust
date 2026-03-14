use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use super::state::{
    current_kira_path, exe_name, print_path_instructions, prompt_yes_no, set_current_kira_binary,
    toolchains_root,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    Release,
    Dev,
}

pub fn cmd_install(release: bool, dev: bool) {
    let mode = match (release, dev) {
        (true, false) => InstallMode::Release,
        (false, true) => InstallMode::Dev,
        (false, false) => {
            eprintln!("error: choose an install mode: `kira install --release` or `kira install --dev`");
            process::exit(2);
        }
        (true, true) => unreachable!("clap should enforce conflicts"),
    };

    let toolchain_src = find_toolchain_src().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        eprintln!("  Run this command from the Kira repository (it needs `toolchain/Cargo.toml`).");
        process::exit(1);
    });

    let version = read_toolchain_version(&toolchain_src).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    println!(
        "  Installing Kira toolchain {} ({})...",
        version,
        match mode {
            InstallMode::Release => "release",
            InstallMode::Dev => "dev",
        }
    );

    build_kira_binary(&toolchain_src, mode).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    let built_binary = built_kira_binary_path(&toolchain_src, mode);
    if !built_binary.is_file() {
        eprintln!("error: built binary not found at {}", built_binary.display());
        process::exit(1);
    }

    let toolchains_root = toolchains_root();
    let dest_dir = toolchains_root
        .join(match mode {
            InstallMode::Release => "release",
            InstallMode::Dev => "dev",
        })
        .join(&version);

    if let Err(e) = fs::create_dir_all(&dest_dir) {
        eprintln!(
            "error: failed to create install directory `{}`: {}",
            dest_dir.display(),
            e
        );
        process::exit(1);
    }

    // Install the CLI binary.
    let dest_bin = dest_dir.join(exe_name("kira"));
    fs::copy(&built_binary, &dest_bin).unwrap_or_else(|e| {
        eprintln!(
            "error: failed to install binary to `{}`: {}",
            dest_bin.display(),
            e
        );
        process::exit(1);
    });
    make_executable(&dest_bin);

    // Bundle toolchain sources so AOT runner builds can `path = ".../toolchain"`.
    let dest_src = dest_dir.join("toolchain");
    copy_toolchain_sources(&toolchain_src, &dest_src).unwrap_or_else(|e| {
        eprintln!("error: failed to bundle toolchain sources: {e}");
        process::exit(1);
    });

    println!("  ✓ Installed to: {}", dest_dir.display());
    println!("  Binary: {}", dest_bin.display());

    let has_current = current_kira_path(&toolchains_root).exists();
    let default_yes = !has_current;
    let set_current = prompt_yes_no("Set as current toolchain?", default_yes).unwrap_or(false);
    if set_current {
        if let Err(e) = set_current_kira_binary(&toolchains_root, &dest_bin) {
            eprintln!("warning: failed to set current toolchain: {e}");
        } else {
            println!("  ✓ Current: {}", current_kira_path(&toolchains_root).display());
        }
    }

    println!();
    print_path_instructions();
}

fn find_toolchain_src() -> Result<PathBuf, String> {
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

fn read_toolchain_version(toolchain_src: &Path) -> Result<String, String> {
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

fn build_kira_binary(toolchain_src: &Path, mode: InstallMode) -> Result<(), String> {
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

fn built_kira_binary_path(toolchain_src: &Path, mode: InstallMode) -> PathBuf {
    let profile = if mode == InstallMode::Release {
        "release"
    } else {
        "debug"
    };
    toolchain_src.join("target").join(profile).join(exe_name("kira"))
}

fn copy_toolchain_sources(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.join("Cargo.toml").is_file() {
        return Err(format!(
            "toolchain source directory `{}` does not look like a Rust crate",
            src.display()
        ));
    }
    fs::create_dir_all(dst).map_err(|e| format!("failed to create `{}`: {}", dst.display(), e))?;
    copy_dir_recursive_filtered(src, dst)
}

fn copy_dir_recursive_filtered(src: &Path, dst: &Path) -> Result<(), String> {
    let entries = fs::read_dir(src)
        .map_err(|e| format!("failed to read `{}`: {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read dir entry: {e}"))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if name == "target" || name == ".git" || name == "out" {
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

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(path, perms);
        }
    }
}
