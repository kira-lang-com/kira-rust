use crate::ast::Span;
use super::Identifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformGroup {
    pub name: Identifier,
    pub members: Vec<Identifier>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformsMetadata {
    pub groups: Vec<PlatformGroup>,
    pub span: Span,
}
