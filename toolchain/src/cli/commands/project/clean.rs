use std::fs;
use std::path::PathBuf;
use std::process;

pub fn cmd_clean() {
    let out_dir = PathBuf::from("out");

    if !out_dir.exists() {
        println!("  Nothing to clean");
        return;
    }

    match fs::remove_dir_all(&out_dir) {
        Ok(_) => println!("  Removed target/"),
        Err(e) => {
            eprintln!("error: failed to remove target/: {}", e);
            process::exit(1);
        }
    }
}
