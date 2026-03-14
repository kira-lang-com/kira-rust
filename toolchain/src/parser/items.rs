use chumsky::prelude::*;

use crate::ast::syntax::{
    ExecutionMode, FunctionDefinition, Import, Parameter, SourceFile, StructDefinition,
    StructField, TopLevelItem,
};

use super::attributes::attributes_parser;
use super::common::{keyword, span_to_range, symbol, token, RichError};
use super::expressions::expression_parser;
use super::identifiers::{identifier_parser, member_identifier_parser, type_name_parser};
use super::platforms::platforms_parser;
use super::statements::block_parser;

pub fn program_parser<'src>(
) -> impl Parser<'src, &'src str, SourceFile, RichError<'src>> + Clone {
    platforms_parser()
        .or_not()
        .then(
            import_parser()
        .repeated()
        .collect::<Vec<_>>()
        )
        .then(
            item_parser()
        .repeated()
        .collect::<Vec<_>>()
        )
        .then_ignore(end())
        .map(|((platforms, imports), items)| SourceFile {
            imports,
            platforms,
            items,
        })
}

fn item_parser<'src>(
) -> impl Parser<'src, &'src str, TopLevelItem, RichError<'src>> + Clone {
    choice((struct_parser(), function_parser()))
}

fn struct_parser<'src>(
) -> impl Parser<'src, &'src str, TopLevelItem, RichError<'src>> + Clone {
    keyword("struct")
        .ignore_then(identifier_parser())
        .then(struct_fields_parser())
        .map_with(|(name, fields), extra| {
            TopLevelItem::Struct(StructDefinition {
                name,
                fields,
                span: span_to_range(extra.span()),
            })
        })
}

fn struct_fields_parser<'src>(
) -> impl Parser<'src, &'src str, Vec<StructField>, RichError<'src>> + Clone {
    struct_field_parser()
        .separated_by(symbol(','))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(symbol('{'), symbol('}'))
}

fn struct_field_parser<'src>(
) -> impl Parser<'src, &'src str, StructField, RichError<'src>> + Clone {
    member_identifier_parser()
        .then_ignore(symbol(':'))
        .then(type_name_parser())
        .map_with(|(name, type_name), extra| StructField {
            name,
            type_name,
            span: span_to_range(extra.span()),
        })
}

fn function_parser<'src>(
) -> impl Parser<'src, &'src str, TopLevelItem, RichError<'src>> + Clone {
    let expression = expression_parser();

    attributes_parser()
        .then(execution_mode_parser().or_not())
        .then_ignore(keyword("func"))
        .then(identifier_parser())
        .then(parameters_parser())
        .then(token("->").ignore_then(type_name_parser()).or_not())
        .then(block_parser(expression))
        .map_with(|(((((attributes, execution_hint), name), params), return_type), body), extra| {
            TopLevelItem::Function(FunctionDefinition {
                attributes,
                execution_hint,
                name,
                params,
                return_type,
                body,
                span: span_to_range(extra.span()),
            })
        })
}

fn execution_mode_parser<'src>(
) -> impl Parser<'src, &'src str, ExecutionMode, RichError<'src>> + Clone {
    choice((
        keyword("native").to(ExecutionMode::Native),
        keyword("runtime").to(ExecutionMode::Runtime),
        keyword("script").to(ExecutionMode::Runtime),
        keyword("auto").to(ExecutionMode::Auto),
    ))
}

fn import_parser<'src>(
) -> impl Parser<'src, &'src str, Import, RichError<'src>> + Clone {
    keyword("import")
        .ignore_then(
            identifier_parser()
                .separated_by(symbol('.'))
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then_ignore(symbol(';'))
        .map_with(|path, extra| Import {
            path,
            span: span_to_range(extra.span()),
        })
}

fn parameters_parser<'src>(
) -> impl Parser<'src, &'src str, Vec<Parameter>, RichError<'src>> + Clone {
    parameter_parser()
        .separated_by(symbol(','))
        .collect::<Vec<_>>()
        .delimited_by(symbol('('), symbol(')'))
}

fn parameter_parser<'src>(
) -> impl Parser<'src, &'src str, Parameter, RichError<'src>> + Clone {
    identifier_parser()
        .then_ignore(symbol(':'))
        .then(type_name_parser())
        .map_with(|(name, type_name), extra| Parameter {
            name,
            type_name,
            span: span_to_range(extra.span()),
        })
}
