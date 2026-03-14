// Control flow instructions (jumps, returns)

use inkwell::basic_block::BasicBlock;
use inkwell::values::PointerValue;

use crate::compiler::FunctionSignature;
use crate::runtime::type_system::KiraType;

use crate::aot::error::AotError;
use super::context::NativeCodegen;

impl<'ctx> NativeCodegen<'ctx> {
    pub(in crate::aot::native) fn emit_jump_if_false(
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

    pub(in crate::aot::native) fn emit_jump(
        &mut self,
        target: usize,
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        self.builder
            .build_unconditional_branch(blocks[target])
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    pub(in crate::aot::native) fn emit_return(
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
}
