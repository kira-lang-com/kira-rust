// Runtime struct operation declarations

use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;

use crate::runtime::type_system::TypeId;

use crate::aot::error::AotError;
use super::super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn declare_runtime_new_struct(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_new_struct",
            handle.fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                ],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_new_struct(
        &mut self,
        ctx: PointerValue<'ctx>,
        type_id: TypeId,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_new_struct();
        let usize_ty = self.ptr_sized_int_type();
        let type_id_value = usize_ty.const_int(type_id.0 as u64, false);
        let call_site = self
            .builder
            .build_call(f, &[ctx.into(), type_id_value.into()], "new_struct")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing new struct result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(in crate::aot::native) fn declare_runtime_struct_set_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        self.declare_runtime_function(
            "kira_native_struct_set_field",
            self.context.void_type().fn_type(
                &[
                    self.value_handle_type().into(),
                    usize_ty.into(),
                    self.value_handle_type().into(),
                ],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_struct_set_field(
        &mut self,
        target: PointerValue<'ctx>,
        field_index: usize,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_struct_set_field();
        let usize_ty = self.ptr_sized_int_type();
        let index = usize_ty.const_int(field_index as u64, false);
        self.builder
            .build_call(f, &[target.into(), index.into(), value.into()], "set_field")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(in crate::aot::native) fn declare_runtime_struct_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_struct_field",
            handle.fn_type(
                &[handle.into(), usize_ty.into()],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_struct_field(
        &mut self,
        target: PointerValue<'ctx>,
        field_index: usize,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_struct_field();
        let usize_ty = self.ptr_sized_int_type();
        let index = usize_ty.const_int(field_index as u64, false);
        let call_site = self
            .builder
            .build_call(f, &[target.into(), index.into()], "get_field")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing struct field result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(in crate::aot::native) fn declare_runtime_store_struct_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        self.declare_runtime_function(
            "kira_native_store_struct_field",
            self.context.void_type().fn_type(
                &[
                    self.value_handle_type().into(),
                    usize_ty.ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                    self.value_handle_type().into(),
                ],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_store_struct_field(
        &mut self,
        target: PointerValue<'ctx>,
        path: PointerValue<'ctx>,
        len: inkwell::values::IntValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_store_struct_field();
        self.builder
            .build_call(f, &[target.into(), path.into(), len.into(), value.into()], "store_field")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }
}
