// Struct value operations

use inkwell::values::PointerValue;

use crate::compiler::Chunk;
use crate::runtime::type_system::{KiraType, TypeId};

use super::super::super::error::AotError;
use super::super::super::stack::StackState;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn emit_build_struct(
        &mut self,
        ctx_arg: PointerValue<'ctx>,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        type_id: TypeId,
        field_count: usize,
    ) -> Result<(), AotError> {
        let KiraType::Struct(struct_type) = self.compiled.types.get(type_id) else {
            return Err(AotError("BuildStruct target type is not a struct".to_string()));
        };
        if struct_type.fields.len() != field_count {
            return Err(AotError(format!(
                "struct `{}` expects {} fields but bytecode provided {}",
                struct_type.name,
                struct_type.fields.len(),
                field_count
            )));
        }

        let struct_handle = self.call_runtime_new_struct(ctx_arg, type_id)?;

        let mut values = Vec::with_capacity(field_count);
        for offset in 0..field_count {
            let field_index = field_count - 1 - offset;
            let slot = depth - 1 - offset;
            let field_type = struct_type
                .fields
                .get(field_index)
                .ok_or_else(|| AotError("invalid struct field index".to_string()))?
                .type_id;
            let value = self.load_stack(stack_slots, slot, field_type, "field")?;
            let boxed = self.box_value_as_handle(field_type, value)?;
            values.push((field_index, boxed));
        }
        values.reverse();

        for (field_index, boxed) in values {
            self.call_runtime_struct_set_field(struct_handle, field_index, boxed)?;
        }

        self.store_stack(stack_slots, depth - field_count, type_id, struct_handle.into())
    }

    pub(super) fn emit_struct_field(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        state: &StackState,
        field_index: usize,
    ) -> Result<(), AotError> {
        let target_type = *state
            .stack
            .last()
            .ok_or_else(|| AotError("stack underflow on struct field".to_string()))?;
        let struct_handle = self.load_stack(stack_slots, depth - 1, target_type, "struct")?;
        let field_type = self
            .compiled
            .types
            .struct_fields(target_type)
            .and_then(|fields| fields.get(field_index))
            .map(|field| field.type_id)
            .ok_or_else(|| AotError(format!("invalid struct field index {}", field_index)))?;
        let handle =
            self.call_runtime_struct_field(struct_handle.into_pointer_value(), field_index)?;
        let field_value = self.unbox_handle_if_needed(field_type, handle)?;
        self.store_stack(stack_slots, depth - 1, field_type, field_value)
    }

    pub(super) fn emit_store_local_field(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        chunk: &Chunk,
        locals: &[PointerValue<'ctx>],
        index: usize,
        local: usize,
        path: &[usize],
    ) -> Result<(), AotError> {
        let value_type = *chunk
            .local_types
            .get(depth - 1)
            .ok_or_else(|| AotError("stack underflow on field store".to_string()))?;
        let value = self.load_stack(stack_slots, depth - 1, value_type, "field_value")?;
        let boxed = self.box_value_as_handle(value_type, value)?;
        let target_type = *chunk
            .local_types
            .get(local)
            .ok_or_else(|| AotError(format!("invalid local index {local}")))?;
        let target_local = *locals
            .get(local)
            .ok_or_else(|| AotError(format!("missing local slot {local}")))?;
        let target_handle =
            self.load_typed_ptr(target_local, target_type, &format!("local_{local}_target"))?
                .into_pointer_value();
        let (path_ptr, path_len) = self.const_usize_path(path, &format!("path_{index}"))?;
        self.call_runtime_store_struct_field(target_handle, path_ptr, path_len, boxed)
    }
}
