use std::collections::HashMap;

use crate::ast::syntax::{ExpressionKind, ForStatement, Statement};
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeSystem};

use super::expressions::{analyze_assignment, analyze_expression};
use super::types::{type_is_native_eligible, LocalBinding};

pub fn analyze_statement(
    statement: &Statement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    loop_depth: usize,
) -> Result<bool, CompileError> {
    match statement {
        Statement::Let(statement) => {
            analyze_let_statement(statement, locals, types, signatures, builtins)
        }
        Statement::Assign(statement) => {
            analyze_assignment(statement, locals, types, signatures, builtins)
        }
        Statement::Expression(statement) => {
            let profile = analyze_expression(
                &statement.expression,
                locals,
                types,
                signatures,
                builtins,
                None,
            )?;
            Ok(profile.native_eligible)
        }
        Statement::Return(statement) => {
            let profile = analyze_expression(
                &statement.expression,
                locals,
                types,
                signatures,
                builtins,
                None,
            )?;
            Ok(profile.native_eligible)
        }
        Statement::If(statement) => {
            analyze_if_statement(statement, locals, types, signatures, builtins, loop_depth)
        }
        Statement::While(statement) => {
            analyze_while_statement(statement, locals, types, signatures, builtins, loop_depth)
        }
        Statement::For(statement) => {
            analyze_for_statement(statement, locals, types, signatures, builtins, loop_depth)
        }
        Statement::Break(_) | Statement::Continue(_) => {
            if loop_depth == 0 {
                return Err(CompileError(
                    "loop control can only be used inside a loop".to_string(),
                ));
            }
            Ok(true)
        }
    }
}

fn analyze_let_statement(
    statement: &crate::ast::syntax::LetStatement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<bool, CompileError> {
    let declared = statement
        .type_ann
        .as_ref()
        .map(|type_name| {
            types.ensure_named(&type_name.name).ok_or_else(|| {
                CompileError(format!(
                    "unknown type `{}` on local `{}`",
                    type_name.name, statement.name.name
                ))
            })
        })
        .transpose()?;
    let profile = analyze_expression(
        &statement.value,
        locals,
        types,
        signatures,
        builtins,
        declared,
    )?;
    let local_type = declared.unwrap_or(profile.type_id);

    if !types.is_assignable(local_type, profile.type_id) {
        return Err(CompileError(format!(
            "cannot assign value of type {:?} to local `{}`",
            types.get(profile.type_id),
            statement.name.name
        )));
    }

    if !profile.native_eligible || !type_is_native_eligible(types, local_type) {
        return Ok(false);
    }

    locals.insert(
        statement.name.name.clone(),
        LocalBinding {
            type_id: local_type,
        },
    );
    Ok(true)
}

fn analyze_if_statement(
    statement: &crate::ast::syntax::IfStatement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    loop_depth: usize,
) -> Result<bool, CompileError> {
    let condition = analyze_expression(
        &statement.condition,
        locals,
        types,
        signatures,
        builtins,
        Some(types.bool()),
    )?;
    if condition.type_id != types.bool() {
        return Err(CompileError(
            "`if` conditions must evaluate to `bool`".to_string(),
        ));
    }

    if !condition.native_eligible {
        return Ok(false);
    }

    let mut then_locals = locals.clone();
    for statement in &statement.then_block.statements {
        if !analyze_statement(
            statement,
            &mut then_locals,
            types,
            signatures,
            builtins,
            loop_depth,
        )? {
            return Ok(false);
        }
    }

    if let Some(else_block) = &statement.else_block {
        let mut else_locals = locals.clone();
        for statement in &else_block.statements {
            if !analyze_statement(
                statement,
                &mut else_locals,
                types,
                signatures,
                builtins,
                loop_depth,
            )? {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn analyze_while_statement(
    statement: &crate::ast::syntax::WhileStatement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    loop_depth: usize,
) -> Result<bool, CompileError> {
    let condition = analyze_expression(
        &statement.condition,
        locals,
        types,
        signatures,
        builtins,
        Some(types.bool()),
    )?;
    if condition.type_id != types.bool() {
        return Err(CompileError(
            "`while` conditions must evaluate to `bool`".to_string(),
        ));
    }

    if !condition.native_eligible {
        return Ok(false);
    }

    let mut body_locals = locals.clone();
    for statement in &statement.body.statements {
        if !analyze_statement(
            statement,
            &mut body_locals,
            types,
            signatures,
            builtins,
            loop_depth + 1,
        )? {
            return Ok(false);
        }
    }

    Ok(true)
}


fn analyze_for_statement(
    statement: &ForStatement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    loop_depth: usize,
) -> Result<bool, CompileError> {
    let binding_type = match &statement.iterable.kind {
        ExpressionKind::Range {
            start,
            end,
            inclusive: _,
        } => {
            let start_profile = analyze_expression(
                start,
                locals,
                types,
                signatures,
                builtins,
                Some(types.int()),
            )?;
            let end_profile =
                analyze_expression(end, locals, types, signatures, builtins, Some(types.int()))?;
            if start_profile.type_id != types.int() || end_profile.type_id != types.int() {
                return Err(CompileError("ranges require `int` bounds".to_string()));
            }
            if !start_profile.native_eligible || !end_profile.native_eligible {
                return Ok(false);
            }
            types.int()
        }
        _ => {
            let iterable = analyze_expression(
                &statement.iterable,
                locals,
                types,
                signatures,
                builtins,
                None,
            )?;
            let KiraType::Array(element_type) = types.get(iterable.type_id) else {
                return Err(CompileError(
                    "`for` loops currently require an array or range iterable".to_string(),
                ));
            };
            if !iterable.native_eligible {
                return Ok(false);
            }
            *element_type
        }
    };

    let mut body_locals = locals.clone();
    body_locals.insert(
        statement.binding.name.clone(),
        LocalBinding {
            type_id: binding_type,
        },
    );
    for statement in &statement.body.statements {
        if !analyze_statement(
            statement,
            &mut body_locals,
            types,
            signatures,
            builtins,
            loop_depth + 1,
        )? {
            return Ok(false);
        }
    }

    Ok(type_is_native_eligible(types, binding_type))
}
