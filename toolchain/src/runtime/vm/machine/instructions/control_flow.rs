use crate::compiler::CompiledModule;
use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

use super::super::vm::Vm;

pub fn execute_jump_if_false(
    stack: &mut Vec<Value>,
    target: usize,
) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while evaluating branch".to_string()))?;
    match value {
        Value::Bool(false) => Ok(Some(target)),
        Value::Bool(true) => Ok(None),
        value => Err(RuntimeError(format!(
            "expected bool condition for branch, got {:?}",
            value
        ))),
    }
}

pub fn execute_call(
    stack: &mut Vec<Value>,
    module: &CompiledModule,
    vm: &mut Vm,
    function: &str,
    arg_count: usize,
) -> Result<Option<usize>, RuntimeError> {
    let mut call_args = Vec::with_capacity(arg_count);
    for _ in 0..arg_count {
        call_args.push(
            stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while preparing call".to_string()))?,
        );
    }
    call_args.reverse();
    let result = vm.run_function(module, function, call_args)?;
    if result != Value::Unit {
        stack.push(result);
    }
    Ok(None)
}
