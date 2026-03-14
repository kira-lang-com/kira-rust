use std::collections::HashMap;

use crate::ast::{ExpressionKind, ForStatement};
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeSystem};

use super::super::expressions::analyze_expression;
use super::super::types::{type_is_native_eligible, LocalBinding};
use super::analyze_statement;

pub fn analyze_if_statement(
    statement: &crate::ast::IfStatement,
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

pub fn analyze_while_statement(
    statement: &crate::ast::WhileStatement,
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

pub fn analyze_for_statement(
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
