use std::{env, fs, process};
use std::path::PathBuf;

use super::super::utils::get_toolchain_dir;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn cmd_toolchain_install(dev: bool) {
    if dev {
        println!("  Building development toolchain...");
        
        let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("kira"));
        let toolchain_src = current_exe
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("toolchain"))
            .filter(|p| p.exists())
            .or_else(|| {
                let cwd = env::current_dir().ok()?;
                let candidate = cwd.join("toolchain");
                if candidate.exists() {
                    Some(candidate)
                } else {
                    None
                }
            });

        let toolchain_src = match toolchain_src {
            Some(dir) => dir,
            None => {
                eprintln!("error: could not find toolchain source directory");
                eprintln!("  Make sure you're running this from the Kira repository root");
                process::exit(1);
            }
        };

        println!("  Found toolchain at: {}", toolchain_src.display());

        let status = process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&toolchain_src)
            .status();

        match status {
            Ok(status) if status.success() => {
                let built_binary = toolchain_src.join("target/release/toolchain");
                if !built_binary.exists() {
                    eprintln!("error: built binary not found at {}", built_binary.display());
                    process::exit(1);
                }

                let toolchain_dir = get_toolchain_dir();
                let version_dir = toolchain_dir.join(format!("dev-{}", VERSION));
                
                if let Err(e) = fs::create_dir_all(&version_dir) {
                    eprintln!("error: failed to create toolchain directory: {}", e);
                    process::exit(1);
                }

                let install_path = version_dir.join("kira");

                if let Err(e) = fs::copy(&built_binary, &install_path) {
                    eprintln!("error: failed to install binary: {}", e);
                    process::exit(1);
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = fs::metadata(&install_path) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(&install_path, perms);
                    }
                }

                let active_link = toolchain_dir.join("active");
                let _ = fs::remove_file(&active_link);
                
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    if let Err(e) = symlink(&version_dir, &active_link) {
                        eprintln!("warning: failed to create active symlink: {}", e);
                    }
                }
                
                #[cfg(windows)]
                {
                    use std::os::windows::fs::symlink_dir;
                    if let Err(e) = symlink_dir(&version_dir, &active_link) {
                        eprintln!("warning: failed to create active symlink: {}", e);
                    }
                }

                println!("  ✓ Development toolchain installed to: {}", version_dir.display());
                println!("  Version: dev-{}", VERSION);
                println!();
                println!("To add Kira to your PATH, run:");
                println!("  kira toolchain path");
            }
            Ok(_) => {
                eprintln!("error: cargo build failed");
                process::exit(1);
            }
            Err(e) => {
                eprintln!("error: failed to run cargo: {}", e);
                eprintln!("  Make sure cargo is installed and in your PATH");
                process::exit(1);
            }
        }
    } else {
        eprintln!("error: toolchain distribution server is not currently available");
        eprintln!();
        eprintln!("To build and install a development version locally, use:");
        eprintln!("  kira toolchain install --dev");
        process::exit(1);
    }
}

pub fn cmd_toolchain_list() {
    let toolchain_dir = get_toolchain_dir();
    
    if !toolchain_dir.exists() {
        println!("No toolchains installed.");
        println!();
        println!("To install a development toolchain, run:");
        println!("  kira toolchain install --dev");
        return;
    }

    let active_link = toolchain_dir.join("active");
    let active_target = fs::read_link(&active_link).ok();

    println!("Installed toolchains:");
    println!();

    let mut found_any = false;
    if let Ok(entries) = fs::read_dir(&toolchain_dir) {
        let mut versions: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir() && e.file_name() != "active")
            .collect();
        
        versions.sort_by_key(|e| e.file_name());

        for entry in versions {
            found_any = true;
            let version_name = entry.file_name();
            let is_active = active_target
                .as_ref()
                .and_then(|t| t.file_name())
                .map(|n| n == version_name)
                .unwrap_or(false);

            if is_active {
                println!("  {} (active)", version_name.to_string_lossy());
            } else {
                println!("  {}", version_name.to_string_lossy());
            }
        }
    }

    if !found_any {
        println!("  (none)");
        println!();
        println!("To install a development toolchain, run:");
        println!("  kira toolchain install --dev");
    }
}

pub fn cmd_toolchain_path() {
    let toolchain_dir = get_toolchain_dir();
    let bin_dir = toolchain_dir.join("active");

    if !bin_dir.exists() {
        eprintln!("error: no active toolchain found");
        eprintln!();
        eprintln!("Install a toolchain first:");
        eprintln!("  kira toolchain install --dev");
        process::exit(1);
    }

    println!("Add the following to your shell configuration:");
    println!();

    #[cfg(unix)]
    {
        let shell = env::var("SHELL").unwrap_or_else(|_| String::from("bash"));
        
        if shell.contains("fish") {
            println!("  set -gx PATH {} $PATH", bin_dir.display());
            println!();
            println!("For fish shell, add to ~/.config/fish/config.fish:");
            println!("  set -gx PATH {} $PATH", bin_dir.display());
        } else if shell.contains("zsh") {
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
            println!();
            println!("For zsh, add to ~/.zshrc:");
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
        } else {
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
            println!();
            println!("For bash, add to ~/.bashrc or ~/.bash_profile:");
            println!("  export PATH=\"{}:$PATH\"", bin_dir.display());
        }
    }

    #[cfg(windows)]
    {
        println!("  setx PATH \"%PATH%;{}\"", bin_dir.display());
        println!();
        println!("Or add it manually through System Properties > Environment Variables");
    }
}
