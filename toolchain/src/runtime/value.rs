use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructValue {
    pub type_name: String,
    pub fields: Vec<(String, Value)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    Float(OrderedFloat<f64>),
    String(String),
    Array(Vec<Value>),
    Struct(StructValue),
}

impl Value {
    pub fn display(&self) -> String {
        match self {
            Self::Unit => "()".to_string(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.0.to_string(),
            Self::String(value) => value.clone(),
            Self::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Value::display)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Struct(value) => format!(
                "{} {{ {} }}",
                value.type_name,
                value
                    .fields
                    .iter()
                    .map(|(name, value)| format!("{name}: {}", value.display()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
