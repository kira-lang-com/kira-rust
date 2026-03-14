use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::ast::syntax::AssignTarget;
use crate::library::LibraryImportError;

use super::super::ProjectError;

pub fn builtin_names() -> BTreeSet<String> {
    ["printIn", "abs", "pow", "max", "min"]
        .into_iter()
        .map(String::from)
        .collect()
}

pub fn import_error(error: LibraryImportError, current_file: &str) -> ProjectError {
    match error {
        LibraryImportError::UnknownFoundationModule { module } => {
            ProjectError(format!("Foundation.{} does not exist", module))
        }
        LibraryImportError::UnknownPackage { package } => ProjectError(format!(
            "package `{}` imported in `{}` does not exist",
            package, current_file
        )),
    }
}

pub fn normalize_manifest_path(path: &str) -> String {
    PathBuf::from(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn display_path(path: &str) -> String {
    path.to_string()
}

pub fn assign_target_root_name(target: &AssignTarget) -> &str {
    match target {
        AssignTarget::Variable(identifier) => &identifier.name,
        AssignTarget::Field { target, .. } => assign_target_root_name(target),
    }
}
