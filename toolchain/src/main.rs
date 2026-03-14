use std::path::Path;

use toolchain::aot::{build_default_project, run_default_project};
use toolchain::compiler::compile;
use toolchain::project::load_project;
use toolchain::runtime::vm::Vm;

fn main() {
    let project_root = if cfg!(debug_assertions) {
        Path::new("../app")
    } else {
        Path::new("./app")
    };
    let out_root = if cfg!(debug_assertions) {
        Path::new("../out")
    } else {
        Path::new("./out")
    };

    match std::env::args().nth(1).as_deref() {
        Some("build") => match build_default_project(project_root, out_root) {
            Ok(binary) => {
                println!("{}", binary.display());
                return;
            }
            Err(error) => {
                eprintln!("Build Error:\n{}", error);
                std::process::exit(1);
            }
        },
        Some("run") => match run_default_project(project_root, out_root) {
            Ok(code) => {
                std::process::exit(code);
            }
            Err(error) => {
                eprintln!("Run Error:\n{}", error);
                std::process::exit(1);
            }
        },
        _ => {}
    }

    let project = match load_project(project_root) {
        Ok(project) => project,
        Err(error) => {
            eprintln!("Project Error:\n{}", error);
            std::process::exit(1);
        }
    };

    let module = match compile(&project.program) {
        Ok(module) => module,
        Err(error) => {
            eprintln!("Compile Error:\n{}", error);
            std::process::exit(1);
        }
    };

    let mut vm = Vm::default();
    match vm.run_entry(&module, &project.entry_symbol) {
        Ok(_) => {
            for line in vm.output() {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("Runtime Error:\n{}", error);
            std::process::exit(1);
        }
    }
}
