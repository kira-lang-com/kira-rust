use crate::runtime::type_system::{TypeId, TypeSystem};

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
