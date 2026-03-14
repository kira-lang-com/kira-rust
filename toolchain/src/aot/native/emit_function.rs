// Function code generation and setup

use inkwell::values::{FunctionValue, PointerValue};

use crate::compiler::{Chunk, CompiledFunction, FunctionSignature};
use crate::runtime::type_system::TypeId;

use crate::aot::error::AotError;
use crate::aot::stack::infer_stack_layout;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn emit_function(
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
}
