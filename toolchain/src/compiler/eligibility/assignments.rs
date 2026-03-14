use std::collections::HashMap;

use crate::ast::{AssignStatement, AssignTarget};
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::expressions::analyze_expression;
use super::types::{type_is_native_eligible, LocalBinding};

pub fn analyze_assignment(
    statement: &AssignStatement,
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
