use crate::ast::{Expression, ExpressionKind};

use super::CompileError;

pub(super) fn direct_callee_name(callee: &Expression) -> Result<String, CompileError> {
    match &callee.kind {
        ExpressionKind::Variable(identifier) => Ok(identifier.name.clone()),
        _ => Err(CompileError(
            "only direct function calls are supported in the current compiler".to_string(),
        )),
    }
}
