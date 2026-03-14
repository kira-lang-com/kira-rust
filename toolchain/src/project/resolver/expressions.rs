use std::collections::{BTreeSet, HashMap};

use crate::ast::syntax::{Expression, ExpressionKind};
use crate::library::ImportedNamespace;

use super::super::ProjectError;
use super::callees::resolve_callee;
use super::functions::FunctionOrigin;

pub fn resolve_expression(
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
