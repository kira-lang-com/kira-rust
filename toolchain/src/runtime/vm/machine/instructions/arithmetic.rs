use ordered_float::OrderedFloat;

use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

use super::super::stack::{
    apply_numeric_division, apply_numeric_op, compare_ordered_pair, pop_int_pair, pop_pair,
};

pub fn execute_negate(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while negating value".to_string()))?;
    match value {
        Value::Int(value) => stack.push(Value::Int(-value)),
        Value::Float(value) => stack.push(Value::Float(OrderedFloat(-value.0))),
        value => {
            return Err(RuntimeError(format!(
                "expected numeric operand for negation, got {:?}",
                value
            )))
        }
    }
    Ok(None)
}

pub fn execute_cast_int_to_float(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while casting value".to_string()))?;
    match value {
        Value::Int(value) => stack.push(Value::Float(OrderedFloat(value as f64))),
        Value::Float(value) => stack.push(Value::Float(value)),
        value => {
            return Err(RuntimeError(format!(
                "expected `int` or `float` for float cast, got {:?}",
                value
            )))
        }
    }
    Ok(None)
}

pub fn execute_add(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result =
        apply_numeric_op(stack, |left, right| left + right, |left, right| left + right)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_subtract(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result =
        apply_numeric_op(stack, |left, right| left - right, |left, right| left - right)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_multiply(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result =
        apply_numeric_op(stack, |left, right| left * right, |left, right| left * right)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_divide(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result = apply_numeric_division(stack)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_modulo(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let (left, right) = pop_int_pair(stack)?;
    if right == 0 {
        return Err(RuntimeError("modulo by zero".to_string()));
    }
    stack.push(Value::Int(left % right));
    Ok(None)
}

pub fn execute_less(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result = compare_ordered_pair(stack, |left, right| left < right)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_greater(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result = compare_ordered_pair(stack, |left, right| left > right)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_equal(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    stack.push(Value::Bool(left == right));
    Ok(None)
}

pub fn execute_not_equal(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    stack.push(Value::Bool(left != right));
    Ok(None)
}

pub fn execute_less_equal(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result = compare_ordered_pair(stack, |left, right| left <= right)?;
    stack.push(result);
    Ok(None)
}

pub fn execute_greater_equal(stack: &mut Vec<Value>) -> Result<Option<usize>, RuntimeError> {
    let result = compare_ordered_pair(stack, |left, right| left >= right)?;
    stack.push(result);
    Ok(None)
}
