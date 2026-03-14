use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::parse;

use super::library::resolve_dependencies;
use super::manifest::{load_manifest, ProjectManifest};
use super::resolver::{module_name_from_path, resolve_graph, ParsedModule};
use super::ProjectError;
use crate::ast::syntax::LinkDirective;
use super::manifest::{DependencySource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProject {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
    pub entry_symbol: String,
    pub program: crate::ast::syntax::Program,
}

pub fn load_project(root: &Path) -> Result<ResolvedProject, ProjectError> {
    let manifest_path = root.join("kira.project");
    let manifest = load_manifest(&manifest_path)?;

    // Dependencies serve two roles:
    // 1) Kira package dependencies (directories containing a `kira.project`)
    // 2) Native header dependencies for auto-FFI (`.h` paths), which are converted into `@Link`.
    let (native_links, kira_manifest) = split_native_header_dependencies(root, &manifest);

    // Resolve Kira package dependencies first (skip native header dependencies).
    let dependencies = resolve_dependencies(root, &kira_manifest)?;
    
    let entry_path = root.join(&manifest.entry);
    let entry_key = normalize_relative_path(Path::new(&manifest.entry));
    let platforms_key = manifest
        .platforms
        .as_deref()
        .map(|path| normalize_relative_path(Path::new(path)));

    if !entry_path.is_file() {
        return Err(ProjectError(format!(
            "entry file `{}` does not exist",
            manifest.entry
        )));
    }

    if let Some(platforms_path) = manifest.platforms.as_ref() {
        let path = root.join(platforms_path);
        if !path.is_file() {
            return Err(ProjectError(format!(
                "platforms file `{}` does not exist",
                platforms_path
            )));
        }
    }

    // Load project modules
    let mut modules = BTreeMap::new();
    for relative_path in collect_project_files(root, root)? {
        let key = normalize_relative_path(&relative_path);
        let path = root.join(&relative_path);
        let source = fs::read_to_string(&path).map_err(|error| {
            ProjectError(format!(
                "failed to read module `{}`: {}",
                path.display(),
                error
            ))
        })?;

        let file = parse(&source).map_err(|errors| {
            ProjectError(format!(
                "failed to parse module `{}`:\n{}",
                path.display(),
                errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        })?;

        modules.insert(
            key.clone(),
            ParsedModule {
                name: module_name_from_path(&key)?,
                relative_path: key,
                path,
                file,
            },
        );
    }

    // Load dependency modules
    for dep in &dependencies {
        for relative_path in collect_project_files(&dep.path, &dep.path)? {
            let key = format!("{}::{}", dep.name, normalize_relative_path(&relative_path));
            let path = dep.path.join(&relative_path);
            let source = fs::read_to_string(&path).map_err(|error| {
                ProjectError(format!(
                    "failed to read dependency module `{}`: {}",
                    path.display(),
                    error
                ))
            })?;

            let file = parse(&source).map_err(|errors| {
                ProjectError(format!(
                    "failed to parse dependency module `{}`:\n{}",
                    path.display(),
                    errors
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            })?;

            modules.insert(
                key.clone(),
                ParsedModule {
                    name: format!("{}::{}", dep.name, module_name_from_path(&normalize_relative_path(&relative_path))?),
                    relative_path: key,
                    path,
                    file,
                },
            );
        }
    }

    if !modules.contains_key(&entry_key) {
        return Err(ProjectError(format!(
            "entry file `{}` was not loaded",
            manifest.entry
        )));
    }

    if let Some(platforms_key) = platforms_key.as_ref() {
        if !modules.contains_key(platforms_key) {
            return Err(ProjectError(format!(
                "platforms file `{}` was not loaded",
                manifest.platforms.as_deref().unwrap_or_default()
            )));
        }
    }

    let resolved = resolve_graph(manifest.clone(), modules)?;
    let program = merge_program_links(resolved.program, native_links);

    Ok(ResolvedProject {
        root: root.to_path_buf(),
        manifest,
        entry_symbol: resolved.entry_symbol,
        program,
    })
}

fn collect_project_files(root: &Path, current: &Path) -> Result<Vec<PathBuf>, ProjectError> {
    let mut files = Vec::new();
    let entries = fs::read_dir(current).map_err(|error| {
        ProjectError(format!(
            "failed to read project directory `{}`: {}",
            current.display(),
            error
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            ProjectError(format!(
                "failed to read directory entry in `{}`: {}",
                current.display(),
                error
            ))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            ProjectError(format!("failed to inspect `{}`: {}", path.display(), error))
        })?;

        if file_type.is_dir() {
            files.extend(collect_project_files(root, &path)?);
            continue;
        }

        if !file_type.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("kira") {
            continue;
        }

        let relative = path.strip_prefix(root).map_err(|error| {
            ProjectError(format!(
                "failed to compute relative project path for `{}`: {}",
                path.display(),
                error
            ))
        })?;
        files.push(relative.to_path_buf());
    }

    files.sort();
    Ok(files)
}

fn normalize_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn split_native_header_dependencies(
    project_root: &Path,
    manifest: &ProjectManifest,
) -> (Vec<LinkDirective>, ProjectManifest) {
    let mut native_links = Vec::new();
    let mut kira_deps = Vec::new();

    for dep in &manifest.dependencies {
        let DependencySource::Path(path) = &dep.source else {
            kira_deps.push(dep.clone());
            continue;
        };

        // Treat `path = ".../*.h"` dependencies as native header links (auto `@Link`).
        // If the path isn't a header file on disk, keep it as a regular Kira dependency.
        let candidate = project_root.join(path);
        let is_header = candidate.is_file()
            && candidate
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("h"))
                .unwrap_or(false);

        if is_header {
            native_links.push(LinkDirective {
                library: dep.name.clone(),
                header: path.clone(),
                span: 0..0,
            });
        } else {
            kira_deps.push(dep.clone());
        }
    }

    let mut kira_manifest = manifest.clone();
    kira_manifest.dependencies = kira_deps;

    (native_links, kira_manifest)
}

fn merge_program_links(mut program: crate::ast::syntax::Program, links: Vec<LinkDirective>) -> crate::ast::syntax::Program {
    if links.is_empty() {
        return program;
    }
    let mut seen = std::collections::HashSet::new();
    for link in &program.links {
        seen.insert((link.library.clone(), link.header.clone()));
    }
    for link in links {
        let key = (link.library.clone(), link.header.clone());
        if seen.insert(key) {
            program.links.push(link);
        }
    }
    program
}
