use chumsky::prelude::*;

use crate::ast::{Identifier, TypeSyntax};

use super::infrastructure::{pad, span_to_range, symbol, RichError};

pub fn identifier_parser<'src>(
) -> impl Parser<'src, &'src str, Identifier, RichError<'src>> + Clone {
    pad(
        text::ascii::ident::<_, RichError<'src>>()
            .try_map(|name: &str, span| {
                if matches!(
                    name,
                    "func"
                        | "import"
                        | "return"
                        | "let"
                        | "struct"
                        | "native"
                        | "runtime"
                        | "script"
                        | "auto"
                        | "if"
                        | "else"
                        | "for"
                        | "in"
                        | "while"
                        | "break"
                        | "continue"
                        | "true"
                        | "false"
                        | "float"
                ) {
                    Err(Rich::custom(span, format!("`{name}` is reserved")))
                } else {
                    Ok(name.to_string())
                }
            })
            .map_with(|name, extra| Identifier {
                name,
                span: span_to_range(extra.span()),
            }),
    )
}

pub fn attribute_name_parser<'src>(
) -> impl Parser<'src, &'src str, Identifier, RichError<'src>> + Clone {
    pad(
        text::ascii::ident::<_, RichError<'src>>()
            .try_map(|name: &str, span| {
                let starts_uppercase = name
                    .chars()
                    .next()
                    .map(|ch| ch.is_ascii_uppercase())
                    .unwrap_or(false);

                if starts_uppercase {
                    Ok(name.to_string())
                } else {
                    Err(Rich::custom(
                        span,
                        "attribute names must be PascalCase".to_string(),
                    ))
                }
            })
            .map_with(|name, extra| Identifier {
                name,
                span: span_to_range(extra.span()),
            }),
    )
}

pub fn type_name_parser<'src>(
) -> impl Parser<'src, &'src str, TypeSyntax, RichError<'src>> + Clone {
    recursive(|type_name| {
        let bare = pad(
            text::ascii::ident::<_, RichError<'src>>()
                .map(|name: &str| name.to_string())
                .map_with(|name, extra| TypeSyntax {
                    name,
                    span: span_to_range(extra.span()),
                }),
        );

        let array = type_name
            .clone()
            .delimited_by(symbol('['), symbol(']'))
            .map_with(|inner: TypeSyntax, extra| TypeSyntax {
                name: format!("[{}]", inner.name),
                span: span_to_range(extra.span()),
            });

        choice((array, bare))
    })
}

pub fn member_identifier_parser<'src>(
) -> impl Parser<'src, &'src str, Identifier, RichError<'src>> + Clone {
    pad(
        text::ascii::ident::<_, RichError<'src>>()
            .map(|name: &str| name.to_string())
            .map_with(|name, extra| Identifier {
                name,
                span: span_to_range(extra.span()),
            }),
    )
}
