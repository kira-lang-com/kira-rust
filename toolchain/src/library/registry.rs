use crate::compiler::BackendKind;

use super::foundation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryFunctionSpec {
    pub full_name: &'static str,
    pub params: &'static [&'static str],
    pub return_type: &'static str,
    pub backend: BackendKind,
}

impl LibraryFunctionSpec {
    pub const fn native(
        full_name: &'static str,
        params: &'static [&'static str],
        return_type: &'static str,
    ) -> Self {
        Self {
            full_name,
            params,
            return_type,
            backend: BackendKind::Native,
        }
    }

    pub fn symbol_name(&self) -> &'static str {
        self.full_name.rsplit('.').next().unwrap_or(self.full_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryModuleSpec {
    pub package: &'static str,
    pub namespace: &'static str,
    pub functions: Vec<LibraryFunctionSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedNamespace {
    pub package: String,
    pub namespace: String,
    pub full_namespace: String,
    pub functions: Vec<LibraryFunctionSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryImportError {
    UnknownFoundationModule { module: String },
    UnknownPackage { package: String },
}

pub fn resolve_import(path: &[String]) -> Result<Vec<ImportedNamespace>, LibraryImportError> {
    let Some(package) = path.first() else {
        return Ok(Vec::new());
    };

    match package.as_str() {
        "Foundation" => resolve_foundation_import(path),
        _ => Err(LibraryImportError::UnknownPackage {
            package: package.clone(),
        }),
    }
}

pub fn all_library_modules() -> Vec<LibraryModuleSpec> {
    foundation::modules()
}

fn resolve_foundation_import(path: &[String]) -> Result<Vec<ImportedNamespace>, LibraryImportError> {
    let modules = foundation::modules();

    match path.len() {
        1 => Ok(modules
            .into_iter()
            .map(imported_namespace_from_module)
            .collect()),
        2 => {
            let module_name = &path[1];
            let module = modules
                .into_iter()
                .find(|module| module.namespace == module_name)
                .ok_or_else(|| LibraryImportError::UnknownFoundationModule {
                    module: module_name.clone(),
                })?;
            Ok(vec![imported_namespace_from_module(module)])
        }
        _ => Err(LibraryImportError::UnknownFoundationModule {
            module: path[1..].join("."),
        }),
    }
}

fn imported_namespace_from_module(module: LibraryModuleSpec) -> ImportedNamespace {
    ImportedNamespace {
        package: module.package.to_string(),
        namespace: module.namespace.to_string(),
        full_namespace: format!("{}.{}", module.package, module.namespace),
        functions: module.functions,
    }
}
