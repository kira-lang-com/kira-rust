// Value operations: arrays, structs, boxing/unboxing

use inkwell::values::{BasicValueEnum, PointerValue};

use crate::compiler::Chunk;
use crate::runtime::type_system::{KiraType, TypeId};
use crate::runtime::Value;

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

    pub(super) fn is_value_handle_type(&self, type_id: TypeId) -> bool {
        matches!(
            self.compiled.types.get(type_id),
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_)
        )
    }

    pub(super) fn llvm_const(&mut self, value: &Value) -> Result<(TypeId, BasicValueEnum<'ctx>), AotError> {
        Ok(match value {
            Value::Bool(b) => (self.compiled.types.bool(), self.context.bool_type().const_int(*b as u64, false).into()),
            Value::Int(i) => (self.compiled.types.int(), self.context.i64_type().const_int(*i as u64, true).into()),
            Value::Float(f) => (self.compiled.types.float(), self.context.f64_type().const_float(f.0).into()),
            Value::String(s) => {
                let handle = self.const_string_handle(s)?;
                (
                    self.compiled
                        .types
                        .resolve_named("string")
                        .ok_or_else(|| AotError("missing string type".to_string()))?,
                    handle.into(),
                )
            }
            Value::Unit => {
                return Err(AotError(
                    "unit constants are not supported as stack values in AOT".to_string(),
                ))
            }
            Value::Array(_) | Value::Struct(_) => {
                return Err(AotError("aggregate constants are not supported in AOT".to_string()))
            }
        })
    }

    fn const_string_handle(&mut self, value: &str) -> Result<PointerValue<'ctx>, AotError> {
        let global = self
            .builder
            .build_global_string_ptr(value, "kira_str")
            .map_err(|e| AotError(e.to_string()))?;
        let bytes_ptr = global.as_pointer_value();
        let len = self
            .ptr_sized_int_type()
            .const_int(value.as_bytes().len() as u64, false);
        let make = self.declare_runtime_make_string();
        let call_site = self
            .builder
            .build_call(make, &[bytes_ptr.into(), len.into()], "make_string")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing string handle result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(super) fn box_value_as_handle(
        &mut self,
        type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, AotError> {
        Ok(match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let f = self.declare_runtime_box_int();
                let call_site = self
                    .builder
                    .build_call(f, &[value.into()], "box_int")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing box result".to_string()))?
                    .into_pointer_value()
            }
            KiraType::Bool => {
                let f = self.declare_runtime_box_bool();
                let call_site = self
                    .builder
                    .build_call(f, &[value.into()], "box_bool")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing box result".to_string()))?
                    .into_pointer_value()
            }
            KiraType::Float => {
                let f = self.declare_runtime_box_float();
                let call_site = self
                    .builder
                    .build_call(f, &[value.into()], "box_float")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing box result".to_string()))?
                    .into_pointer_value()
            }
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                value.into_pointer_value()
            }
            KiraType::Opaque(_) => {
                return Err(AotError(
                    "opaque handles cannot be boxed into Kira runtime values".to_string(),
                ))
            }
            other => return Err(AotError(format!("cannot box type {:?}", other))),
        })
    }

    pub(super) fn unbox_handle_if_needed(
        &mut self,
        type_id: TypeId,
        handle: PointerValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        Ok(match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let f = self.declare_runtime_unbox_int();
                let call_site = self
                    .builder
                    .build_call(f, &[handle.into()], "unbox_int")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing unbox result".to_string()))?
            }
            KiraType::Bool => {
                let f = self.declare_runtime_unbox_bool();
                let call_site = self
                    .builder
                    .build_call(f, &[handle.into()], "unbox_bool")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing unbox result".to_string()))?
            }
            KiraType::Float => {
                let f = self.declare_runtime_unbox_float();
                let call_site = self
                    .builder
                    .build_call(f, &[handle.into()], "unbox_float")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing unbox result".to_string()))?
            }
            _ => handle.into(),
        })
    }

    pub(super) fn clone_value_handle(&mut self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, AotError> {
        let clone = self.declare_runtime_clone_value();
        let call_site = self
            .builder
            .build_call(clone, &[value.into()], "clone")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing clone result".to_string()))
    }

    pub(super) fn const_usize_path(
        &mut self,
        path: &[usize],
        name: &str,
    ) -> Result<(PointerValue<'ctx>, inkwell::values::IntValue<'ctx>), AotError> {
        let usize_ty = self.ptr_sized_int_type();
        let elements = path
            .iter()
            .map(|value| usize_ty.const_int(*value as u64, false))
            .collect::<Vec<_>>();
        let array = usize_ty.const_array(&elements);
        let global = self.module.add_global(array.get_type(), None, name);
        global.set_initializer(&array);
        global.set_constant(true);
        let ptr = global.as_pointer_value();
        let len = usize_ty.const_int(path.len() as u64, false);
        Ok((ptr, len))
    }
}
