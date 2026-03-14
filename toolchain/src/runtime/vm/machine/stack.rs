use ordered_float::OrderedFloat;

use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

pub fn pop_pair(stack: &mut Vec<Value>) -> Result<(Value, Value), RuntimeError> {
    let right = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading rhs".to_string()))?;
    let left = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading lhs".to_string()))?;
    Ok((left, right))
}

pub fn apply_numeric_op(
    stack: &mut Vec<Value>,
    int_op: fn(i64, i64) -> i64,
    float_op: fn(f64, f64) -> f64,
) -> Result<Value, RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Ok(Value::Int(int_op(left, right))),
        (Value::Float(left), Value::Float(right)) => {
            Ok(Value::Float(OrderedFloat(float_op(left.0, right.0))))
        }
        (left, right) => Err(RuntimeError(format!(
            "expected matching numeric operands, got {:?} and {:?}",
            left, right
        ))),
    }
}

pub fn apply_numeric_division(stack: &mut Vec<Value>) -> Result<Value, RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            if right == 0 {
                return Err(RuntimeError("division by zero".to_string()));
            }
            Ok(Value::Int(left / right))
        }
        (Value::Float(left), Value::Float(right)) => {
            if right.0 == 0.0 {
                return Err(RuntimeError("division by zero".to_string()));
            }
            Ok(Value::Float(OrderedFloat(left.0 / right.0)))
        }
        (left, right) => Err(RuntimeError(format!(
            "expected matching numeric operands, got {:?} and {:?}",
            left, right
        ))),
    }
}

pub fn compare_ordered_pair(
    stack: &mut Vec<Value>,
    comparison: fn(f64, f64) -> bool,
) -> Result<Value, RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Ok(Value::Bool(comparison(left as f64, right as f64))),
        (Value::Float(left), Value::Float(right)) => Ok(Value::Bool(comparison(left.0, right.0))),
        (left, right) => Err(RuntimeError(format!(
            "expected ordered numeric operands, got {:?} and {:?}",
            left, right
        ))),
    }
}

pub fn pop_int_pair(stack: &mut Vec<Value>) -> Result<(i64, i64), RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Ok((left, right)),
        (left, right) => Err(RuntimeError(format!(
            "expected integer operands, got {:?} and {:?}",
            left, right
        ))),
    }
}

pub fn store_struct_field(
    target: &mut Value,
    path: &[usize],
    value: Value,
) -> Result<(), RuntimeError> {
    let Some((field_index, rest)) = path.split_first() else {
        *target = value;
        return Ok(());
    };

    let Value::Struct(struct_value) = target else {
        return Err(RuntimeError(format!(
            "expected struct while storing nested field, got {:?}",
            target
        )));
    };

    let (_, field_value) = struct_value.fields.get_mut(*field_index).ok_or_else(|| {
        RuntimeError(format!(
            "struct `{}` has no field at index {}",
            struct_value.type_name, field_index
        ))
    })?;

    if rest.is_empty() {
        *field_value = value;
        return Ok(());
    }

    store_struct_field(field_value, rest, value)
}
