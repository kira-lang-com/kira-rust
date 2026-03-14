mod control_flow;
mod let_assign;

use std::collections::HashMap;

use crate::ast::syntax::Statement;
use crate::compiler::{BuiltinFunction, CompileError, FunctionSignature};
use crate::runtime::type_system::TypeSystem;

use super::expressions::{analyze_assignment, analyze_expression};
use super::types::LocalBinding;

pub use control_flow::{analyze_for_statement, analyze_if_statement, analyze_while_statement};
pub use let_assign::analyze_let_statement;

pub fn analyze_statement(
    statement: &Statement,
    locals: &mut HashMap<String, LocalBinding>,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
    builtins: &HashMap<String, BuiltinFunction>,
    loop_depth: usize,
) -> Result<bool, CompileError> {
    match statement {
        Statement::Let(statement) => {
            analyze_let_statement(statement, locals, types, signatures, builtins)
        }
        Statement::Assign(statement) => {
            analyze_assignment(statement, locals, types, signatures, builtins)
        }
        Statement::Expression(statement) => {
            let profile = analyze_expression(
                &statement.expression,
                locals,
                types,
                signatures,
                builtins,
                None,
            )?;
            Ok(profile.native_eligible)
        }
        Statement::Return(statement) => {
            let profile = analyze_expression(
                &statement.expression,
                locals,
                types,
                signatures,
                builtins,
                None,
            )?;
            Ok(profile.native_eligible)
        }
        Statement::If(statement) => {
            analyze_if_statement(statement, locals, types, signatures, builtins, loop_depth)
        }
        Statement::While(statement) => {
            analyze_while_statement(statement, locals, types, signatures, builtins, loop_depth)
        }
        Statement::For(statement) => {
            analyze_for_statement(statement, locals, types, signatures, builtins, loop_depth)
        }
        Statement::Break(_) | Statement::Continue(_) => {
            if loop_depth == 0 {
                return Err(CompileError(
                    "loop control can only be used inside a loop".to_string(),
                ));
            }
            Ok(true)
        }
    }
}
