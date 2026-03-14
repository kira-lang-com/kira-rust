// Function call emission (FFI, native, bridge)

use inkwell::types::BasicType;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

use crate::compiler::FunctionSignature;
use crate::runtime::type_system::KiraType;

use crate::aot::error::AotError;
use crate::aot::utils::mangle_ident;
use super::super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn emit_ffi_call(
        &mut self,
        callee: &str,
        signature: &FunctionSignature,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let fn_value = self.declare_ffi_function(callee, signature)?;
        let call_site = self
            .builder
            .build_call(
                fn_value,
                &args.iter().copied().map(Into::into).collect::<Vec<_>>(),
                "ffi_call",
            )
            .map_err(|e| AotError(e.to_string()))?;
        if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
            Ok(self.context.i64_type().const_zero().into())
        } else {
            call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| AotError("missing ffi call result".to_string()))
        }
    }

    pub(in crate::aot::native) fn emit_native_call(
        &mut self,
        callee: &str,
        signature: &FunctionSignature,
        ctx_arg: PointerValue<'ctx>,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let fn_value = *self.function_values.get(callee).ok_or_else(|| {
            AotError(format!("missing native callee `{}`", callee))
        })?;
        let mut full_args = Vec::with_capacity(args.len() + 1);
        full_args.push(ctx_arg.into());
        for value in args.iter().copied() {
            full_args.push(value.into());
        }
        let call_site = self
            .builder
            .build_call(fn_value, &full_args, "call_native")
            .map_err(|e| AotError(e.to_string()))?;
        if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
            Ok(self.context.i64_type().const_zero().into())
        } else {
            call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| AotError("missing call result".to_string()))
        }
    }

    pub(in crate::aot::native) fn emit_bridge_call(
        &mut self,
        callee: &str,
        signature: &FunctionSignature,
        ctx_arg: PointerValue<'ctx>,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let bridge_symbol = format!("kira_bridge_{}", mangle_ident(callee));
        let bridge_fn = self.declare_bridge_function(&bridge_symbol, signature)?;
        let mut full_args = Vec::with_capacity(args.len() + 1);
        full_args.push(ctx_arg.into());
        for value in args.iter().copied() {
            full_args.push(value.into());
        }
        let call_site = self
            .builder
            .build_call(bridge_fn, &full_args, "call_bridge")
            .map_err(|e| AotError(e.to_string()))?;
        if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
            Ok(self.context.i64_type().const_zero().into())
        } else {
            call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| AotError("missing bridge call result".to_string()))
        }
    }

    pub(super) fn declare_bridge_function(
        &mut self,
        symbol: &str,
        signature: &FunctionSignature,
    ) -> Result<FunctionValue<'ctx>, AotError> {
        if let Some(existing) = self.bridge_values.get(symbol).copied() {
            return Ok(existing);
        }
        let fn_type = self.llvm_function_type(signature)?;
        let value = self.module.add_function(symbol, fn_type, None);
        self.bridge_values.insert(symbol.to_string(), value);
        Ok(value)
    }

    pub(super) fn declare_ffi_function(
        &mut self,
        symbol: &str,
        signature: &FunctionSignature,
    ) -> Result<FunctionValue<'ctx>, AotError> {
        if let Some(existing) = self.module.get_function(symbol) {
            return Ok(existing);
        }
        let mut params = Vec::with_capacity(signature.params.len());
        for type_id in &signature.params {
            params.push(
                self.llvm_basic_type(*type_id)
                    .ok_or_else(|| AotError("unsupported FFI parameter type".to_string()))?,
            );
        }
        let fn_type = match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => self.context.void_type().fn_type(
                &params.iter().copied().map(Into::into).collect::<Vec<_>>(),
                false,
            ),
            _ => self
                .llvm_basic_type(signature.return_type)
                .ok_or_else(|| AotError("unsupported FFI return type".to_string()))?
                .fn_type(&params.iter().copied().map(Into::into).collect::<Vec<_>>(), false),
        };
        Ok(self.module.add_function(symbol, fn_type, None))
    }
}
