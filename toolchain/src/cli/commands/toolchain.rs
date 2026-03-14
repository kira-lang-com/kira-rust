use std::{fs, process};

use super::install::cmd_install;
use super::toolchain_state::{
    print_path_instructions, read_current_kira_target, set_current_kira_binary, toolchains_root,
};

pub fn cmd_toolchain_install(dev: bool) {
    eprintln!("warning: `kira toolchain install` is deprecated; use `kira install --dev/--release`");
    cmd_install(!dev, dev);
}

pub fn cmd_toolchain_list() {
    let toolchain_dir = toolchains_root();
    
    if !toolchain_dir.exists() {
        println!("No toolchains installed.");
        println!();
        println!("To install a development toolchain, run:");
        println!("  kira install --dev");
        return;
    }

    let current_target = read_current_kira_target(&toolchain_dir);
    let current_name = current_target
        .as_ref()
        .and_then(|t| t.strip_prefix(&toolchain_dir).ok())
        .and_then(|rel| {
            let mut it = rel.iter();
            let mode = it.next()?.to_string_lossy().into_owned();
            let version = it.next()?.to_string_lossy().into_owned();
            Some(format!("{}/{}", mode, version))
        });

    println!("Installed toolchains:");
    println!();

    let mut found_any = false;
    for mode in ["release", "dev"] {
        let mode_dir = toolchain_dir.join(mode);
        let Ok(entries) = fs::read_dir(&mode_dir) else {
            continue;
        };
        let mut versions: Vec<_> = entries.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect();
        versions.sort_by_key(|e| e.file_name());

        for entry in versions {
            found_any = true;
            let version_name = entry.file_name().to_string_lossy().into_owned();
            let full_name = format!("{}/{}", mode, version_name);
            let is_current = current_name
                .as_ref()
                .map(|n| n == &full_name)
                .unwrap_or(false);

            if is_current {
                println!("  {} (current)", full_name);
            } else {
                println!("  {}", full_name);
            }
        }
    }

    if !found_any {
        println!("  (none)");
        println!();
        println!("To install a development toolchain, run:");
        println!("  kira install --dev");
    }
}

pub fn cmd_toolchain_use(toolchain: &str) {
    let toolchain_dir = toolchains_root();
    let (mode, version) = toolchain
        .split_once('/')
        .map(|(a, b)| (a.trim(), b.trim()))
        .unwrap_or(("", ""));

    if mode.is_empty() || version.is_empty() || !(mode == "dev" || mode == "release") {
        eprintln!("error: invalid toolchain `{}`", toolchain);
        eprintln!("  Expected: `dev/<version>` or `release/<version>`");
        process::exit(2);
    }

    let kira_bin = toolchain_dir.join(mode).join(version).join(exe_name("kira"));
    if !kira_bin.is_file() {
        eprintln!(
            "error: toolchain not found: `{}`",
            toolchain_dir.join(mode).join(version).display()
        );
        eprintln!();
        eprintln!("Installed toolchains:");
        cmd_toolchain_list();
        process::exit(1);
    }

    if let Err(e) = set_current_kira_binary(&toolchain_dir, &kira_bin) {
        eprintln!("error: {e}");
        process::exit(1);
    }

    println!("  ✓ Set current toolchain to: {}/{}", mode, version);
    println!();
    print_path_instructions();
}

pub fn cmd_toolchain_path() {
    let toolchain_dir = toolchains_root();
    let current = read_current_kira_target(&toolchain_dir);
    if current.is_none() {
        eprintln!("error: no current toolchain is set");
        eprintln!();
        eprintln!("Install a toolchain first:");
        eprintln!("  kira install --dev");
        eprintln!();
        eprintln!("Or select an installed toolchain:");
        eprintln!("  kira toolchain use dev/<version>");
        process::exit(1);
    }

    print_path_instructions();
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
