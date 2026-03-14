use std::fs;
use std::process;

use crate::project::{load_manifest, resolve_dependencies, DependencySource};

use super::super::utils::find_project_root;

pub fn cmd_fetch() {
    let project_root = find_project_root();
    let manifest_path = project_root.join("kira.project");

    let manifest = match load_manifest(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    if manifest.dependencies.is_empty() {
        println!("  No dependencies to fetch");
        return;
    }

    println!("  Fetching dependencies for {} v{}", manifest.name, manifest.version);

    match resolve_dependencies(&project_root, &manifest) {
        Ok(deps) => {
            if deps.is_empty() {
                println!("  All dependencies already cached");
            } else {
                println!("  ✓ Resolved {} dependencies", deps.len());
                for dep in deps {
                    println!("    - {} v{}", dep.name, dep.version);
                }
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}

pub fn cmd_add(name: &str, version: Option<String>, path: Option<String>, git: Option<String>) {
    let project_root = find_project_root();
    let manifest_path = project_root.join("kira.project");

    let mut manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("error: failed to read kira.project: {}", e);
            process::exit(1);
        }
    };

    let source = if let Some(path_str) = path {
        DependencySource::Path(path_str.clone())
    } else if let Some(git_url) = git {
        DependencySource::Git {
            url: git_url.clone(),
            rev: None,
        }
    } else {
        DependencySource::Registry
    };

    let version_str = version.unwrap_or_else(|| "*".to_string());

    let has_deps_section = manifest_content.contains("[dependencies]");

    if !has_deps_section {
        manifest_content.push_str("\n[dependencies]\n");
    }

    let dep_line = match source {
        DependencySource::Registry => format!("{} = \"{}\"\n", name, version_str),
        DependencySource::Path(ref p) => {
            format!("{} = {{ version = \"{}\", path = \"{}\" }}\n", name, version_str, p)
        }
        DependencySource::Git { ref url, .. } => {
            format!("{} = {{ version = \"{}\", git = \"{}\" }}\n", name, version_str, url)
        }
    };

    manifest_content.push_str(&dep_line);

    if let Err(e) = fs::write(&manifest_path, manifest_content) {
        eprintln!("error: failed to write kira.project: {}", e);
        process::exit(1);
    }

    println!("  ✓ Added dependency: {} v{}", name, version_str);
    println!();
    println!("Run 'kira fetch' to download the dependency");
}
