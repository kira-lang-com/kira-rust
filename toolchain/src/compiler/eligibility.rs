use std::collections::HashMap;

use crate::ast::syntax::{
    AssignTarget, BinaryOperator, Expression, ExpressionKind, ForStatement, FunctionDefinition,
    Literal, Statement,
};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::{calls::direct_callee_name, BuiltinFunction, CompileError, FunctionSignature};

#[derive(Clone, Copy)]
struct LocalBinding {
    type_id: TypeId,
}

#[derive(Clone, Copy)]
struct ExpressionProfile {
    type_id: TypeId,
    native_eligible: bool,
}

pub(super) fn is_native_eligible(
    function: &FunctionDefinition,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<bool, CompileError> {
    let signature = signatures
        .get(&function.name.name)
        .ok_or_else(|| CompileError(format!("missing signature for `{}`", function.name.name)))?;

    if !signature
        .params
        .iter()
        .copied()
        .all(|type_id| type_is_native_eligible(types, type_id))
    {
        return Ok(false);
    }

    if !type_is_native_eligible(types, signature.return_type) {
        return Ok(false);
    }

    let mut locals = HashMap::new();
    for (slot, parameter) in function.params.iter().enumerate() {
        let _ = slot;
        locals.insert(
            parameter.name.name.clone(),
            LocalBinding {
                type_id: signature.params[slot],
            },
        );
    }

    for statement in &function.body.statements {
        if !analyze_statement(statement, &mut locals, types, signatures, builtins, 0)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn analyze_statement(
    statement: &Statement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    loop_depth: usize,
) -> Result<bool, CompileError> {
    match statement {
        Statement::Let(statement) => {
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
        Statement::While(statement) => {
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

fn analyze_expression(
    expression: &Expression,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    expected_type: Option<TypeId>,
) -> Result<ExpressionProfile, CompileError> {
    match &expression.kind {
        ExpressionKind::Literal(Literal::Bool(_)) => Ok(ExpressionProfile {
            type_id: types.bool(),
            native_eligible: true,
        }),
        ExpressionKind::Literal(Literal::Integer(_)) => Ok(ExpressionProfile {
            type_id: types.int(),
            native_eligible: true,
        }),
        ExpressionKind::Literal(Literal::Float(_)) => Ok(ExpressionProfile {
            type_id: types.float(),
            native_eligible: true,
        }),
        ExpressionKind::Literal(Literal::String(_)) => Ok(ExpressionProfile {
            type_id: types
                .resolve_named("string")
                .expect("built-in string type must exist"),
            native_eligible: true,
        }),
        ExpressionKind::ArrayLiteral(elements) => {
            let expected_element = expected_type.and_then(|type_id| match types.get(type_id) {
                KiraType::Array(element) => Some(*element),
                _ => None,
            });

            let mut element_type = expected_element;
            let mut native_eligible = true;
            for element in elements {
                let profile =
                    analyze_expression(element, locals, types, signatures, builtins, element_type)?;
                native_eligible &= profile.native_eligible;
                match element_type {
                    Some(expected_element) => {
                        if !types.is_assignable(expected_element, profile.type_id) {
                            return Err(CompileError(format!(
                                "array literal element has type {:?}, expected {:?}",
                                types.get(profile.type_id),
                                types.get(expected_element)
                            )));
                        }
                    }
                    None => element_type = Some(profile.type_id),
                }
            }

            let Some(element_type) = element_type else {
                return Err(CompileError(
                    "cannot infer type of empty array literal".to_string(),
                ));
            };
            let array_type = types.register_array(element_type);
            Ok(ExpressionProfile {
                type_id: array_type,
                native_eligible: native_eligible && type_is_native_eligible(types, array_type),
            })
        }
        ExpressionKind::StructLiteral { name, fields } => {
            analyze_struct_literal(name, fields, locals, types, signatures, builtins)
        }
        ExpressionKind::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .ok_or_else(|| CompileError(format!("unknown variable `{}`", identifier.name)))?;
            Ok(ExpressionProfile {
                type_id: binding.type_id,
                native_eligible: type_is_native_eligible(types, binding.type_id),
            })
        }
        ExpressionKind::Member { target, field } => {
            analyze_member_expression(target, field, locals, types, signatures, builtins)
        }
        ExpressionKind::Index { target, index } => {
            let target_profile =
                analyze_expression(target, locals, types, signatures, builtins, None)?;
            let index_profile = analyze_expression(
                index,
                locals,
                types,
                signatures,
                builtins,
                Some(types.int()),
            )?;
            if index_profile.type_id != types.int() {
                return Err(CompileError("array indices must be `int`".to_string()));
            }
            match types.get(target_profile.type_id) {
                KiraType::Array(element_type) => Ok(ExpressionProfile {
                    type_id: *element_type,
                    native_eligible: target_profile.native_eligible
                        && index_profile.native_eligible
                        && type_is_native_eligible(types, *element_type),
                }),
                _ => Err(CompileError("indexing requires an array value".to_string())),
            }
        }
        ExpressionKind::Call { callee, arguments } => {
            if let Some(profile) =
                analyze_special_call(callee, arguments, locals, types, signatures, builtins)?
            {
                return Ok(profile);
            }

            let callee_name = direct_callee_name(callee)?;
            let signature = signatures
                .get(&callee_name)
                .ok_or_else(|| CompileError(format!("unknown function `{callee_name}`")))?;

            if arguments.len() != signature.params.len() {
                return Err(CompileError(format!(
                    "function `{callee_name}` expects {} arguments but got {}",
                    signature.params.len(),
                    arguments.len()
                )));
            }

            let builtin_native = builtins
                .get(&callee_name)
                .map(|builtin| builtin.backend == super::BackendKind::Native)
                .unwrap_or(true);

            let mut native_eligible = builtin_native && type_is_native_eligible(types, signature.return_type);
            if callee_name != "printIn" {
                native_eligible &= signature
                    .params
                    .iter()
                    .copied()
                    .all(|type_id| type_is_native_eligible(types, type_id));
            }

            for (index, argument) in arguments.iter().enumerate() {
                let expected = signature.params[index];
                let profile = analyze_expression(
                    argument,
                    locals,
                    types,
                    signatures,
                    builtins,
                    Some(expected),
                )?;
                if !types.is_assignable(expected, profile.type_id) {
                    return Err(CompileError(format!(
                        "argument {} for `{}` has type {:?}, expected {:?}",
                        index,
                        callee_name,
                        types.get(profile.type_id),
                        types.get(expected)
                    )));
                }
                native_eligible &= profile.native_eligible;
            }

            Ok(ExpressionProfile {
                type_id: signature.return_type,
                native_eligible,
            })
        }
        ExpressionKind::Range { .. } => Err(CompileError(
            "range expressions can only be used as `for` loop iterables".to_string(),
        )),
        ExpressionKind::Cast { target, expr } => {
            let profile = analyze_expression(expr, locals, types, signatures, builtins, None)?;
            let target_type = types
                .resolve_named(&target.name)
                .ok_or_else(|| CompileError(format!("unknown cast target `{}`", target.name)))?;

            if target_type != types.float() {
                return Err(CompileError(format!(
                    "unsupported cast target `{}`",
                    target.name
                )));
            }

            if profile.type_id != types.int() && profile.type_id != types.float() {
                return Err(CompileError(
                    "`float()` can only convert from `int` or `float`".to_string(),
                ));
            }

            Ok(ExpressionProfile {
                type_id: types.float(),
                native_eligible: profile.native_eligible,
            })
        }
        ExpressionKind::Unary { expr, .. } => {
            let profile = analyze_expression(expr, locals, types, signatures, builtins, None)?;
            if !is_numeric_type(types, profile.type_id) {
                return Err(CompileError(
                    "unary negation currently requires an `int` or `float` operand".to_string(),
                ));
            }

            Ok(ExpressionProfile {
                type_id: profile.type_id,
                native_eligible: profile.native_eligible,
            })
        }
        ExpressionKind::Binary { left, right, .. } => {
            let left = analyze_expression(left, locals, types, signatures, builtins, None)?;
            let right = analyze_expression(right, locals, types, signatures, builtins, None)?;
            let type_id =
                analyze_binary_result_type(types, left.type_id, right.type_id, expression)?;
            Ok(ExpressionProfile {
                type_id,
                native_eligible: left.native_eligible
                    && right.native_eligible
                    && type_is_native_eligible(types, type_id),
            })
        }
    }
}

fn analyze_special_call(
    callee: &Expression,
    arguments: &[Expression],
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<Option<ExpressionProfile>, CompileError> {
    let ExpressionKind::Member { target, field } = &callee.kind else {
        return Ok(None);
    };
    let ExpressionKind::Variable(identifier) = &target.kind else {
        return Ok(None);
    };

    let target_name = &identifier.name;
    let member_name = &field.name;
    let Some(binding) = locals.get(target_name).copied() else {
        return Ok(None);
    };

    match (types.get(binding.type_id), member_name.as_str()) {
        (KiraType::Array(element_type), "append") => {
            let element_type = *element_type;
            if arguments.len() != 1 {
                return Err(CompileError(format!(
                    "`{}.append` expects 1 argument but got {}",
                    target_name,
                    arguments.len()
                )));
            }
            let profile = analyze_expression(
                &arguments[0],
                locals,
                types,
                signatures,
                builtins,
                Some(element_type),
            )?;
            if !types.is_assignable(element_type, profile.type_id) {
                return Err(CompileError(format!(
                    "array append expected {:?}, got {:?}",
                    types.get(element_type),
                    types.get(profile.type_id)
                )));
            }
            Ok(Some(ExpressionProfile {
                type_id: types.unit(),
                native_eligible: profile.native_eligible,
            }))
        }
        (KiraType::Array(_), _) => Err(CompileError(format!(
            "unknown array method `{}`",
            member_name
        ))),
        _ => Ok(None),
    }
}

fn analyze_assignment(
    statement: &crate::ast::syntax::AssignStatement,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<bool, CompileError> {
    match &statement.target {
        AssignTarget::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .copied()
                .ok_or_else(|| CompileError(format!("unknown local `{}`", identifier.name)))?;
            let profile = analyze_expression(
                &statement.value,
                locals,
                types,
                signatures,
                builtins,
                Some(binding.type_id),
            )?;
            if !types.is_assignable(binding.type_id, profile.type_id) {
                return Err(CompileError(format!(
                    "cannot assign value of type {:?} to local `{}`",
                    types.get(profile.type_id),
                    identifier.name
                )));
            }
            Ok(profile.native_eligible)
        }
        AssignTarget::Field { .. } => {
            let (_, field_type) = resolve_assign_target(&statement.target, locals, types)?;
            let profile = analyze_expression(
                &statement.value,
                locals,
                types,
                signatures,
                builtins,
                Some(field_type),
            )?;
            if !types.is_assignable(field_type, profile.type_id) {
                return Err(CompileError(format!(
                    "cannot assign value of type {:?} to field of type {:?}",
                    types.get(profile.type_id),
                    types.get(field_type)
                )));
            }
            Ok(profile.native_eligible && type_is_native_eligible(types, field_type))
        }
    }
}

fn resolve_assign_target(
    target: &AssignTarget,
    locals: &HashMap<String, LocalBinding>,
    types: &TypeSystem,
) -> Result<(TypeId, TypeId), CompileError> {
    match target {
        AssignTarget::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .copied()
                .ok_or_else(|| CompileError(format!("unknown local `{}`", identifier.name)))?;
            Ok((binding.type_id, binding.type_id))
        }
        AssignTarget::Field { target, field, .. } => {
            let (_, owner_type) = resolve_assign_target(target, locals, types)?;
            match types.get(owner_type) {
                KiraType::Struct(struct_type) => {
                    let (_, field_type) = types
                        .struct_field(owner_type, &field.name)
                        .ok_or_else(|| {
                            CompileError(format!("{} has no field '{}'", struct_type.name, field.name))
                        })?;
                    Ok((owner_type, field_type))
                }
                KiraType::Array(_) => Err(CompileError(format!(
                    "cannot assign to array member `{}`",
                    field.name
                ))),
                _ => Err(CompileError(format!(
                    "type `{}` has no assignable field `{}`",
                    types.type_name(owner_type),
                    field.name
                ))),
            }
        }
    }
}

fn analyze_struct_literal(
    name: &crate::ast::syntax::TypeSyntax,
    fields: &[crate::ast::syntax::StructLiteralField],
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
    let struct_type = types
        .resolve_named(&name.name)
        .ok_or_else(|| CompileError(format!("unknown type `{}`", name.name)))?;
    let declared_fields = types
        .struct_fields(struct_type)
        .ok_or_else(|| CompileError(format!("`{}` is not a struct type", name.name)))?
        .to_vec();

    let mut provided = HashMap::new();
    for field in fields {
        if provided.insert(field.name.name.clone(), &field.value).is_some() {
            return Err(CompileError(format!(
                "struct literal for `{}` sets field `{}` more than once",
                name.name, field.name.name
            )));
        }
    }

    let mut native_eligible = type_is_native_eligible(types, struct_type);
    for field in fields {
        if !declared_fields.iter().any(|declared| declared.name == field.name.name) {
            return Err(CompileError(format!(
                "{} has no field '{}'",
                name.name, field.name.name
            )));
        }
    }

    for declared in &declared_fields {
        let value = provided.get(&declared.name).ok_or_else(|| {
            CompileError(format!(
                "struct literal for `{}` is missing field `{}`",
                name.name, declared.name
            ))
        })?;
        let profile = analyze_expression(
            value,
            locals,
            types,
            signatures,
            builtins,
            Some(declared.type_id),
        )?;
        if !types.is_assignable(declared.type_id, profile.type_id) {
            return Err(CompileError(format!(
                "field `{}` on `{}` has type {:?}, expected {:?}",
                declared.name,
                name.name,
                types.get(profile.type_id),
                types.get(declared.type_id)
            )));
        }
        native_eligible &= profile.native_eligible;
    }

    Ok(ExpressionProfile {
        type_id: struct_type,
        native_eligible,
    })
}

fn analyze_member_expression(
    target: &Expression,
    field: &crate::ast::syntax::Identifier,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
) -> Result<ExpressionProfile, CompileError> {
    let target_profile = analyze_expression(target, locals, types, signatures, builtins, None)?;

    match types.get(target_profile.type_id) {
        KiraType::Array(_) if field.name == "length" => Ok(ExpressionProfile {
            type_id: types.int(),
            native_eligible: target_profile.native_eligible,
        }),
        KiraType::Array(_) => Err(CompileError(format!(
            "unknown array member `{}`",
            field.name
        ))),
        KiraType::Struct(struct_type) => {
            let (_, field_type) = types.struct_field(target_profile.type_id, &field.name).ok_or_else(|| {
                CompileError(format!("{} has no field '{}'", struct_type.name, field.name))
            })?;
            Ok(ExpressionProfile {
                type_id: field_type,
                native_eligible: target_profile.native_eligible
                    && type_is_native_eligible(types, field_type),
            })
        }
        _ => Err(CompileError(format!(
            "type `{}` has no field `{}`",
            types.type_name(target_profile.type_id),
            field.name
        ))),
    }
}

fn analyze_binary_result_type(
    types: &TypeSystem,
    left_type: TypeId,
    right_type: TypeId,
    expression: &Expression,
) -> Result<TypeId, CompileError> {
    let ExpressionKind::Binary { op, .. } = &expression.kind else {
        unreachable!()
    };

    match op {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide => {
            if left_type != right_type || !is_numeric_type(types, left_type) {
                return Err(CompileError(
                    "numeric operations require operands of the same `int` or `float` type"
                        .to_string(),
                ));
            }
            Ok(left_type)
        }
        BinaryOperator::Modulo => {
            if left_type != types.int() || right_type != types.int() {
                return Err(CompileError(
                    "modulo currently requires `int` operands".to_string(),
                ));
            }
            Ok(types.int())
        }
        BinaryOperator::Less
        | BinaryOperator::Greater
        | BinaryOperator::LessEqual
        | BinaryOperator::GreaterEqual => {
            if left_type != right_type || !is_numeric_type(types, left_type) {
                return Err(CompileError(
                    "ordered comparison requires operands of the same `int` or `float` type"
                        .to_string(),
                ));
            }
            Ok(types.bool())
        }
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            if left_type != right_type || !is_equatable_type(types, left_type) {
                return Err(CompileError(
                    "comparison requires operands of the same comparable type".to_string(),
                ));
            }
            Ok(types.bool())
        }
    }
}

fn type_is_native_eligible(types: &TypeSystem, type_id: TypeId) -> bool {
    match types.get(type_id) {
        KiraType::Dynamic => true,
        KiraType::Array(element) => type_is_native_eligible(types, *element),
        KiraType::Struct(_) => types
            .struct_fields(type_id)
            .into_iter()
            .flatten()
            .all(|field| type_is_native_eligible(types, field.type_id)),
        _ => true,
    }
}

fn is_numeric_type(types: &TypeSystem, type_id: TypeId) -> bool {
    matches!(types.get(type_id), KiraType::Int | KiraType::Float)
}

fn is_equatable_type(types: &TypeSystem, type_id: TypeId) -> bool {
    matches!(
        types.get(type_id),
        KiraType::Bool
            | KiraType::Int
            | KiraType::Float
            | KiraType::String
            | KiraType::Array(_)
            | KiraType::Struct(_)
    )
}
