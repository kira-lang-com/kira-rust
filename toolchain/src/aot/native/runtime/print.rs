// Runtime print function declarations

use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

use super::super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn declare_runtime_print_int(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_int",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.i64_type().into()], false),
        )
    }

    pub(in crate::aot::native) fn declare_runtime_print_bool(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_bool",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.bool_type().into()], false),
        )
    }

    pub(in crate::aot::native) fn declare_runtime_print_float(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_float",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.f64_type().into()], false),
        )
    }

    pub(in crate::aot::native) fn declare_runtime_print_value(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        let handle = self.value_handle_type();
        self.declare_runtime_function(
            "kira_native_print_value",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), handle.into()], false),
        )
    }
}
