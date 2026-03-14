// Parser infrastructure: error handling and common utilities

use chumsky::prelude::*;
use chumsky::error::Rich;
use chumsky::span::SimpleSpan;

use crate::ast::Span;

// Error handling

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

// Common parser utilities

pub type RichError<'src> = extra::Err<Rich<'src, char>>;

pub fn span_to_range(span: SimpleSpan<usize>) -> Span {
    span.into_range()
}

pub fn keyword<'src>(
    value: &'static str,
) -> impl Parser<'src, &'src str, &'src str, RichError<'src>> + Clone {
    pad(text::keyword::<_, _, RichError<'src>>(value))
}

pub fn symbol<'src>(
    value: char,
) -> impl Parser<'src, &'src str, char, RichError<'src>> + Clone {
    pad(just::<_, _, RichError<'src>>(value))
}

pub fn token<'src>(
    value: &'static str,
) -> impl Parser<'src, &'src str, &'src str, RichError<'src>> + Clone {
    pad(just::<_, _, RichError<'src>>(value))
}

pub fn pad<'src, O, P>(
    parser: P,
) -> impl Parser<'src, &'src str, O, RichError<'src>> + Clone
where
    P: Parser<'src, &'src str, O, RichError<'src>> + Clone,
{
    parser.padded_by(padding())
}

fn padding<'src>() -> impl Parser<'src, &'src str, (), RichError<'src>> + Clone {
    choice((whitespace_padding(), line_comment_padding()))
        .repeated()
        .ignored()
}

fn whitespace_padding<'src>() -> impl Parser<'src, &'src str, (), RichError<'src>> + Clone {
    any()
        .filter(|ch: &char| ch.is_whitespace())
        .repeated()
        .at_least(1)
        .ignored()
}

fn line_comment_padding<'src>() -> impl Parser<'src, &'src str, (), RichError<'src>> + Clone {
    just::<_, _, RichError<'src>>("//")
        .then(any().filter(|ch: &char| *ch != '\n').repeated())
        .ignored()
}
