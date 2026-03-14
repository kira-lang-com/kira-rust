use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use crate::ast::syntax::{
    AssignTarget, Block, Expression, ExpressionKind, FunctionDefinition, Identifier, Program,
    SourceFile, Statement, TopLevelItem,
};
use crate::library::{resolve_import, ImportedNamespace, LibraryImportError};

use super::{manifest::ProjectManifest, ProjectError};

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
    for module in modules.values() {
        let imported_namespaces = resolve_imports(module, &local_modules)?;
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

    Ok(ResolvedGraph {
        manifest,
        entry_symbol: "main".to_string(),
        program: Program {
            platforms,
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

fn builtin_names() -> BTreeSet<String> {
    ["printIn", "abs", "pow", "max", "min"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn resolve_function_body(
    function: &mut FunctionDefinition,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
) -> Result<(), ProjectError> {
    let mut scope = function
        .params
        .iter()
        .map(|parameter| parameter.name.name.clone())
        .collect::<BTreeSet<_>>();

    resolve_block(
        &mut function.body,
        current_file,
        global_functions,
        builtins,
        local_modules,
        imported_namespaces,
        &mut scope,
    )
}

fn resolve_block(
    block: &mut Block,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &mut BTreeSet<String>,
) -> Result<(), ProjectError> {
    for statement in &mut block.statements {
        match statement {
            Statement::Let(statement) => {
                resolve_expression(
                    &mut statement.value,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
                scope.insert(statement.name.name.clone());
            }
            Statement::Assign(statement) => {
                if !scope.contains(assign_target_root_name(&statement.target)) {
                    return Err(ProjectError(format!(
                        "unknown local `{}` in `{}`",
                        assign_target_root_name(&statement.target),
                        current_file
                    )));
                }
                resolve_expression(
                    &mut statement.value,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Statement::Return(statement) => {
                resolve_expression(
                    &mut statement.expression,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Statement::Expression(statement) => {
                resolve_expression(
                    &mut statement.expression,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Statement::If(statement) => {
                resolve_expression(
                    &mut statement.condition,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
                let mut then_scope = scope.clone();
                resolve_block(
                    &mut statement.then_block,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    &mut then_scope,
                )?;
                if let Some(else_block) = &mut statement.else_block {
                    let mut else_scope = scope.clone();
                    resolve_block(
                        else_block,
                        current_file,
                        global_functions,
                        builtins,
                        local_modules,
                        imported_namespaces,
                        &mut else_scope,
                    )?;
                }
            }
            Statement::While(statement) => {
                resolve_expression(
                    &mut statement.condition,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
                let mut body_scope = scope.clone();
                resolve_block(
                    &mut statement.body,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    &mut body_scope,
                )?;
            }
            Statement::For(statement) => {
                resolve_expression(
                    &mut statement.iterable,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
                let mut body_scope = scope.clone();
                body_scope.insert(statement.binding.name.clone());
                resolve_block(
                    &mut statement.body,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    &mut body_scope,
                )?;
            }
            Statement::Break(_) | Statement::Continue(_) => {}
        }
    }

    Ok(())
}

fn resolve_expression(
    expression: &mut Expression,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &BTreeSet<String>,
) -> Result<(), ProjectError> {
    match &mut expression.kind {
        ExpressionKind::Literal(_) | ExpressionKind::Variable(_) => Ok(()),
        ExpressionKind::ArrayLiteral(elements) => {
            for element in elements {
                resolve_expression(
                    element,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Ok(())
        }
        ExpressionKind::StructLiteral { fields, .. } => {
            for field in fields {
                resolve_expression(
                    &mut field.value,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Ok(())
        }
        ExpressionKind::Index { target, index } => {
            resolve_expression(
                target,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )?;
            resolve_expression(
                index,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )
        }
        ExpressionKind::Member { target, field } => {
            resolve_expression(
                target,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )?;

            if let ExpressionKind::Variable(identifier) = &target.kind {
                if scope.contains(&identifier.name) {
                    return Ok(());
                }

                if local_modules.contains_key(&identifier.name) {
                    return Err(ProjectError(format!(
                        "internal project function `{}.{}` in `{}` does not need qualification; use `{}` instead",
                        identifier.name, field.name, current_file, field.name
                    )));
                }

                if imported_namespaces.contains_key(&identifier.name) {
                    return Err(ProjectError(format!(
                        "library reference `{}.{}` in `{}` can only be used as a function call",
                        identifier.name, field.name, current_file
                    )));
                }
            }

            Ok(())
        }
        ExpressionKind::Call { callee, arguments } => {
            for argument in arguments {
                resolve_expression(
                    argument,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }

            resolve_callee(
                callee,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )
        }
        ExpressionKind::Range { start, end, .. } => {
            resolve_expression(
                start,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )?;
            resolve_expression(
                end,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )
        }
        ExpressionKind::Cast { expr, .. } | ExpressionKind::Unary { expr, .. } => {
            resolve_expression(
                expr,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )
        }
        ExpressionKind::Binary { left, right, .. } => {
            resolve_expression(
                left,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )?;
            resolve_expression(
                right,
                current_file,
                global_functions,
                builtins,
                local_modules,
                imported_namespaces,
                scope,
            )
        }
    }
}

fn resolve_callee(
    callee: &mut Expression,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &BTreeSet<String>,
) -> Result<(), ProjectError> {
    match &mut callee.kind {
        ExpressionKind::Variable(identifier) => {
            if scope.contains(&identifier.name) {
                return Err(ProjectError(format!(
                    "`{}` in `{}` refers to a local binding, not a function",
                    identifier.name, current_file
                )));
            }

            if builtins.contains(&identifier.name) || global_functions.contains_key(&identifier.name)
            {
                return Ok(());
            }

            Err(ProjectError(format!(
                "unknown function `{}` referenced in `{}`",
                identifier.name, current_file
            )))
        }
        ExpressionKind::Member { .. } => {
            resolve_imported_callee(
                callee,
                current_file,
                local_modules,
                imported_namespaces,
                scope,
            )
        }
        _ => Ok(()),
    }
}

#[derive(Debug, Clone)]
struct FunctionOrigin {
    file_name: String,
}

fn collect_local_modules(modules: &BTreeMap<String, ParsedModule>) -> HashMap<String, String> {
    let mut local_modules = HashMap::new();
    for module in modules.values() {
        local_modules.insert(module.name.clone(), display_path(&module.relative_path));
    }
    local_modules
}

fn resolve_imports(
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
            if let Some(local_file) = local_modules.get(first) {
                return Err(ProjectError(format!(
                    "internal project import `import {};` is not allowed in `{}`; `{}` is already in the project-wide global scope",
                    import_name,
                    display_path(&module.relative_path),
                    local_file
                )));
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

fn resolve_imported_callee(
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

    if local_modules.contains_key(qualifier) {
        return Err(ProjectError(format!(
            "internal project function `{}.{}` in `{}` does not need qualification; use `{}` instead",
            qualifier, symbol, current_file, symbol
        )));
    }

    if scope.contains(qualifier) {
        return Ok(());
    }

    let namespace = imported_namespaces.get(qualifier).ok_or_else(|| {
        ProjectError(format!(
            "namespace `{}` is not imported in `{}`",
            qualifier, current_file
        ))
    })?;

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
    Ok(())
}

fn import_error(error: LibraryImportError, current_file: &str) -> ProjectError {
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

fn normalize_manifest_path(path: &str) -> String {
    PathBuf::from(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn display_path(path: &str) -> String {
    path.to_string()
}

fn assign_target_root_name(target: &AssignTarget) -> &str {
    match target {
        AssignTarget::Variable(identifier) => &identifier.name,
        AssignTarget::Field { target, .. } => assign_target_root_name(target),
    }
}
