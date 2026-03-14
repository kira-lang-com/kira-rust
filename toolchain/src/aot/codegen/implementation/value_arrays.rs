// Array value operations

use inkwell::values::PointerValue;

use crate::compiler::Chunk;
use crate::runtime::type_system::{KiraType, TypeId};

use super::super::super::error::AotError;
use super::super::super::stack::StackState;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn emit_build_array(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        type_id: TypeId,
        element_count: usize,
    ) -> Result<(), AotError> {
        let KiraType::Array(element_type) = self.compiled.types.get(type_id) else {
            return Err(AotError("BuildArray target type is not an array".to_string()));
        };

        let array_handle = self.call_runtime_new_array()?;

        let mut elements = Vec::with_capacity(element_count);
        for offset in 0..element_count {
            let slot = depth - 1 - offset;
            let value = self.load_stack(stack_slots, slot, *element_type, "arr_elem")?;
            let boxed = self.box_value_as_handle(*element_type, value)?;
            elements.push(boxed);
        }
        elements.reverse();
        for boxed in elements {
            self.call_runtime_array_push(array_handle, boxed)?;
        }

        self.store_stack(stack_slots, depth - element_count, type_id, array_handle.into())
    }

    pub(super) fn emit_array_length(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        state: &StackState,
    ) -> Result<(), AotError> {
        let target_type = *state
            .stack
            .last()
            .ok_or_else(|| AotError("stack underflow on array length".to_string()))?;
        let array_handle = self.load_stack(stack_slots, depth - 1, target_type, "array")?;
        let len = self.call_runtime_array_length(array_handle.into_pointer_value())?;
        self.store_stack(stack_slots, depth - 1, self.compiled.types.int(), len.into())
    }

    pub(super) fn emit_array_index(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        state: &StackState,
    ) -> Result<(), AotError> {
        let index_value = self.load_stack(stack_slots, depth - 1, self.compiled.types.int(), "idx")?;
        let array_type = *state
            .stack
            .get(depth - 2)
            .ok_or_else(|| AotError("stack underflow on array index".to_string()))?;
        let array_handle = self.load_stack(stack_slots, depth - 2, array_type, "array")?;
        let KiraType::Array(element_type) = self.compiled.types.get(array_type) else {
            return Err(AotError("array index expected array target".to_string()));
        };
        let handle = self.call_runtime_array_index(
            array_handle.into_pointer_value(),
            index_value.into_int_value(),
        )?;
        let element = self.unbox_handle_if_needed(*element_type, handle)?;
        self.store_stack(stack_slots, depth - 2, *element_type, element)
    }

    pub(super) fn emit_array_append_local(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        chunk: &Chunk,
        locals: &[PointerValue<'ctx>],
        state: &StackState,
        local_index: usize,
    ) -> Result<(), AotError> {
        let value_type = *state
            .stack
            .last()
            .ok_or_else(|| AotError("stack underflow on array append".to_string()))?;
        let value = self.load_stack(stack_slots, depth - 1, value_type, "append_value")?;

        let array_type = *chunk
            .local_types
            .get(local_index)
            .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
        let KiraType::Array(element_type) = self.compiled.types.get(array_type) else {
            return Err(AotError("array append expected array local".to_string()));
        };
        if *element_type != value_type {
            return Err(AotError("array append type mismatch".to_string()));
        }

        let boxed = self.box_value_as_handle(value_type, value)?;
        let local = *locals
            .get(local_index)
            .ok_or_else(|| AotError(format!("missing local slot {local_index}")))?;
        let array_handle = self
            .load_typed_ptr(local, array_type, "array_local")?
            .into_pointer_value();
        self.call_runtime_array_append(array_handle, boxed)
    }
}
