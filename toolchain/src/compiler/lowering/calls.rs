use std::collections::HashMap;

use crate::ast::{Expression, ExpressionKind, Identifier};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::expressions::lower_expression;
use super::infrastructure::LocalBinding;

pub fn lower_call_expression(
    callee: &Expression,
    arguments: &[Expression],
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<TypeId, CompileError> {
    if let Some(return_type) =
        lower_special_call(callee, arguments, chunk, locals, types, signatures)?
    {
        return Ok(return_type);
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

    for (index, argument) in arguments.iter().enumerate() {
        let expected = signature.params[index];
        let arg_type =
            lower_expression(argument, chunk, locals, types, signatures, Some(expected))?;
        if !types.is_assignable(expected, arg_type) {
            return Err(CompileError(format!(
                "argument {} for `{}` has type {:?}, expected {:?}",
                index,
                callee_name,
                types.get(arg_type),
                types.get(expected)
            )));
        }
    }

    chunk.instructions.push(Instruction::Call {
        function: callee_name,
        arg_count: arguments.len(),
    });
    Ok(signature.return_type)
}

fn lower_special_call(
    callee: &Expression,
    arguments: &[Expression],
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<Option<TypeId>, CompileError> {
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

            let arg_type = lower_expression(
                &arguments[0],
                chunk,
                locals,
                types,
                signatures,
                Some(element_type),
            )?;
            if !types.is_assignable(element_type, arg_type) {
                return Err(CompileError(format!(
                    "array append expected {:?}, got {:?}",
                    types.get(element_type),
                    types.get(arg_type)
                )));
            }

            chunk
                .instructions
                .push(Instruction::ArrayAppendLocal(binding.slot));
            Ok(Some(types.unit()))
        }
        (KiraType::Array(_), _) => Err(CompileError(format!(
            "unknown array method `{}`",
            member_name
        ))),
        _ => Ok(None),
    }
}

pub fn direct_callee_name(callee: &Expression) -> Result<String, CompileError> {
    match &callee.kind {
        ExpressionKind::Variable(identifier) => Ok(identifier.name.clone()),
        ExpressionKind::Member { target, field } => {
            let ExpressionKind::Variable(module) = &target.kind else {
                return Err(CompileError(
                    "qualified calls must be in the form `module.function`".to_string(),
                ));
            };
            Ok(format!("{}.{}", module.name, field.name))
        }
        _ => Err(CompileError(
            "function calls must use a direct function name or qualified name".to_string(),
        )),
    }
}
