use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::ast::syntax::{Expression, ExpressionKind, Identifier};
use crate::library::{resolve_import, ImportedNamespace};

use super::super::ProjectError;
use super::graph::ParsedModule;
use super::utils::{display_path, import_error};

pub fn collect_local_modules(modules: &BTreeMap<String, ParsedModule>) -> HashMap<String, String> {
    let mut local_modules = HashMap::new();
    for module in modules.values() {
        local_modules.insert(module.name.clone(), display_path(&module.relative_path));
    }
    local_modules
}

pub fn resolve_imports(
    module: &ParsedModule,
    local_modules: &HashMap<String, String>,
) -> Result<HashMap<String, ImportedNamespace>, ProjectError> {
    let mut imported: HashMap<String, ImportedNamespace> = HashMap::new();

    for import in &module.file.imports {
        let path = import
            .path
            .iter()
            .map(|segment| segment.name.clone())
            .collect::<Vec<_>>();
        let import_name = path.join(".");

        if let Some(first) = path.first() {
            if local_modules.contains_key(first) && !first.contains("::") {
                return Err(ProjectError(format!(
                    "internal project import `import {};` is not allowed in `{}`; `{}` is already in the project-wide global scope",
                    import_name,
                    display_path(&module.relative_path),
                    local_modules.get(first).unwrap()
                )));
            }

            let is_dependency = local_modules.keys().any(|k| k.starts_with(&format!("{}::", first)));
            if is_dependency {
                let lib_namespace = ImportedNamespace {
                    package: first.clone(),
                    namespace: first.clone(),
                    full_namespace: first.clone(),
                    functions: Vec::new(),
                };
                imported.insert(first.clone(), lib_namespace);
                continue;
            }
        }

        let namespaces = resolve_import(&path).map_err(|error| {
            import_error(error, &display_path(&module.relative_path))
        })?;

        for namespace in namespaces {
            match imported.get(&namespace.namespace) {
                Some(existing) if existing.full_namespace == namespace.full_namespace => {}
                Some(existing) => {
                    return Err(ProjectError(format!(
                        "namespace `{}` is imported from both `{}` and `{}` in `{}`",
                        namespace.namespace,
                        existing.full_namespace,
                        namespace.full_namespace,
                        display_path(&module.relative_path)
                    )));
                }
                None => {
                    imported.insert(namespace.namespace.clone(), namespace);
                }
            }
        }
    }

    Ok(imported)
}

pub fn resolve_imported_callee(
    callee: &mut Expression,
    current_file: &str,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &BTreeSet<String>,
) -> Result<(), ProjectError> {
    let ExpressionKind::Member { target, field } = &callee.kind else {
        return Ok(());
    };
    let ExpressionKind::Variable(identifier) = &target.kind else {
        return Ok(());
    };
    let qualifier = &identifier.name;
    let symbol = &field.name;

    if local_modules.contains_key(qualifier) && !qualifier.contains("::") {
        return Err(ProjectError(format!(
            "internal project function `{}.{}` in `{}` does not need qualification; use `{}` instead",
            qualifier, symbol, current_file, symbol
        )));
    }

    if scope.contains(qualifier) {
        return Ok(());
    }

    if imported_namespaces.contains_key(qualifier) {
        let namespace = imported_namespaces.get(qualifier).unwrap();
        
        if !namespace.package.starts_with("Foundation") {
            let replacement = Identifier {
                name: symbol.clone(),
                span: field.span.clone(),
            };
            *callee = Expression::new(ExpressionKind::Variable(replacement), callee.span.clone());
            return Ok(());
        }

        let function = namespace
            .functions
            .iter()
            .find(|function| function.symbol_name() == symbol)
            .ok_or_else(|| {
                ProjectError(format!(
                    "{}.{} does not exist",
                    namespace.full_namespace, symbol
                ))
            })?;

        let replacement = Identifier {
            name: function.full_name.to_string(),
            span: field.span.clone(),
        };
        *callee = Expression::new(ExpressionKind::Variable(replacement), callee.span.clone());
        return Ok(());
    }

    Err(ProjectError(format!(
        "namespace `{}` is not imported in `{}`",
        qualifier, current_file
    )))
}
