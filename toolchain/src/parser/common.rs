use chumsky::prelude::*;

use crate::ast::Span;

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
