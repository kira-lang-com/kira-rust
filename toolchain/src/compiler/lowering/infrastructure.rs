// Lowering infrastructure: types, chunk helpers, and utilities

use crate::compiler::{Chunk, CompileError, Instruction};
use crate::runtime::type_system::{TypeId, TypeSystem};

// Type utilities

#[derive(Clone, Copy)]
pub struct LocalBinding {
    pub slot: usize,
    pub type_id: TypeId,
}

#[derive(Default)]
pub struct LoopContext {
    pub break_jumps: Vec<usize>,
    pub continue_jumps: Vec<usize>,
}

pub fn is_numeric_type(types: &TypeSystem, type_id: TypeId) -> bool {
    type_id == types.int() || type_id == types.float()
}

pub fn is_equatable_type(types: &TypeSystem, type_id: TypeId) -> bool {
    is_numeric_type(types, type_id) || type_id == types.bool()
}

// Chunk manipulation helpers

pub fn allocate_local(chunk: &mut Chunk) -> usize {
    let slot = chunk.local_count;
    chunk.local_count += 1;
    if chunk.local_types.len() < chunk.local_count {
        chunk.local_types.resize(chunk.local_count, TypeId(0));
    }
    slot
}

pub fn ensure_local_type(chunk: &mut Chunk, slot: usize, type_id: TypeId) {
    if slot < chunk.local_types.len() {
        chunk.local_types[slot] = type_id;
    }
}

pub fn patch_jump(instructions: &mut [Instruction], index: usize, target: usize) {
    if let Some(Instruction::JumpIfFalse(_)) = instructions.get_mut(index) {
        instructions[index] = Instruction::JumpIfFalse(target);
    } else if let Some(Instruction::Jump(_)) = instructions.get_mut(index) {
        instructions[index] = Instruction::Jump(target);
    }
}

pub fn patch_jumps(instructions: &mut [Instruction], indexes: &[usize], target: usize) {
    for &index in indexes {
        patch_jump(instructions, index, target);
    }
}

pub fn require_same_numeric_types(
    left: TypeId,
    right: TypeId,
    types: &TypeSystem,
) -> Result<TypeId, CompileError> {
    if left == right && is_numeric_type(types, left) {
        Ok(left)
    } else {
        Err(CompileError(format!(
            "expected matching numeric types, got {:?} and {:?}",
            types.get(left),
            types.get(right)
        )))
    }
}

pub fn require_same_equatable_types(
    left: TypeId,
    right: TypeId,
    types: &TypeSystem,
) -> Result<(), CompileError> {
    if left == right && is_equatable_type(types, left) {
        Ok(())
    } else {
        Err(CompileError(format!(
            "expected matching equatable types, got {:?} and {:?}",
            types.get(left),
            types.get(right)
        )))
    }
}
