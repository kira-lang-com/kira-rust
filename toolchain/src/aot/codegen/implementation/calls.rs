// Function call emission (FFI, native, bridge)

use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

use crate::compiler::{BackendKind, FunctionSignature};
use crate::runtime::type_system::{KiraType, TypeId};

use super::super::super::error::AotError;
use super::super::super::utils::mangle_ident;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_call_instruction(
        &mut self,
        function_value: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        function: &str,
        arg_count: usize,
        stack_slots: &[PointerValue<'ctx>],
        stack_slot_types: &[TypeId],
        depth: usize,
    ) -> Result<(), AotError> {
        if function == "printIn" {
            return self.emit_print_in_call(ctx_arg, arg_count, stack_slots, stack_slot_types, depth);
        }

        let signature = self
            .compiled
            .functions
            .get(function)
            .map(|function| function.signature.clone())
            .or_else(|| {
                self.compiled
                    .ffi
                    .functions
                    .get(function)
                    .map(|function| function.signature.clone())
            })
            .or_else(|| {
                self.compiled
                    .builtins
                    .get(function)
                    .map(|builtin| builtin.signature.clone())
            })
            .ok_or_else(|| AotError(format!("missing signature for `{function}`")))?;

        let base = depth
            .checked_sub(arg_count)
            .ok_or_else(|| AotError("stack underflow on call".to_string()))?;

        let args = (0..arg_count)
            .map(|offset| {
                let type_id = signature.params[offset];
                self.load_stack(stack_slots, base + offset, type_id, "call_arg")
            })
            .collect::<Result<Vec<_>, _>>()?;

        let result =
            self.emit_call(function_value, ctx_arg, function, &signature, &args)?;
        if self.compiled.types.get(signature.return_type) != &KiraType::Unit {
            let result_slot = base;
            self.store_stack(stack_slots, result_slot, signature.return_type, result)?;
        }
        Ok(())
    }

    fn emit_print_in_call(
        &mut self,
        ctx_arg: PointerValue<'ctx>,
        arg_count: usize,
        stack_slots: &[PointerValue<'ctx>],
        stack_slot_types: &[TypeId],
        depth: usize,
    ) -> Result<(), AotError> {
        if arg_count != 1 {
            return Err(AotError(format!(
                "`printIn` expects 1 argument but got {}",
                arg_count
            )));
        }
        let base = depth
            .checked_sub(arg_count)
            .ok_or_else(|| AotError("stack underflow on call".to_string()))?;
        let arg_type = *stack_slot_types
            .get(base)
            .ok_or_else(|| AotError("missing stack slot type for printIn".to_string()))?;
        let arg = self.load_stack(stack_slots, base, arg_type, "print_arg")?;
        self.emit_print_in(ctx_arg, arg_type, arg)
    }

    pub(super) fn emit_print_in(
        &mut self,
        ctx_arg: PointerValue<'ctx>,
        type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), AotError> {
        match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let f = self.declare_runtime_print_int();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_int")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::Bool => {
                let f = self.declare_runtime_print_bool();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_bool")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::Float => {
                let f = self.declare_runtime_print_float();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_float")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::Opaque(_) => {
                let f = self.declare_runtime_print_int();
                let ptr = value.into_pointer_value();
                let as_int = self.ptr_to_int(ptr)?;
                self.builder
                    .build_call(f, &[ctx_arg.into(), as_int.into()], "print_opaque")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                let f = self.declare_runtime_print_value();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_value")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            other => {
                return Err(AotError(format!(
                    "`printIn` is not supported for type {:?} in native codegen",
                    other
                )));
            }
        }
        Ok(())
    }

    pub(super) fn emit_call(
        &mut self,
        _function_value: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        callee: &str,
        signature: &FunctionSignature,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        if let Some(ffi) = self.compiled.ffi.functions.get(callee) {
            return self.emit_ffi_call(callee, &ffi.signature, args);
        }

        if let Some(function) = self.compiled.functions.get(callee) {
            if function.selected_backend == BackendKind::Native {
                return self.emit_native_call(callee, signature, ctx_arg, args);
            }
        }

        self.emit_bridge_call(callee, signature, ctx_arg, args)
    }

    fn emit_ffi_call(
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

    fn emit_native_call(
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

    fn emit_bridge_call(
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
