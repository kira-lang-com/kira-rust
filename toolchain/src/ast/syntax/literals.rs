use ordered_float::OrderedFloat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Bool(bool),
    Integer(i64),
    Float(OrderedFloat<f64>),
    String(String),
}
