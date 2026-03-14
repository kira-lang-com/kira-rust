pub type Span = std::ops::Range<usize>;

mod attributes;
mod expressions;
mod identifiers;
mod items;
mod literals;
mod metadata;
mod operators;
mod statements;

pub use attributes::Attribute;
pub use expressions::{Expression, ExpressionKind, StructLiteralField};
pub use identifiers::{Identifier, TypeSyntax};
pub use items::{
    ExecutionMode, FunctionDefinition, Import, Parameter, Program, SourceFile, StructDefinition,
    StructField, TopLevelItem,
};
pub use literals::Literal;
pub use metadata::{PlatformGroup, PlatformsMetadata};
pub use operators::{BinaryOperator, UnaryOperator};
pub use statements::{
    AssignStatement, AssignTarget, Block, ExpressionStatement, ForStatement, IfStatement,
    LetStatement, ReturnStatement, Statement, WhileStatement,
};
