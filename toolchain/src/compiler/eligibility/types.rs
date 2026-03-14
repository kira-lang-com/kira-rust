use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

#[derive(Clone, Copy)]
pub struct LocalBinding {
    pub type_id: TypeId,
}

#[derive(Clone, Copy)]
pub struct ExpressionProfile {
    pub type_id: TypeId,
    pub native_eligible: bool,
}

pub fn type_is_native_eligible(types: &TypeSystem, type_id: TypeId) -> bool {
    match types.get(type_id) {
        KiraType::Dynamic => true,
        KiraType::Array(element) => type_is_native_eligible(types, *element),
        KiraType::Struct(_) => types
            .struct_fields(type_id)
            .into_iter()
            .flatten()
            .all(|field| type_is_native_eligible(types, field.type_id)),
        _ => true,
    }
}

pub fn is_numeric_type(types: &TypeSystem, type_id: TypeId) -> bool {
    matches!(types.get(type_id), KiraType::Int | KiraType::Float)
}

pub fn is_equatable_type(types: &TypeSystem, type_id: TypeId) -> bool {
    matches!(
        types.get(type_id),
        KiraType::Bool
            | KiraType::Int
            | KiraType::Float
            | KiraType::String
            | KiraType::Array(_)
            | KiraType::Struct(_)
    )
}
