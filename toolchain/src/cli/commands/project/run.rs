use std::path::PathBuf;
use std::process;

use crate::aot::{build_default_project, run_default_project};
use crate::project::load_project;

use crate::cli::utils::find_project_root;

pub fn cmd_run() {
    let project_root = find_project_root();
    let out_root = PathBuf::from("out");

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    println!("  Compiling {} v{}", project.manifest.name, project.manifest.version);

    let start = std::time::Instant::now();
    match run_default_project(&project_root, &out_root) {
        Ok(code) => {
            let elapsed = start.elapsed();
            println!("  Finished in {:.1}s", elapsed.as_secs_f64());
            process::exit(code);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
