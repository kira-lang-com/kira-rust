// Type system and LLVM type conversions

use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, PointerValue};
use inkwell::AddressSpace;

use crate::compiler::{Chunk, CompiledFunction, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeId};
use crate::runtime::Value;

use crate::aot::error::AotError;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn llvm_function_type(
        &self,
        signature: &FunctionSignature,
    ) -> Result<inkwell::types::FunctionType<'ctx>, AotError> {
        let mut params = vec![self
            .context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .into()];
        for type_id in &signature.params {
            params.push(
                self.llvm_basic_type(*type_id)
                    .ok_or_else(|| {
                        AotError(format!(
                            "LLVM AOT backend does not yet support parameter type {:?}",
                            self.compiled.types.get(*type_id)
                        ))
                    })?
                    .into(),
            );
        }

        match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => Ok(self.context.void_type().fn_type(&params, false)),
            _ => Ok(self
                .llvm_basic_type(signature.return_type)
                .ok_or_else(|| {
                    AotError(format!(
                        "LLVM AOT backend does not yet support return type {:?}",
                        self.compiled.types.get(signature.return_type)
                    ))
                })?
                .fn_type(&params, false)),
        }
    }

    pub(in crate::aot::native) fn llvm_basic_type(&self, type_id: TypeId) -> Option<BasicTypeEnum<'ctx>> {
        match self.compiled.types.get(type_id) {
            KiraType::Int => Some(self.context.i64_type().into()),
            KiraType::Float => Some(self.context.f64_type().into()),
            KiraType::Bool => Some(self.context.bool_type().into()),
            KiraType::String
            | KiraType::Dynamic
            | KiraType::Array(_)
            | KiraType::Struct(_)
            | KiraType::Opaque(_) => {
                Some(self.value_handle_type())
            }
            _ => None,
        }
    }

    pub(in crate::aot::native) fn value_handle_type(&self) -> BasicTypeEnum<'ctx> {
        self.context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .into()
    }

    pub(in crate::aot::native) fn ptr_sized_int_type(&self) -> inkwell::types::IntType<'ctx> {
        let target_data = self.target_machine.get_target_data();
        self.context.ptr_sized_int_type(&target_data, None)
    }

    pub(in crate::aot::native) fn ptr_to_int(&self, ptr: PointerValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        self.builder
            .build_ptr_to_int(ptr, self.ptr_sized_int_type(), "ptrtoint")
            .map_err(|e| AotError(e.to_string()))
    }

    pub(in crate::aot::native) fn load_typed_ptr(
        &self,
        ptr: PointerValue<'ctx>,
        type_id: TypeId,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ty = self.llvm_basic_type(type_id).ok_or_else(|| {
            AotError(format!(
                "LLVM AOT backend does not yet support type {:?} for load",
                self.compiled.types.get(type_id)
            ))
        })?;
        self.builder
            .build_load(ty, ptr, name)
            .map_err(|e| AotError(e.to_string()))
    }

    pub(in crate::aot::native) fn store_ptr(&self, ptr: PointerValue<'ctx>, value: BasicValueEnum<'ctx>) -> Result<(), AotError> {
        self.builder
            .build_store(ptr, value)
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(in crate::aot::native) fn load_stack(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        type_id: TypeId,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        let value = self.load_typed_ptr(ptr, type_id, name)?;
        match self.compiled.types.get(type_id) {
            KiraType::Bool => Ok(value.into_int_value().into()),
            _ => Ok(value),
        }
    }

    pub(in crate::aot::native) fn store_stack(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        _type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        self.store_ptr(ptr, value)
    }

    pub(in crate::aot::native) fn ensure_supported_signature(
        &self,
        signature: &FunctionSignature,
        name: &str,
    ) -> Result<(), AotError> {
        for type_id in &signature.params {
            self.ensure_primitive_type(*type_id, name)?;
        }
        self.ensure_primitive_or_unit_type(signature.return_type, name)
    }

    pub(in crate::aot::native) fn ensure_supported_chunk(
        &self,
        function: &CompiledFunction,
        chunk: &Chunk,
    ) -> Result<(), AotError> {
        for (slot, type_id) in chunk.local_types.iter().enumerate().take(chunk.local_count) {
            self.ensure_primitive_type(*type_id, &format!("{} local {}", function.name, slot))?;
        }
        for constant in &chunk.constants {
            match constant {
                Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Unit | Value::String(_) => {}
                Value::Array(_) | Value::Struct(_) => {
                    return Err(AotError(format!(
                        "LLVM AOT backend does not yet support constant {:?} in `{}`",
                        constant, function.name
                    )))
                }
            }
        }
        Ok(())
    }

    fn ensure_primitive_or_unit_type(&self, type_id: TypeId, name: &str) -> Result<(), AotError> {
        if self.llvm_basic_type(type_id).is_some() || self.compiled.types.get(type_id) == &KiraType::Unit {
            Ok(())
        } else {
            Err(AotError(format!(
                "LLVM AOT backend does not yet support type {:?} in `{}`",
                self.compiled.types.get(type_id),
                name
            )))
        }
    }

    fn ensure_primitive_type(&self, type_id: TypeId, name: &str) -> Result<(), AotError> {
        if self.llvm_basic_type(type_id).is_some() {
            Ok(())
        } else {
            Err(AotError(format!(
                "LLVM AOT backend does not yet support type {:?} in `{}`",
                self.compiled.types.get(type_id),
                name
            )))
        }
    }
}
