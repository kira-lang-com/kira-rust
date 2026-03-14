use super::{types::{Attribute, Identifier, PlatformsMetadata, TypeSyntax}, Block, Span};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Auto,
    Native,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkDirective {
    pub library: String,
    pub header: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: Identifier,
    pub type_name: TypeSyntax,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDefinition {
    pub attributes: Vec<Attribute>,
    pub name: Identifier,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueTypeDefinition {
    pub name: Identifier,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: Identifier,
    pub type_name: TypeSyntax,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub attributes: Vec<Attribute>,
    pub execution_hint: Option<ExecutionMode>,
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeSyntax>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternFunctionDefinition {
    pub attributes: Vec<Attribute>,
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub return_type: Option<TypeSyntax>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: Vec<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopLevelItem {
    OpaqueType(OpaqueTypeDefinition),
    Struct(StructDefinition),
    ExternFunction(ExternFunctionDefinition),
    Function(FunctionDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub links: Vec<LinkDirective>,
    pub imports: Vec<Import>,
    pub platforms: Option<PlatformsMetadata>,
    pub items: Vec<TopLevelItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub platforms: Option<PlatformsMetadata>,
    pub links: Vec<LinkDirective>,
    pub items: Vec<TopLevelItem>,
}
