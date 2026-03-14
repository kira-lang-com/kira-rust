mod core;
mod infrastructure;
mod metadata;

#[cfg(test)]
mod tests;

pub use infrastructure::ParseError;

use chumsky::Parser;

use crate::ast::SourceFile;

pub fn parse(source: &str) -> Result<SourceFile, Vec<ParseError>> {
    core::items::program_parser()
        .parse(source)
        .into_result()
        .map_err(infrastructure::convert_errors)
}
