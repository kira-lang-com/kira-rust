// Base runtime function declaration helper

use inkwell::values::FunctionValue;

use super::super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn declare_runtime_function(
        &self,
        name: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module
            .get_function(name)
            .unwrap_or_else(|| self.module.add_function(name, fn_type, None))
    }
}
