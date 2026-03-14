// Small atomic AST types

mod attributes;
mod identifiers;
mod literals;
mod metadata;
mod operators;

pub use attributes::Attribute;
pub use identifiers::{Identifier, TypeSyntax};
pub use literals::Literal;
pub use metadata::{PlatformGroup, PlatformsMetadata};
pub use operators::{BinaryOperator, UnaryOperator};
