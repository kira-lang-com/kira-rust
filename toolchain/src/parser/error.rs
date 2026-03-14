use chumsky::error::Rich;
use chumsky::span::SimpleSpan;

use crate::ast::Span;

use super::common::span_to_range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}..{}", self.message, self.span.start, self.span.end)
    }
}

impl std::error::Error for ParseError {}

pub fn convert_errors(errors: Vec<Rich<'_, char>>) -> Vec<ParseError> {
    errors
        .into_iter()
        .map(|error| ParseError {
            message: error.to_string(),
            span: span_to_range(copy_span(error.span())),
        })
        .collect()
}

fn copy_span(span: &SimpleSpan<usize>) -> SimpleSpan<usize> {
    *span
}
