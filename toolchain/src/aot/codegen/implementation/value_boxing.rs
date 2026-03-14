// Value boxing and unboxing operations

use inkwell::values::{BasicValueEnum, PointerValue};

use crate::runtime::type_system::{KiraType, TypeId};

use super::super::super::error::AotError;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn is_value_handle_type(&self, type_id: TypeId) -> bool {
        matches!(
            self.compiled.types.get(type_id),
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_)
        )
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
}
