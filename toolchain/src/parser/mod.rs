mod expressions;
mod identifiers;
mod infrastructure;
mod items;
mod literals;
mod metadata;
mod statements;

#[cfg(test)]
mod tests;

pub use infrastructure::ParseError;

use chumsky::Parser;

use crate::ast::SourceFile;

pub fn parse(source: &str) -> Result<SourceFile, Vec<ParseError>> {
    items::program_parser()
        .parse(source)
        .into_result()
        .map_err(infrastructure::convert_errors)
}
