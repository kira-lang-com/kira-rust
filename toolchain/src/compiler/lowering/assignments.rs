use std::collections::HashMap;

use crate::ast::syntax::{AssignStatement, AssignTarget};
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

use super::expressions::lower_expression;
use super::types::LocalBinding;

pub fn lower_assignment(
    statement: &AssignStatement,
    chunk: &mut Chunk,
    locals: &HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<(), CompileError> {
    match &statement.target {
        AssignTarget::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .copied()
                .ok_or_else(|| CompileError(format!("unknown local `{}`", identifier.name)))?;
            let value_type = lower_expression(
                &statement.value,
                chunk,
                locals,
                types,
                signatures,
                Some(binding.type_id),
            )?;
            if !types.is_assignable(binding.type_id, value_type) {
                return Err(CompileError(format!(
                    "cannot assign value of type {:?} to local `{}`",
                    types.get(value_type),
                    identifier.name
                )));
            }
            chunk.instructions.push(Instruction::StoreLocal(binding.slot));
            Ok(())
        }
        AssignTarget::Field { .. } => {
            let (binding, path, field_type) = resolve_assign_target(&statement.target, locals, types)?;
            let value_type = lower_expression(
                &statement.value,
                chunk,
                locals,
                types,
                signatures,
                Some(field_type),
            )?;
            if !types.is_assignable(field_type, value_type) {
                return Err(CompileError(format!(
                    "cannot assign value of type {:?} to field of type {:?}",
                    types.get(value_type),
                    types.get(field_type)
                )));
            }

            chunk.instructions.push(Instruction::StoreLocalField {
                local: binding.slot,
                path,
            });
            Ok(())
        }
    }
}

fn resolve_assign_target(
    target: &AssignTarget,
    locals: &HashMap<String, LocalBinding>,
    types: &TypeSystem,
) -> Result<(LocalBinding, Vec<usize>, TypeId), CompileError> {
    match target {
        AssignTarget::Variable(identifier) => {
            let binding = locals
                .get(&identifier.name)
                .copied()
                .ok_or_else(|| CompileError(format!("unknown local `{}`", identifier.name)))?;
            Ok((binding, Vec::new(), binding.type_id))
        }
        AssignTarget::Field { target, field, .. } => {
            let (binding, mut path, owner_type) = resolve_assign_target(target, locals, types)?;
            match types.get(owner_type) {
                KiraType::Struct(struct_type) => {
                    let (field_index, field_type) = types
                        .struct_field(owner_type, &field.name)
                        .ok_or_else(|| {
                            CompileError(format!("{} has no field '{}'", struct_type.name, field.name))
                        })?;
                    path.push(field_index);
                    Ok((binding, path, field_type))
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
