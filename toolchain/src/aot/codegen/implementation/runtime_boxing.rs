// Runtime boxing/unboxing and value operations

use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn declare_runtime_box_int(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_int",
            handle.fn_type(&[self.context.i64_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_box_bool(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_bool",
            handle.fn_type(&[self.context.bool_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_box_float(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_float",
            handle.fn_type(&[self.context.f64_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_unbox_int(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_int",
            self.context.i64_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_unbox_bool(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_bool",
            self.context.bool_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_unbox_float(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_float",
            self.context.f64_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    pub(super) fn declare_runtime_clone_value(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_clone_value",
            handle.fn_type(&[handle.into()], false),
        )
    }

    pub(super) fn declare_runtime_make_string(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_make_string",
            handle.fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                ],
                false,
            ),
        )
    }

    pub(super) fn declare_runtime_value_eq(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_value_eq",
            self.context.bool_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    pub(super) fn call_runtime_value_eq(
        &mut self,
        left: inkwell::values::PointerValue<'ctx>,
        right: inkwell::values::PointerValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, super::super::super::error::AotError> {
        let f = self.declare_runtime_value_eq();
        let call_site = self
            .builder
            .build_call(f, &[left.into(), right.into()], "value_eq")
            .map_err(|e| super::super::super::error::AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| super::super::super::error::AotError("missing eq result".to_string()))
            .map(|v| v.into_int_value())
    }
}
