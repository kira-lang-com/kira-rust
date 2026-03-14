use std::path::Path;
use std::process;

use crate::project::{load_project, package_library, ProjectKind};

use crate::cli::utils::find_project_root;

pub fn cmd_package(output_dir: &Path) {
    let project_root = find_project_root();

    let project = match load_project(&project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    if !matches!(project.manifest.kind, ProjectKind::Library) {
        eprintln!("error: only library projects can be packaged");
        eprintln!("  Set 'kind = \"library\"' in kira.project");
        process::exit(1);
    }

    println!("  Packaging {} v{}", project.manifest.name, project.manifest.version);

    match package_library(&project_root, &project.manifest, output_dir) {
        Ok(package_path) => {
            println!("  ✓ Package created: {}", package_path.display());
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
