use chumsky::prelude::*;

use crate::ast::syntax::{
    AssignStatement, AssignTarget, Block, Expression, ExpressionStatement, ForStatement,
    IfStatement, LetStatement, ReturnStatement, Statement, WhileStatement,
};

use super::common::{keyword, span_to_range, symbol, RichError};
use super::identifiers::{identifier_parser, member_identifier_parser, type_name_parser};

pub fn block_parser<'src, P>(
    expression: P,
) -> impl Parser<'src, &'src str, Block, RichError<'src>> + Clone
where
    P: Parser<'src, &'src str, Expression, RichError<'src>> + Clone + 'src,
{
    recursive(move |block| {
        let if_statement = keyword("if")
            .ignore_then(expression.clone())
            .then(block.clone())
            .then(keyword("else").ignore_then(block.clone()).or_not())
            .map_with(|((condition, then_block), else_block), extra| {
                Statement::If(IfStatement {
                    condition,
                    then_block,
                    else_block,
                    span: span_to_range(extra.span()),
                })
            });

        let while_statement = keyword("while")
            .ignore_then(expression.clone())
            .then(block.clone())
            .map_with(|(condition, body), extra| {
                Statement::While(WhileStatement {
                    condition,
                    body,
                    span: span_to_range(extra.span()),
                })
            });

        let for_statement = keyword("for")
            .ignore_then(identifier_parser())
            .then_ignore(keyword("in"))
            .then(expression.clone())
            .then(block.clone())
            .map_with(|((binding, iterable), body), extra| {
                Statement::For(ForStatement {
                    binding,
                    iterable,
                    body,
                    span: span_to_range(extra.span()),
                })
            });

        let let_statement = keyword("let")
            .ignore_then(identifier_parser())
            .then(symbol(':').ignore_then(type_name_parser()).or_not())
            .then_ignore(symbol('='))
            .then(expression.clone())
            .then_ignore(symbol(';'))
            .map_with(|((name, type_ann), value), extra| {
                Statement::Let(LetStatement {
                    name,
                    type_ann,
                    value,
                    span: span_to_range(extra.span()),
                })
            });

        let assign_statement = identifier_parser()
            .then(
                symbol('.')
                    .ignore_then(member_identifier_parser())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(head, fields)| {
                let mut target = AssignTarget::Variable(head);
                for field in fields {
                    let span = target.span().start..field.span.end;
                    target = AssignTarget::Field {
                        target: Box::new(target),
                        field,
                        span,
                    };
                }
                target
            })
            .then_ignore(symbol('='))
            .then(expression.clone())
            .then_ignore(symbol(';'))
            .map_with(|(target, value), extra| {
                Statement::Assign(AssignStatement {
                    target,
                    value,
                    span: span_to_range(extra.span()),
                })
            });

        let return_statement = keyword("return")
            .ignore_then(expression.clone())
            .then_ignore(symbol(';'))
            .map_with(|expression, extra| {
                Statement::Return(ReturnStatement {
                    expression,
                    span: span_to_range(extra.span()),
                })
            });

        let expression_statement = expression
            .clone()
            .then_ignore(symbol(';'))
            .map_with(|expression, extra| {
                Statement::Expression(ExpressionStatement {
                    expression,
                    span: span_to_range(extra.span()),
                })
            });

        let break_statement = keyword("break")
            .then_ignore(symbol(';'))
            .map_with(|_, extra| Statement::Break(span_to_range(extra.span())));

        let continue_statement = keyword("continue")
            .then_ignore(symbol(';'))
            .map_with(|_, extra| Statement::Continue(span_to_range(extra.span())));

        choice((
            if_statement,
            while_statement,
            for_statement,
            let_statement,
            assign_statement,
            return_statement,
            break_statement,
            continue_statement,
            expression_statement,
        ))
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(symbol('{'), symbol('}'))
            .map_with(|statements, extra| Block {
                statements,
                span: span_to_range(extra.span()),
            })
    })
}
