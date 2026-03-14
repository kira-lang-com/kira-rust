use chumsky::prelude::*;
use ordered_float::OrderedFloat;

use crate::ast::{Expression, ExpressionKind, Literal};

use super::infrastructure::{pad, span_to_range, RichError};

pub fn string_literal_parser<'src>(
) -> impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone {
    let escape = just::<_, _, RichError<'src>>('\\').ignore_then(choice((
        just('\\'),
        just('"'),
        just('n').to('\n'),
        just('t').to('\t'),
        just('r').to('\r'),
    )));

    escape
        .or(none_of("\\\""))
        .repeated()
        .collect::<String>()
        .delimited_by(
            just::<_, _, RichError<'src>>('"'),
            just::<_, _, RichError<'src>>('"'),
        )
        .map_with(|value, extra| {
            Expression::new(
                ExpressionKind::Literal(Literal::String(value)),
                span_to_range(extra.span()),
            )
        })
}

pub fn bool_literal_parser<'src>(
) -> impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone {
    pad(
        choice((
            text::keyword::<_, _, RichError<'src>>("true").to(true),
            text::keyword::<_, _, RichError<'src>>("false").to(false),
        ))
        .map_with(|value, extra| {
            Expression::new(
                ExpressionKind::Literal(Literal::Bool(value)),
                span_to_range(extra.span()),
            )
        }),
    )
}

pub fn float_literal_parser<'src>(
) -> impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone {
    pad(
        one_of("0123456789")
            .repeated()
            .at_least(1)
            .collect::<String>()
            .then_ignore(just::<_, _, RichError<'src>>('.'))
            .then(
                one_of("0123456789")
                    .repeated()
                    .at_least(1)
                    .collect::<String>(),
            )
            .map(|(whole, fractional)| format!("{whole}.{fractional}"))
            .try_map(|value, span| {
                value
                    .parse::<f64>()
                    .map(OrderedFloat)
                    .map_err(|_| Rich::custom(span, "invalid float literal".to_string()))
            })
            .map_with(|value, extra| {
                Expression::new(
                    ExpressionKind::Literal(Literal::Float(value)),
                    span_to_range(extra.span()),
                )
            }),
    )
}

pub fn integer_literal_parser<'src>(
) -> impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone {
    pad(
        text::int::<_, RichError<'src>>(10)
            .from_str::<i64>()
            .unwrapped()
            .map_with(|value, extra| {
                Expression::new(
                    ExpressionKind::Literal(Literal::Integer(value)),
                    span_to_range(extra.span()),
                )
            }),
    )
}
