use chumsky::prelude::*;

use crate::ast::syntax::{PlatformGroup, PlatformsMetadata};

use super::common::{span_to_range, symbol, token, RichError};
use super::identifiers::identifier_parser;

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
