// Instruction dispatch and comparison operations

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

use crate::compiler::{Chunk, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId};

use crate::aot::error::AotError;
use crate::aot::stack::StackState;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::aot::native) fn emit_instruction(
        &mut self,
        function_value: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        signature: &FunctionSignature,
        chunk: &Chunk,
        index: usize,
        state: &StackState,
        locals: &[PointerValue<'ctx>],
        stack_slots: &[PointerValue<'ctx>],
        stack_slot_types: &[TypeId],
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        let instruction = chunk
            .instructions
            .get(index)
            .ok_or_else(|| AotError(format!("invalid instruction index {index}")))?;
        let depth = state.depth();

        match instruction {
            Instruction::LoadConst(const_index) => {
                self.emit_load_const(stack_slots, depth, chunk, *const_index)?;
            }
            Instruction::LoadLocal(local_index) => {
                self.emit_load_local(stack_slots, depth, chunk, locals, *local_index)?;
            }
            Instruction::StoreLocal(local_index) => {
                self.emit_store_local(stack_slots, depth, chunk, locals, *local_index)?;
            }
            Instruction::Negate => {
                self.emit_negate(stack_slots, depth, state)?;
            }
            Instruction::CastIntToFloat => {
                self.emit_cast_int_to_float(stack_slots, depth)?;
            }
            Instruction::Add
            | Instruction::Subtract
            | Instruction::Multiply
            | Instruction::Divide
            | Instruction::Modulo => {
                self.emit_binary_arithmetic(instruction, stack_slots, depth, state)?;
            }
            Instruction::Less
            | Instruction::Greater
            | Instruction::Equal
            | Instruction::NotEqual
            | Instruction::LessEqual
            | Instruction::GreaterEqual => {
                self.emit_comparison_op(instruction, stack_slots, depth, state)?;
            }
            Instruction::BuildArray {
                type_id,
                element_count,
            } => {
                self.emit_build_array(stack_slots, depth, *type_id, *element_count)?;
            }
            Instruction::BuildStruct { type_id, field_count } => {
                self.emit_build_struct(ctx_arg, stack_slots, depth, *type_id, *field_count)?;
            }
            Instruction::ArrayLength => {
                self.emit_array_length(stack_slots, depth, state)?;
            }
            Instruction::ArrayIndex => {
                self.emit_array_index(stack_slots, depth, state)?;
            }
            Instruction::StructField(field_index) => {
                self.emit_struct_field(stack_slots, depth, state, *field_index)?;
            }
            Instruction::StoreLocalField { local, path } => {
                self.emit_store_local_field(stack_slots, depth, chunk, locals, index, *local, path)?;
            }
            Instruction::ArrayAppendLocal(local_index) => {
                self.emit_array_append_local(stack_slots, depth, chunk, locals, state, *local_index)?;
            }
            Instruction::JumpIfFalse(target) => {
                self.emit_jump_if_false(stack_slots, depth, index, *target, blocks)?;
                return Ok(());
            }
            Instruction::Jump(target) => {
                self.emit_jump(*target, blocks)?;
                return Ok(());
            }
            Instruction::Call { function, arg_count } => {
                self.emit_call_instruction(
                    function_value,
                    ctx_arg,
                    function,
                    *arg_count,
                    stack_slots,
                    stack_slot_types,
                    depth,
                )?;
            }
            Instruction::Pop => {
                // Stack slot values are left as-is; the stack layout determines liveness.
            }
            Instruction::Return => {
                self.emit_return(signature, stack_slots, depth)?;
                return Ok(());
            }
        }

        // Default fallthrough
        self.emit_fallthrough(function_value, index, blocks)?;
        Ok(())
    }

    fn emit_fallthrough(
        &mut self,
        function_value: FunctionValue<'ctx>,
        index: usize,
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        if index + 1 < blocks.len() {
            let block = self
                .builder
                .get_insert_block()
                .ok_or_else(|| AotError("missing insert block".to_string()))?;
            if block.get_terminator().is_none() {
                self.builder
                    .build_unconditional_branch(blocks[index + 1])
                    .map_err(|e| AotError(e.to_string()))?;
            }
        } else {
            let block = self
                .builder
                .get_insert_block()
                .ok_or_else(|| AotError("missing insert block".to_string()))?;
            if block.get_terminator().is_none() {
                return Err(AotError(format!(
                    "`{}` has no terminator at end of function",
                    function_value.get_name().to_string_lossy()
                )));
            }
        }
        Ok(())
    }

    pub(in crate::aot::native) fn emit_comparison(
        &mut self,
        instruction: &Instruction,
        type_id: TypeId,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        match self.compiled.types.get(type_id) {
            KiraType::Int | KiraType::Bool => {
                let pred = match instruction {
                    Instruction::Less => IntPredicate::SLT,
                    Instruction::Greater => IntPredicate::SGT,
                    Instruction::Equal => IntPredicate::EQ,
                    Instruction::NotEqual => IntPredicate::NE,
                    Instruction::LessEqual => IntPredicate::SLE,
                    Instruction::GreaterEqual => IntPredicate::SGE,
                    _ => return Err(AotError("invalid comparison opcode".to_string())),
                };
                self.builder
                    .build_int_compare(pred, left.into_int_value(), right.into_int_value(), "icmp")
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::Float => {
                let pred = match instruction {
                    Instruction::Less => FloatPredicate::OLT,
                    Instruction::Greater => FloatPredicate::OGT,
                    Instruction::Equal => FloatPredicate::OEQ,
                    Instruction::NotEqual => FloatPredicate::ONE,
                    Instruction::LessEqual => FloatPredicate::OLE,
                    Instruction::GreaterEqual => FloatPredicate::OGE,
                    _ => return Err(AotError("invalid comparison opcode".to_string())),
                };
                self.builder
                    .build_float_compare(
                        pred,
                        left.into_float_value(),
                        right.into_float_value(),
                        "fcmp",
                    )
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::Opaque(_) => {
                let pred = match instruction {
                    Instruction::Equal => IntPredicate::EQ,
                    Instruction::NotEqual => IntPredicate::NE,
                    _ => {
                        return Err(AotError(
                            "only == and != are supported for opaque handles".to_string(),
                        ))
                    }
                };
                self.builder
                    .build_int_compare(
                        pred,
                        self.ptr_to_int(left.into_pointer_value())?,
                        self.ptr_to_int(right.into_pointer_value())?,
                        "pcmp",
                    )
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                if !matches!(instruction, Instruction::Equal | Instruction::NotEqual) {
                    return Err(AotError(
                        "only == and != are supported for value handles".to_string(),
                    ));
                }
                let eq = self.call_runtime_value_eq(
                    left.into_pointer_value(),
                    right.into_pointer_value(),
                )?;
                if matches!(instruction, Instruction::Equal) {
                    Ok(eq)
                } else {
                    self.builder
                        .build_not(eq, "not")
                        .map_err(|e| AotError(e.to_string()))
                }
            }
            other => Err(AotError(format!(
                "comparison is not supported for type {:?}",
                other
            ))),
        }
    }
}
