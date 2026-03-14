use std::path::PathBuf;
use std::process;

use crate::aot::{build_default_project, build_library_project};
use crate::project::{generate_ffi_bindings, load_project};

use crate::cli::utils::find_project_root;

pub fn cmd_build(lib: bool, _bin: bool) {
    let project_root = find_project_root();
    let out_root = PathBuf::from("target");

    if let Err(e) = generate_ffi_bindings(&project_root) {
        eprintln!("error: {}", e);
        process::exit(1);
    }

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    println!("  Compiling {} v{}", project.manifest.name, project.manifest.version);

    let start = std::time::Instant::now();
    let result = if lib {
        build_library_project(&project_root, &out_root)
    } else {
        build_default_project(&project_root, &out_root)
    };

    match result {
        Ok(output) => {
            let elapsed = start.elapsed();
            println!("  Finished in {:.1}s → {}", elapsed.as_secs_f64(), output.display());
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
