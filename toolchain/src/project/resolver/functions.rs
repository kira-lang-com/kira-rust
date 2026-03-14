use std::collections::{BTreeSet, HashMap};

use crate::ast::syntax::{Block, Expression, ExpressionKind, FunctionDefinition, Statement};
use crate::library::ImportedNamespace;

use super::super::ProjectError;
use super::imports::resolve_imported_callee;
use super::utils::assign_target_root_name;

#[derive(Debug, Clone)]
pub struct FunctionOrigin {
    pub file_name: String,
}

pub fn resolve_function_body(
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
                resolve_if_statement(
                    statement,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Statement::While(statement) => {
                resolve_while_statement(
                    statement,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Statement::For(statement) => {
                resolve_for_statement(
                    statement,
                    current_file,
                    global_functions,
                    builtins,
                    local_modules,
                    imported_namespaces,
                    scope,
                )?;
            }
            Statement::Break(_) | Statement::Continue(_) => {}
        }
    }

    Ok(())
}

fn resolve_if_statement(
    statement: &mut crate::ast::syntax::IfStatement,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &mut BTreeSet<String>,
) -> Result<(), ProjectError> {
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
    Ok(())
}

fn resolve_while_statement(
    statement: &mut crate::ast::syntax::WhileStatement,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &mut BTreeSet<String>,
) -> Result<(), ProjectError> {
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
    )
}

fn resolve_for_statement(
    statement: &mut crate::ast::syntax::ForStatement,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
    scope: &mut BTreeSet<String>,
) -> Result<(), ProjectError> {
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
    )
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
