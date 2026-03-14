use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use crate::ast::{Program, SourceFile, TopLevelItem};

use super::super::{manifest::ProjectManifest, ProjectError};
use super::functions::{resolve_function_body, FunctionOrigin};
use super::imports::{collect_local_modules, resolve_imports};
use super::utils::{builtin_names, display_path, normalize_manifest_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModule {
    pub name: String,
    pub relative_path: String,
    pub path: PathBuf,
    pub file: SourceFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGraph {
    pub manifest: ProjectManifest,
    pub entry_symbol: String,
    pub program: Program,
}

pub fn resolve_graph(
    manifest: ProjectManifest,
    modules: BTreeMap<String, ParsedModule>,
) -> Result<ResolvedGraph, ProjectError> {
    let entry_path = normalize_manifest_path(&manifest.entry);
    let platforms_path = manifest
        .platforms
        .as_deref()
        .map(normalize_manifest_path);
    let local_modules = collect_local_modules(&modules);

    let mut global_functions: HashMap<String, FunctionOrigin> = HashMap::new();
    for module in modules.values() {
        for item in &module.file.items {
            if let TopLevelItem::Function(function) = item {
                if let Some(previous) = global_functions.insert(
                    function.name.name.clone(),
                    FunctionOrigin {
                        file_name: display_path(&module.relative_path),
                    },
                ) {
                    return Err(ProjectError(format!(
                        "duplicate function `{}` defined in `{}` and `{}`",
                        function.name.name,
                        previous.file_name,
                        display_path(&module.relative_path)
                    )));
                }
            }
        }
    }

    let builtins = builtin_names();
    let mut resolved_items = Vec::new();
    let mut resolved_links = Vec::new();
    let mut seen_links = std::collections::HashSet::new();
    for module in modules.values() {
        let imported_namespaces = resolve_imports(module, &local_modules)?;
        for link in &module.file.links {
            // Preserve the first occurrence order across modules.
            let key = (link.library.clone(), link.header.clone());
            if seen_links.insert(key) {
                resolved_links.push(link.clone());
            }
        }
        for item in &module.file.items {
            match item {
                TopLevelItem::Struct(definition) => {
                    resolved_items.push(TopLevelItem::Struct(definition.clone()));
                }
                TopLevelItem::Function(function) => {
                    let mut resolved = function.clone();
                    resolve_function_body(
                        &mut resolved,
                        &display_path(&module.relative_path),
                        &global_functions,
                        &builtins,
                        &local_modules,
                        &imported_namespaces,
                    )?;
                    resolved_items.push(TopLevelItem::Function(resolved));
                }
            }
        }
    }

    let platforms = match platforms_path {
        Some(path) => {
            let module = modules.get(&path).ok_or_else(|| {
                ProjectError(format!(
                    "missing parsed platforms file `{path}`"
                ))
            })?;
            module.file.platforms.clone()
        }
        None => None,
    };

    let entry_module = modules
        .get(&entry_path)
        .ok_or_else(|| ProjectError(format!("entry file `{}` was not loaded", manifest.entry)))?;
    
    let entry_symbol = if matches!(manifest.kind, super::super::manifest::ProjectKind::Application) {
        let entry_has_main = entry_module.file.items.iter().any(|item| {
            matches!(
                item,
                TopLevelItem::Function(function) if function.name.name == "main"
            )
        });
        if !entry_has_main {
            return Err(ProjectError(format!(
                "entry file `{}` does not define `func main()`",
                manifest.entry
            )));
        }
        "main".to_string()
    } else {
        entry_module
            .file
            .items
            .iter()
            .find_map(|item| {
                if let TopLevelItem::Function(function) = item {
                    Some(function.name.name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "".to_string())
    };

    Ok(ResolvedGraph {
        manifest,
        entry_symbol,
        program: Program {
            platforms,
            links: resolved_links,
            items: resolved_items,
        },
    })
}

pub fn module_name_from_path(path: &str) -> Result<String, ProjectError> {
    let path_buf = PathBuf::from(path);
    let stem = path_buf
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| ProjectError(format!("invalid module path `{path}`")))?;
    Ok(stem.to_string())
}
