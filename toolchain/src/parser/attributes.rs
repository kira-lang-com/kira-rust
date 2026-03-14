use chumsky::prelude::*;

use crate::ast::syntax::Attribute;

use super::common::{span_to_range, symbol, RichError};
use super::identifiers::{attribute_name_parser, identifier_parser};

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
) -> impl Parser<'src, &'src str, Vec<crate::ast::syntax::Identifier>, RichError<'src>> + Clone {
    identifier_parser()
        .separated_by(symbol(','))
        .collect::<Vec<_>>()
        .delimited_by(symbol('('), symbol(')'))
}
