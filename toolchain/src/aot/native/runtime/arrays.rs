// Runtime array operation declarations

use inkwell::values::{FunctionValue, PointerValue};
use inkwell::AddressSpace;

use crate::aot::error::AotError;
use super::super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn declare_runtime_new_array(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_new_array",
            handle.fn_type(&[], false),
        )
    }

    pub(in crate::aot::native) fn call_runtime_new_array(&mut self) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_new_array();
        let call_site = self
            .builder
            .build_call(f, &[], "new_array")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing new array result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    pub(in crate::aot::native) fn declare_runtime_array_push(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_push",
            self.context.void_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_array_push(
        &mut self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_array_push();
        self.builder
            .build_call(f, &[array.into(), value.into()], "push")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(in crate::aot::native) fn declare_runtime_array_append(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_append",
            self.context.void_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_array_append(
        &mut self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_array_append();
        self.builder
            .build_call(f, &[array.into(), value.into()], "append")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(in crate::aot::native) fn declare_runtime_array_length(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_length",
            self.context
                .i64_type()
                .fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(in crate::aot::native) fn call_runtime_array_length(
        &mut self,
        array: PointerValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        let f = self.declare_runtime_array_length();
        let call_site = self
            .builder
            .build_call(f, &[array.into()], "len")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing array length result".to_string()))
            .map(|v| v.into_int_value())
    }

    pub(in crate::aot::native) fn declare_runtime_array_index(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_array_index",
            handle.fn_type(
                &[handle.into(), self.context.i64_type().into()],
                false,
            ),
        )
    }

    pub(in crate::aot::native) fn call_runtime_array_index(
        &mut self,
        array: PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_array_index();
        let call_site = self
            .builder
            .build_call(f, &[array.into(), index.into()], "idx")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing array index result".to_string()))
            .map(|v| v.into_pointer_value())
    }
}
