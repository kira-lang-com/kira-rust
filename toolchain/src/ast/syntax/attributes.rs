use super::{Identifier, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: Identifier,
    pub arguments: Vec<Identifier>,
    pub span: Span,
}
