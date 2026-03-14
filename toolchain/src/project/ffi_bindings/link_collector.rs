// Link directive collection for FFI bindings

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::LinkDirective;
use crate::parser::parse;

use super::super::manifest::{load_manifest, DependencySource};
use super::super::ProjectError;

pub fn collect_links(root: &Path) -> Result<Vec<LinkDirective>, ProjectError> {
    let manifest_path = root.join("kira.project");
    let manifest = load_manifest(&manifest_path)?;
    let mut links = Vec::new();
    let mut seen = HashSet::new();

    for dep in &manifest.dependencies {
        let DependencySource::Path(path) = &dep.source else {
            continue;
        };
        let candidate = root.join(path);
        let is_header = candidate.is_file()
            && candidate
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("h"))
                .unwrap_or(false);
        if !is_header {
            continue;
        }
        let link = LinkDirective {
            library: dep.name.clone(),
            header: path.clone(),
            span: 0..0,
        };
        let key = (link.library.clone(), link.header.clone());
        if seen.insert(key) {
            links.push(link);
        }
    }

    for relative_path in collect_project_files(root, root)? {
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

        for link in file.links {
            let key = (link.library.clone(), link.header.clone());
            if seen.insert(key) {
                links.push(link);
            }
        }
    }

    Ok(links)
}

pub fn resolve_header_path(project_root: &Path, header: &str) -> Result<PathBuf, ProjectError> {
    let candidate = project_root.join(header);
    if candidate.is_file() {
        return Ok(candidate);
    }
    Err(ProjectError(format!(
        "linked header `{}` does not exist (looked for `{}`)",
        header,
        candidate.display()
    )))
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
