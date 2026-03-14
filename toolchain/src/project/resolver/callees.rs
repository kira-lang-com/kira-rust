use std::collections::{BTreeSet, HashMap};

use crate::ast::syntax::{Expression, ExpressionKind};
use crate::library::ImportedNamespace;

use super::super::ProjectError;
use super::functions::FunctionOrigin;
use super::imports::resolve_imported_callee;

pub fn resolve_callee(
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
