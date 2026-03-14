use std::process;

use crate::compiler::compile_project;
use crate::project::load_project;

use crate::cli::utils::find_project_root;

pub fn cmd_check() {
    let project_root = find_project_root();

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    println!("  Checking {} v{}", project.manifest.name, project.manifest.version);

    match compile_project(&project.program, &project_root) {
        Ok(_) => {
            println!("  ✓ No errors found");
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
