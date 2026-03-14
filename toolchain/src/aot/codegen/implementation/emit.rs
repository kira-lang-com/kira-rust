// Instruction emission and function code generation

use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{FloatPredicate, IntPredicate};

use crate::compiler::{BackendKind, Chunk, CompiledFunction, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId};

use super::super::super::error::AotError;
use super::super::super::stack::{infer_stack_layout, StackState};
use super::super::super::utils::mangle_ident;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(super) fn emit_function(
        &mut self,
        function: &CompiledFunction,
        chunk: &Chunk,
    ) -> Result<(), AotError> {
        self.ensure_supported_signature(&function.signature, &function.name)?;
        self.ensure_supported_chunk(function, chunk)?;

        let function_value = *self
            .function_values
            .get(&function.name)
            .ok_or_else(|| AotError(format!("missing LLVM declaration for `{}`", function.name)))?;

        let entry = self.context.append_basic_block(function_value, "entry");
        let blocks = (0..chunk.instructions.len())
            .map(|index| {
                self.context
                    .append_basic_block(function_value, &format!("bb{index}"))
            })
            .collect::<Vec<_>>();

        let ctx_arg = function_value
            .get_first_param()
            .ok_or_else(|| {
                AotError(format!(
                    "missing runtime context parameter for `{}`",
                    function.name
                ))
            })?
            .into_pointer_value();

        self.builder.position_at_end(entry);

        let locals =
            self.build_local_allocas_in_place(function_value, &function.signature, chunk)?;
        let stack_layout = infer_stack_layout(self.compiled, chunk).map_err(|error| {
            AotError(format!(
                "failed to infer native stack layout for `{}`: {}",
                function.name, error
            ))
        })?;
        let stack_slots =
            self.build_stack_allocas_in_place(function_value, &stack_layout.stack_slot_types)?;

        for (index, local) in locals
            .iter()
            .enumerate()
            .take(function.signature.params.len())
        {
            let param = function_value
                .get_nth_param((index + 1) as u32)
                .ok_or_else(|| {
                    AotError(format!("missing parameter {index} for `{}`", function.name))
                })?;
            self.builder
                .build_store(*local, param)
                .map_err(|error| AotError(error.to_string()))?;
        }

        self.builder
            .build_unconditional_branch(blocks[0])
            .map_err(|error| AotError(error.to_string()))?;

        for (index, block) in blocks.iter().enumerate() {
            self.builder.position_at_end(*block);
            let state = &stack_layout.states[index];
            self.emit_instruction(
                function_value,
                ctx_arg,
                &function.signature,
                chunk,
                index,
                state,
                &locals,
                &stack_slots,
                &stack_layout.stack_slot_types,
                &blocks,
            )?;
        }

        Ok(())
    }

    fn build_local_allocas_in_place(
        &mut self,
        _function: FunctionValue<'ctx>,
        _signature: &FunctionSignature,
        chunk: &Chunk,
    ) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut locals = Vec::with_capacity(chunk.local_count);
        for (index, type_id) in chunk.local_types.iter().enumerate().take(chunk.local_count) {
            let llvm_type = self.llvm_basic_type(*type_id).ok_or_else(|| {
                AotError(format!(
                    "AOT backend does not yet support local type {:?} in slot {}",
                    self.compiled.types.get(*type_id),
                    index
                ))
            })?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, &format!("local_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            locals.push(alloca);
        }
        Ok(locals)
    }

    fn build_stack_allocas_in_place(
        &mut self,
        _function: FunctionValue<'ctx>,
        stack_slot_types: &[TypeId],
    ) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut slots = Vec::with_capacity(stack_slot_types.len());
        for (index, type_id) in stack_slot_types.iter().copied().enumerate() {
            let llvm_type = self.llvm_basic_type(type_id).ok_or_else(|| {
                AotError(format!(
                    "AOT backend does not yet support stack type {:?} in slot {}",
                    self.compiled.types.get(type_id),
                    index
                ))
            })?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, &format!("stack_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            slots.push(alloca);
        }
        Ok(slots)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_instruction(
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

    pub(super) fn emit_comparison(
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
