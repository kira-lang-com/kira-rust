// Abstract Syntax Tree definitions

pub type Span = std::ops::Range<usize>;

mod expressions;
mod items;
mod statements;
mod types;

pub use expressions::{Expression, ExpressionKind, StructLiteralField};
pub use items::{
    ExecutionMode, ExternFunctionDefinition, FunctionDefinition, Import, LinkDirective,
    OpaqueTypeDefinition, Parameter, Program, SourceFile, StructDefinition, StructField,
    TopLevelItem,
};
pub use statements::{
    AssignStatement, AssignTarget, Block, ExpressionStatement, ForStatement, IfStatement,
    LetStatement, ReturnStatement, Statement, WhileStatement,
};
pub use types::{Attribute, BinaryOperator, Identifier, Literal, PlatformGroup, PlatformsMetadata, TypeSyntax, UnaryOperator};
