use std::collections::{BTreeSet, HashMap};

use crate::ast::{Block, Statement};
use crate::library::ImportedNamespace;

use super::super::ProjectError;
use super::expressions::resolve_expression;
use super::functions::FunctionOrigin;
use super::utils::assign_target_root_name;

pub fn resolve_block(
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
    statement: &mut crate::ast::IfStatement,
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
    statement: &mut crate::ast::WhileStatement,
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
    statement: &mut crate::ast::ForStatement,
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
