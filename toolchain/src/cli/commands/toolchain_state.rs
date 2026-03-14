use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use super::super::utils::get_toolchain_dir;

pub fn toolchains_root() -> PathBuf {
    get_toolchain_dir()
}

pub fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

pub fn current_dir(root: &Path) -> PathBuf {
    root.join("current")
}

pub fn current_kira_path(root: &Path) -> PathBuf {
    current_dir(root).join(exe_name("kira"))
}

pub fn set_current_kira_binary(root: &Path, kira_binary: &Path) -> Result<(), String> {
    let current = current_dir(root);
    fs::create_dir_all(&current)
        .map_err(|e| format!("failed to create `{}`: {}", current.display(), e))?;

    let link_path = current_kira_path(root);
    let _ = fs::remove_file(&link_path);

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(kira_binary, &link_path).map_err(|e| {
            format!(
                "failed to create symlink `{}` -> `{}`: {}",
                link_path.display(),
                kira_binary.display(),
                e
            )
        })?;
    }

    #[cfg(windows)]
    {
        // Best-effort on Windows: copy instead of symlink (symlinks often require admin mode).
        fs::copy(kira_binary, &link_path).map_err(|e| {
            format!(
                "failed to copy `{}` to `{}`: {}",
                kira_binary.display(),
                link_path.display(),
                e
            )
        })?;
    }

    Ok(())
}

pub fn read_current_kira_target(root: &Path) -> Option<PathBuf> {
    let link = current_kira_path(root);
    let target = fs::read_link(&link).ok();
    // Normalize through canonicalize when possible (handles nested symlinks).
    // If `current/kira` is a plain file (Windows copy), canonicalize it directly.
    if let Some(t) = target {
        fs::canonicalize(t).ok().or_else(|| fs::canonicalize(link).ok())
    } else {
        fs::canonicalize(link).ok()
    }
}

pub fn print_path_instructions() {
    let root = toolchains_root();
    let dir = current_dir(&root);

    #[cfg(windows)]
    {
        println!("Add the following directory to your PATH:");
        println!();
        println!("  {}", dir.display());
        println!();
        println!("PowerShell (current session):");
        println!("  $env:Path = \"{};\" + $env:Path", dir.display());
        println!();
        println!("Persist (user PATH):");
        println!("  setx PATH \"{};%PATH%\"", dir.display());
        return;
    }

    let shell = env::var("SHELL").unwrap_or_default();
    println!("Add the following to your shell configuration so `kira` is on PATH:");
    println!();

    if shell.contains("fish") {
        println!("  set -gx PATH {} $PATH", dir.display());
        println!();
        println!("Persist in `~/.config/fish/config.fish`:");
        println!("  set -gx PATH {} $PATH", dir.display());
    } else if shell.contains("zsh") {
        println!("  export PATH=\"{}:$PATH\"", dir.display());
        println!();
        println!("Persist in `~/.zshrc`:");
        println!("  export PATH=\"{}:$PATH\"", dir.display());
    } else {
        println!("  export PATH=\"{}:$PATH\"", dir.display());
        println!();
        println!("Persist in `~/.bashrc` (or `~/.bash_profile`):");
        println!("  export PATH=\"{}:$PATH\"", dir.display());
    }
}

pub fn prompt_yes_no(question: &str, default_yes: bool) -> io::Result<bool> {
    // Avoid hanging in non-interactive contexts.
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }

    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{question} {suffix} ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let s = input.trim().to_ascii_lowercase();
    if s.is_empty() {
        return Ok(default_yes);
    }
    Ok(matches!(s.as_str(), "y" | "yes"))
}
