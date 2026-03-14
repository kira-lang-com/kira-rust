// Parsing for attributes and platform metadata

use chumsky::prelude::*;

use crate::ast::{Attribute, PlatformGroup, PlatformsMetadata};

use super::infrastructure::{span_to_range, symbol, token, RichError};
use super::core::identifiers::{attribute_name_parser, identifier_parser};

// Attributes

pub fn attributes_parser<'src>(
) -> impl Parser<'src, &'src str, Vec<Attribute>, RichError<'src>> + Clone {
    attribute_parser().repeated().collect::<Vec<_>>()
}

fn attribute_parser<'src>(
) -> impl Parser<'src, &'src str, Attribute, RichError<'src>> + Clone {
    symbol('@')
        .ignore_then(attribute_name_parser())
        .then(attribute_arguments_parser().or_not())
        .map_with(|(name, arguments), extra| Attribute {
            name,
            arguments: arguments.unwrap_or_default(),
            span: span_to_range(extra.span()),
        })
}

fn attribute_arguments_parser<'src>(
) -> impl Parser<'src, &'src str, Vec<crate::ast::Identifier>, RichError<'src>> + Clone {
    identifier_parser()
        .separated_by(symbol(','))
        .collect::<Vec<_>>()
        .delimited_by(symbol('('), symbol(')'))
}

// Platforms

pub fn platforms_parser<'src>(
) -> impl Parser<'src, &'src str, PlatformsMetadata, RichError<'src>> + Clone {
    token("#platforms")
        .ignore_then(platform_group_parser().repeated().collect::<Vec<_>>().delimited_by(
            symbol('{'),
            symbol('}'),
        ))
        .map_with(|groups, extra| PlatformsMetadata {
            groups,
            span: span_to_range(extra.span()),
        })
}

fn platform_group_parser<'src>(
) -> impl Parser<'src, &'src str, PlatformGroup, RichError<'src>> + Clone {
    identifier_parser()
        .then_ignore(symbol('='))
        .then(
            identifier_parser()
                .separated_by(symbol(','))
                .collect::<Vec<_>>()
                .delimited_by(symbol('['), symbol(']')),
        )
        .then_ignore(symbol(';'))
        .map_with(|(name, members), extra| PlatformGroup {
            name,
            members,
            span: span_to_range(extra.span()),
        })
}
