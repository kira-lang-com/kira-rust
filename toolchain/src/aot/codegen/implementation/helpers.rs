// Helper functions for specific instruction emissions

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

use crate::compiler::{BackendKind, Chunk, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId};

use super::super::super::error::AotError;
use super::super::super::stack::StackState;
use super::super::super::utils::mangle_ident;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn emit_load_const(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        chunk: &Chunk,
        const_index: usize,
    ) -> Result<(), AotError> {
        let value = chunk
            .constants
            .get(const_index)
            .ok_or_else(|| AotError(format!("invalid constant index {const_index}")))?;
        let (type_id, llvm_value) = self.llvm_const(value)?;
        self.store_stack(stack_slots, depth, type_id, llvm_value)
    }

    pub(super) fn emit_load_local(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        chunk: &Chunk,
        locals: &[PointerValue<'ctx>],
        local_index: usize,
    ) -> Result<(), AotError> {
        let type_id = *chunk
            .local_types
            .get(local_index)
            .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
        let local = *locals
            .get(local_index)
            .ok_or_else(|| AotError(format!("missing local slot {local_index}")))?;
        let mut value =
            self.load_typed_ptr(local, type_id, &format!("local_{local_index}_value"))?;
        if self.is_value_handle_type(type_id) {
            value = self.clone_value_handle(value)?;
        }
        self.store_stack(stack_slots, depth, type_id, value)
    }

    pub(super) fn emit_store_local(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        chunk: &Chunk,
        locals: &[PointerValue<'ctx>],
        local_index: usize,
    ) -> Result<(), AotError> {
        let type_id = *chunk
            .local_types
            .get(local_index)
            .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
        let value = self.load_stack(stack_slots, depth - 1, type_id, "store_value")?;
        let local = *locals
            .get(local_index)
            .ok_or_else(|| AotError(format!("missing local slot {local_index}")))?;
        self.store_ptr(local, value)
    }

    pub(super) fn emit_negate(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        state: &StackState,
    ) -> Result<(), AotError> {
        let type_id = *state
            .stack
            .last()
            .ok_or_else(|| AotError("stack underflow on negate".to_string()))?;
        let value = self.load_stack(stack_slots, depth - 1, type_id, "neg_arg")?;
        let result = match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let v = value.into_int_value();
                self.builder
                    .build_int_neg(v, "neg")
                    .map_err(|e| AotError(e.to_string()))?
                    .into()
            }
            KiraType::Float => {
                let v = value.into_float_value();
                self.builder
                    .build_float_neg(v, "fneg")
                    .map_err(|e| AotError(e.to_string()))?
                    .into()
            }
            other => {
                return Err(AotError(format!(
                    "negation is not supported for type {:?}",
                    other
                )));
            }
        };
        self.store_stack(stack_slots, depth - 1, type_id, result)
    }

    pub(super) fn emit_cast_int_to_float(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
    ) -> Result<(), AotError> {
        let src = self.load_stack(stack_slots, depth - 1, self.compiled.types.int(), "int")?;
        let float = self
            .builder
            .build_signed_int_to_float(
                src.into_int_value(),
                self.context.f64_type(),
                "i2f",
            )
            .map_err(|e| AotError(e.to_string()))?;
        self.store_stack(stack_slots, depth - 1, self.compiled.types.float(), float.into())
    }

    pub(super) fn emit_binary_arithmetic(
        &mut self,
        instruction: &Instruction,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        state: &StackState,
    ) -> Result<(), AotError> {
        let right_type = *state
            .stack
            .last()
            .ok_or_else(|| AotError("stack underflow".to_string()))?;
        let left_type = *state
            .stack
            .get(depth - 2)
            .ok_or_else(|| AotError("stack underflow".to_string()))?;
        if left_type != right_type {
            return Err(AotError("binary operand type mismatch".to_string()));
        }
        let left = self.load_stack(stack_slots, depth - 2, left_type, "lhs")?;
        let right = self.load_stack(stack_slots, depth - 1, right_type, "rhs")?;

        let result = match (instruction, self.compiled.types.get(left_type)) {
            (Instruction::Add, KiraType::Int) => self
                .builder
                .build_int_add(left.into_int_value(), right.into_int_value(), "add")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Subtract, KiraType::Int) => self
                .builder
                .build_int_sub(left.into_int_value(), right.into_int_value(), "sub")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Multiply, KiraType::Int) => self
                .builder
                .build_int_mul(left.into_int_value(), right.into_int_value(), "mul")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Divide, KiraType::Int) => self
                .builder
                .build_int_signed_div(left.into_int_value(), right.into_int_value(), "div")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Modulo, KiraType::Int) => self
                .builder
                .build_int_signed_rem(left.into_int_value(), right.into_int_value(), "rem")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Add, KiraType::Float) => self
                .builder
                .build_float_add(left.into_float_value(), right.into_float_value(), "fadd")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Subtract, KiraType::Float) => self
                .builder
                .build_float_sub(left.into_float_value(), right.into_float_value(), "fsub")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Multiply, KiraType::Float) => self
                .builder
                .build_float_mul(left.into_float_value(), right.into_float_value(), "fmul")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Divide, KiraType::Float) => self
                .builder
                .build_float_div(left.into_float_value(), right.into_float_value(), "fdiv")
                .map_err(|e| AotError(e.to_string()))?
                .into(),
            (Instruction::Modulo, KiraType::Float) => {
                return Err(AotError("modulo is not supported for float".to_string()));
            }
            (_, other) => {
                return Err(AotError(format!(
                    "arithmetic is not supported for type {:?}",
                    other
                )));
            }
        };

        self.store_stack(stack_slots, depth - 2, left_type, result)
    }

    pub(super) fn emit_comparison_op(
        &mut self,
        instruction: &Instruction,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        state: &StackState,
    ) -> Result<(), AotError> {
        let right_type = *state
            .stack
            .last()
            .ok_or_else(|| AotError("stack underflow".to_string()))?;
        let left_type = *state
            .stack
            .get(depth - 2)
            .ok_or_else(|| AotError("stack underflow".to_string()))?;
        let left = self.load_stack(stack_slots, depth - 2, left_type, "cmp_lhs")?;
        let right = self.load_stack(stack_slots, depth - 1, right_type, "cmp_rhs")?;

        let result = self.emit_comparison(instruction, left_type, left, right)?;
        self.store_stack(
            stack_slots,
            depth - 2,
            self.compiled.types.bool(),
            result.into(),
        )
    }

    pub(super) fn emit_jump_if_false(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
        index: usize,
        target: usize,
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        if index + 1 >= blocks.len() {
            return Err(AotError("JumpIfFalse at end of function".to_string()));
        }
        let cond = self.load_stack(
            stack_slots,
            depth - 1,
            self.compiled.types.bool(),
            "cond",
        )?;
        self.builder
            .build_conditional_branch(
                cond.into_int_value(),
                blocks[target],
                blocks[index + 1],
            )
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(super) fn emit_jump(
        &mut self,
        target: usize,
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        self.builder
            .build_unconditional_branch(blocks[target])
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(super) fn emit_return(
        &mut self,
        signature: &FunctionSignature,
        stack_slots: &[PointerValue<'ctx>],
        depth: usize,
    ) -> Result<(), AotError> {
        match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => {
                self.builder
                    .build_return(None)
                    .map_err(|e| AotError(e.to_string()))?;
            }
            _ => {
                let type_id = signature.return_type;
                let value =
                    self.load_stack(stack_slots, depth - 1, type_id, "ret_value")?;
                self.builder
                    .build_return(Some(&value))
                    .map_err(|e| AotError(e.to_string()))?;
            }
        }
        Ok(())
    }

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
