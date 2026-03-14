use super::{types::{Identifier, TypeSyntax}, Expression, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignTarget {
    Variable(Identifier),
    Field {
        target: Box<AssignTarget>,
        field: Identifier,
        span: Span,
    },
}

impl AssignTarget {
    pub fn span(&self) -> Span {
        match self {
            Self::Variable(identifier) => identifier.span.clone(),
            Self::Field { span, .. } => span.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Identifier,
    pub type_ann: Option<TypeSyntax>,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub expression: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignStatement {
    pub target: AssignTarget,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForStatement {
    pub binding: Identifier,
    pub iterable: Expression,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(LetStatement),
    Assign(AssignStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Break(Span),
    Continue(Span),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}
