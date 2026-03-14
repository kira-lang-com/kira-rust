use super::{BinaryOperator, Identifier, Literal, Span, TypeSyntax, UnaryOperator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

impl Expression {
    pub fn new(kind: ExpressionKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructLiteralField {
    pub name: Identifier,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind {
    Literal(Literal),
    ArrayLiteral(Vec<Expression>),
    StructLiteral {
        name: TypeSyntax,
        fields: Vec<StructLiteralField>,
    },
    Variable(Identifier),
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
    },
    Member {
        target: Box<Expression>,
        field: Identifier,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    Cast {
        target: TypeSyntax,
        expr: Box<Expression>,
    },
    Unary {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
}
