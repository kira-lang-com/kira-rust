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
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Kira/toolchains"))
            .unwrap_or_else(|| PathBuf::from("~/.kira/toolchains"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .map(|d| d.join("Kira\\toolchains"))
            .unwrap_or_else(|| PathBuf::from("%LOCALAPPDATA%\\Kira\\toolchains"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::home_dir()
            .map(|h| h.join(".kira/toolchains"))
            .unwrap_or_else(|| PathBuf::from("~/.kira/toolchains"))
    }
}
