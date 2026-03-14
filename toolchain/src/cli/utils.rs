use std::{env, process};
use std::path::PathBuf;

pub fn find_project_root() -> PathBuf {
    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    let mut dir = current.as_path();
    loop {
        let manifest = dir.join("kira.project");
        if manifest.exists() {
            return dir.to_path_buf();
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                eprintln!("error: no kira.project found in current directory or any parent directory");
                process::exit(1);
            }
        }
    }
}

pub fn get_toolchain_dir() -> PathBuf {
    // Keep this consistent across platforms: `~/kira/toolchains/...`
    // (Windows will resolve `home_dir()` to the user profile directory.)
    dirs::home_dir()
        .map(|h| h.join("kira/toolchains"))
        .unwrap_or_else(|| PathBuf::from("~/kira/toolchains"))
}
