// Basic instruction helpers (load, store, arithmetic, etc.)

use inkwell::values::{BasicValueEnum, PointerValue};

use crate::compiler::Chunk;
use crate::runtime::type_system::{KiraType, TypeId};

use crate::aot::error::AotError;
use crate::aot::stack::StackState;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn emit_load_const(
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

    pub(in crate::aot::native) fn emit_load_local(
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

    pub(in crate::aot::native) fn emit_store_local(
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

    pub(in crate::aot::native) fn emit_negate(
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

    pub(in crate::aot::native) fn emit_cast_int_to_float(
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

    pub(in crate::aot::native) fn emit_binary_arithmetic(
        &mut self,
        instruction: &crate::compiler::Instruction,
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

        use crate::compiler::Instruction;
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

    pub(in crate::aot::native) fn emit_comparison_op(
        &mut self,
        instruction: &crate::compiler::Instruction,
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
}
