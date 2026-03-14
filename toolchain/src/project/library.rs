use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::manifest::{Dependency, DependencySource, ProjectManifest};
use super::ProjectError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub manifest: ProjectManifest,
}

pub fn resolve_dependencies(
    project_root: &Path,
    manifest: &ProjectManifest,
) -> Result<Vec<ResolvedDependency>, ProjectError> {
    let mut resolved = Vec::new();
    let mut visited = HashMap::new();

    for dep in &manifest.dependencies {
        resolve_dependency_recursive(project_root, dep, &mut resolved, &mut visited)?;
    }

    Ok(resolved)
}

fn resolve_dependency_recursive(
    project_root: &Path,
    dep: &Dependency,
    resolved: &mut Vec<ResolvedDependency>,
    visited: &mut HashMap<String, String>,
) -> Result<(), ProjectError> {
    // Check for circular dependencies
    if let Some(existing_version) = visited.get(&dep.name) {
        if existing_version != &dep.version {
            return Err(ProjectError(format!(
                "version conflict for dependency '{}': requires both {} and {}",
                dep.name, existing_version, dep.version
            )));
        }
        return Ok(());
    }

    visited.insert(dep.name.clone(), dep.version.clone());

    let dep_path = match &dep.source {
        DependencySource::Path(path) => {
            let resolved_path = project_root.join(path);
            if !resolved_path.exists() {
                return Err(ProjectError(format!(
                    "dependency path does not exist: {}",
                    resolved_path.display()
                )));
            }
            resolved_path
        }
        DependencySource::Registry => {
            // Look in local registry cache
            let cache_dir = get_registry_cache_dir()?;
            let dep_dir = cache_dir.join(format!("{}-{}", dep.name, dep.version));
            
            if !dep_dir.exists() {
                return Err(ProjectError(format!(
                    "dependency '{}' version '{}' not found in registry cache. Run 'kira fetch' to download dependencies.",
                    dep.name, dep.version
                )));
            }
            dep_dir
        }
        DependencySource::Git { url, rev } => {
            // Look in git cache
            let cache_dir = get_git_cache_dir()?;
            let repo_name = url.rsplit('/').next().unwrap_or("unknown").replace(".git", "");
            let rev_str = rev.as_deref().unwrap_or("main");
            let dep_dir = cache_dir.join(format!("{}-{}", repo_name, rev_str));
            
            if !dep_dir.exists() {
                return Err(ProjectError(format!(
                    "git dependency '{}' not found in cache. Run 'kira fetch' to download dependencies.",
                    url
                )));
            }
            dep_dir
        }
    };

    // Load dependency manifest
    let dep_manifest_path = dep_path.join("kira.project");
    if !dep_manifest_path.exists() {
        return Err(ProjectError(format!(
            "dependency '{}' is missing kira.project manifest",
            dep.name
        )));
    }

    let dep_manifest = super::manifest::load_manifest(&dep_manifest_path)?;

    // Verify dependency name matches
    if dep_manifest.name != dep.name {
        return Err(ProjectError(format!(
            "dependency name mismatch: expected '{}', found '{}' in manifest",
            dep.name, dep_manifest.name
        )));
    }

    // Recursively resolve transitive dependencies
    for transitive_dep in &dep_manifest.dependencies {
        resolve_dependency_recursive(&dep_path, transitive_dep, resolved, visited)?;
    }

    resolved.push(ResolvedDependency {
        name: dep.name.clone(),
        version: dep.version.clone(),
        path: dep_path,
        manifest: dep_manifest,
    });

    Ok(())
}

pub fn get_registry_cache_dir() -> Result<PathBuf, ProjectError> {
    let cache_dir = get_kira_home()?.join("registry");
    Ok(cache_dir)
}

pub fn get_git_cache_dir() -> Result<PathBuf, ProjectError> {
    let cache_dir = get_kira_home()?.join("git");
    Ok(cache_dir)
}

fn get_kira_home() -> Result<PathBuf, ProjectError> {
    #[cfg(target_os = "macos")]
    let base = dirs::home_dir()
        .map(|h| h.join("Library/Application Support/Kira"))
        .ok_or_else(|| ProjectError("could not determine home directory".to_string()))?;
    
    #[cfg(target_os = "windows")]
    let base = dirs::data_local_dir()
        .map(|d| d.join("Kira"))
        .ok_or_else(|| ProjectError("could not determine local app data directory".to_string()))?;
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let base = dirs::home_dir()
        .map(|h| h.join(".kira"))
        .ok_or_else(|| ProjectError("could not determine home directory".to_string()))?;

    Ok(base)
}

pub fn package_library(
    project_root: &Path,
    manifest: &ProjectManifest,
    output_dir: &Path,
) -> Result<PathBuf, ProjectError> {

    // Verify this is a library project
    if !matches!(manifest.kind, super::manifest::ProjectKind::Library) {
        return Err(ProjectError(
            "only library projects can be packaged".to_string(),
        ));
    }

    // Create output directory
    fs::create_dir_all(output_dir).map_err(|e| {
        ProjectError(format!("failed to create output directory: {}", e))
    })?;

    let package_name = format!("{}-{}.kpkg", manifest.name, manifest.version);
    let package_path = output_dir.join(&package_name);

    // Create a tar.gz archive
    let tar_gz = fs::File::create(&package_path).map_err(|e| {
        ProjectError(format!("failed to create package file: {}", e))
    })?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Add manifest
    let manifest_path = project_root.join("kira.project");
    tar.append_path_with_name(&manifest_path, "kira.project")
        .map_err(|e| ProjectError(format!("failed to add manifest to package: {}", e)))?;

    // Add all .kira files
    add_kira_files_to_tar(&mut tar, project_root, project_root)?;

    tar.finish()
        .map_err(|e| ProjectError(format!("failed to finalize package: {}", e)))?;

    Ok(package_path)
}

fn add_kira_files_to_tar<W: std::io::Write>(
    tar: &mut tar::Builder<W>,
    project_root: &Path,
    current_dir: &Path,
) -> Result<(), ProjectError> {
    let entries = fs::read_dir(current_dir).map_err(|e| {
        ProjectError(format!(
            "failed to read directory {}: {}",
            current_dir.display(),
            e
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            ProjectError(format!("failed to read directory entry: {}", e))
        })?;
        let path = entry.path();

        if path.is_dir() {
            // Skip common directories
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if dir_name == "target" || dir_name.starts_with('.') {
                continue;
            }
            add_kira_files_to_tar(tar, project_root, &path)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("kira") {
            let relative_path = path.strip_prefix(project_root).map_err(|e| {
                ProjectError(format!("failed to compute relative path: {}", e))
            })?;
            tar.append_path_with_name(&path, relative_path)
                .map_err(|e| {
                    ProjectError(format!(
                        "failed to add {} to package: {}",
                        path.display(),
                        e
                    ))
                })?;
        }
    }

    Ok(())
}
