use chumsky::prelude::*;

use crate::ast::{
    BinaryOperator, Expression, ExpressionKind, StructLiteralField, TypeSyntax, UnaryOperator,
};

use super::common::{keyword, span_to_range, symbol, token, RichError};
use super::identifiers::{identifier_parser, member_identifier_parser};
use super::literals::{
    bool_literal_parser, float_literal_parser, integer_literal_parser, string_literal_parser,
};

enum Postfix {
    Call(Vec<Expression>),
    Index(Expression),
    Member(crate::ast::Identifier),
}

pub fn expression_parser<'src>(
) -> impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone {
    recursive(|expression| {
        let variable_or_struct = identifier_parser()
            .then(
                struct_literal_fields_parser(expression.clone())
                    .or_not(),
            )
            .map(|(name, fields)| match fields {
                Some(fields) => {
                    let span = name.span.start
                        ..fields
                            .last()
                            .map(|field| field.span.end)
                            .unwrap_or(name.span.end + 2);
                    Expression::new(
                        ExpressionKind::StructLiteral {
                            name: TypeSyntax {
                                name: name.name.clone(),
                                span: name.span.clone(),
                            },
                            fields,
                        },
                        span,
                    )
                }
                None => Expression::new(ExpressionKind::Variable(name.clone()), name.span.clone()),
            });

        let float_cast = keyword("float")
            .ignore_then(expression.clone().delimited_by(symbol('('), symbol(')')))
            .map_with(|expr, extra| {
                Expression::new(
                    ExpressionKind::Cast {
                        target: TypeSyntax {
                            name: "float".to_string(),
                            span: span_to_range(extra.span()),
                        },
                        expr: Box::new(expr),
                    },
                    span_to_range(extra.span()),
                )
            });

        let array_literal = expression
            .clone()
            .separated_by(symbol(','))
            .collect::<Vec<_>>()
            .delimited_by(symbol('['), symbol(']'))
            .map_with(|elements, extra| {
                Expression::new(
                    ExpressionKind::ArrayLiteral(elements),
                    span_to_range(extra.span()),
                )
            });

        let atom = choice((
            float_cast,
            bool_literal_parser(),
            float_literal_parser(),
            string_literal_parser(),
            integer_literal_parser(),
            array_literal,
            variable_or_struct,
            expression.clone().delimited_by(symbol('('), symbol(')')),
        ));

        let postfix = choice((
            expression
                .clone()
                .separated_by(symbol(','))
                .collect::<Vec<_>>()
                .delimited_by(symbol('('), symbol(')'))
                .map(Postfix::Call),
            expression
                .clone()
                .delimited_by(symbol('['), symbol(']'))
                .map(Postfix::Index),
            symbol('.')
                .ignore_then(member_identifier_parser())
                .map(Postfix::Member),
        ));

        let call = atom
            .clone()
            .foldl(postfix.repeated(), |target, postfix| match postfix {
                Postfix::Call(arguments) => {
                    let span = target.span.start
                        ..arguments
                            .last()
                            .map(|argument| argument.span.end)
                            .unwrap_or(target.span.end);
                    Expression::new(
                        ExpressionKind::Call {
                            callee: Box::new(target),
                            arguments,
                        },
                        span,
                    )
                }
                Postfix::Index(index) => {
                    let span = target.span.start..index.span.end;
                    Expression::new(
                        ExpressionKind::Index {
                            target: Box::new(target),
                            index: Box::new(index),
                        },
                        span,
                    )
                }
                Postfix::Member(field) => {
                    let span = target.span.start..field.span.end;
                    Expression::new(
                        ExpressionKind::Member {
                            target: Box::new(target),
                            field,
                        },
                        span,
                    )
                }
            });

        let unary = symbol('-')
            .repeated()
            .collect::<Vec<_>>()
            .then(call.clone())
            .map_with(|(operators, expr), extra| {
                if operators.len() % 2 == 0 {
                    expr
                } else {
                    Expression::new(
                        ExpressionKind::Unary {
                            op: UnaryOperator::Negate,
                            expr: Box::new(expr),
                        },
                        span_to_range(extra.span()),
                    )
                }
            });

        let product = unary.clone().foldl(
            choice((
                symbol('*').to(BinaryOperator::Multiply),
                symbol('/').to(BinaryOperator::Divide),
                symbol('%').to(BinaryOperator::Modulo),
            ))
            .then(unary.clone())
            .repeated(),
            |left, (op, right)| {
                let span = left.span.start..right.span.end;
                Expression::new(
                    ExpressionKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    span,
                )
            },
        );

        let additive = product.clone().foldl(
            choice((
                symbol('+').to(BinaryOperator::Add),
                symbol('-').to(BinaryOperator::Subtract),
            ))
            .then(product)
            .repeated(),
            |left, (op, right)| {
                let span = left.span.start..right.span.end;
                Expression::new(
                    ExpressionKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    span,
                )
            },
        );

        let comparison = additive.clone().foldl(
            choice((
                token("==").to(BinaryOperator::Equal),
                token("!=").to(BinaryOperator::NotEqual),
                token("<=").to(BinaryOperator::LessEqual),
                token(">=").to(BinaryOperator::GreaterEqual),
                symbol('<').to(BinaryOperator::Less),
                symbol('>').to(BinaryOperator::Greater),
            ))
            .then(additive)
            .repeated(),
            |left, (op, right)| {
                let span = left.span.start..right.span.end;
                Expression::new(
                    ExpressionKind::Binary {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    span,
                )
            },
        );

        comparison
            .clone()
            .then(
                choice((token("..=").to(true), token("..").to(false)))
                    .then(comparison)
                    .or_not(),
            )
            .map(|(start, range)| match range {
                Some((inclusive, end)) => {
                    let span = start.span.start..end.span.end;
                    Expression::new(
                        ExpressionKind::Range {
                            start: Box::new(start),
                            end: Box::new(end),
                            inclusive,
                        },
                        span,
                    )
                }
                None => start,
            })
    })
}

fn struct_literal_fields_parser<'src>(
    expression: impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, Vec<StructLiteralField>, RichError<'src>> + Clone {
    struct_literal_field_parser(expression)
        .separated_by(symbol(','))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(symbol('{'), symbol('}'))
}

fn struct_literal_field_parser<'src>(
    expression: impl Parser<'src, &'src str, Expression, RichError<'src>> + Clone + 'src,
) -> impl Parser<'src, &'src str, StructLiteralField, RichError<'src>> + Clone {
    member_identifier_parser()
        .then_ignore(symbol(':'))
        .then(expression)
        .map_with(|(name, value), extra| StructLiteralField {
            name,
            value,
            span: span_to_range(extra.span()),
        })
}
