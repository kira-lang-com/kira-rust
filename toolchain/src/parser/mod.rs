mod attributes;
mod common;
mod error;
mod expressions;
mod identifiers;
mod items;
mod literals;
mod platforms;
mod statements;

#[cfg(test)]
mod tests;

pub use error::ParseError;

use chumsky::Parser;

use crate::ast::SourceFile;

pub fn parse(source: &str) -> Result<SourceFile, Vec<ParseError>> {
    items::program_parser()
        .parse(source)
        .into_result()
        .map_err(error::convert_errors)
}
