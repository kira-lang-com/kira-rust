use std::fs;
use std::path::Path;
use std::process;

use super::install_assets::{copy_runtime_staticlib, copy_toolchain_sources};
use super::install_build::{build_kira_binary, build_runtime_staticlib};
use super::install_paths::{built_kira_binary_path, find_toolchain_src, profile_name, read_toolchain_version, runtime_staticlib_source_name};
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

    let runtime_target = toolchain_src.join("target").join("runtime");
    build_runtime_staticlib(&toolchain_src, mode, &runtime_target).unwrap_or_else(|e| {
        eprintln!("error: failed to build runtime static library: {e}");
        process::exit(1);
    });

    let runtime_lib = runtime_target
        .join(profile_name(mode))
        .join(runtime_staticlib_source_name());
    copy_runtime_staticlib(&runtime_lib, &dest_dir).unwrap_or_else(|e| {
        eprintln!("error: failed to bundle runtime static library: {e}");
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
