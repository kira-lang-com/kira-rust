use ordered_float::OrderedFloat;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::runtime::Value;

use super::{machine::Vm, RuntimeError};

pub(super) fn call_builtin(
    vm: &mut Vm,
    name: &str,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    match name {
        "printIn" => {
            if args.len() != 1 {
                return Err(RuntimeError(format!(
                    "`printIn` expects 1 argument but got {}",
                    args.len()
                )));
            }

            vm.output.push(args[0].display());
            Ok(Value::Unit)
        }
        "abs" => {
            let value = expect_int_arg(name, &args, 0)?;
            Ok(Value::Int(value.abs()))
        }
        "pow" => {
            let base = expect_int_arg(name, &args, 0)?;
            let exponent = expect_int_arg(name, &args, 1)?;
            if exponent < 0 {
                return Err(RuntimeError(
                    "`pow` currently requires a non-negative exponent".to_string(),
                ));
            }
            Ok(Value::Int(base.pow(exponent as u32)))
        }
        "max" => {
            let left = expect_int_arg(name, &args, 0)?;
            let right = expect_int_arg(name, &args, 1)?;
            Ok(Value::Int(left.max(right)))
        }
        "min" => {
            let left = expect_int_arg(name, &args, 0)?;
            let right = expect_int_arg(name, &args, 1)?;
            Ok(Value::Int(left.min(right)))
        }
        "Foundation.Math.sqrt" => {
            let value = expect_float_arg(name, &args, 0)?;
            if value < 0.0 {
                return Err(RuntimeError(format!(
                    "`{name}` requires a non-negative operand"
                )));
            }
            Ok(Value::Float(OrderedFloat(value.sqrt())))
        }
        "Foundation.Math.floor" => {
            let value = expect_float_arg(name, &args, 0)?;
            Ok(Value::Int(value.floor() as i64))
        }
        "Foundation.Math.ceil" => {
            let value = expect_float_arg(name, &args, 0)?;
            Ok(Value::Int(value.ceil() as i64))
        }
        "Foundation.Math.round" => {
            let value = expect_float_arg(name, &args, 0)?;
            Ok(Value::Int(value.round() as i64))
        }
        "Foundation.Math.pi" => Ok(Value::Float(OrderedFloat(std::f64::consts::PI))),
        "Foundation.Math.clamp" => {
            let value = expect_int_arg(name, &args, 0)?;
            let low = expect_int_arg(name, &args, 1)?;
            let high = expect_int_arg(name, &args, 2)?;
            Ok(Value::Int(value.clamp(low, high)))
        }
        "Foundation.Math.lerp" => {
            let a = expect_float_arg(name, &args, 0)?;
            let b = expect_float_arg(name, &args, 1)?;
            let t = expect_float_arg(name, &args, 2)?;
            Ok(Value::Float(OrderedFloat(a + ((b - a) * t))))
        }
        "Foundation.Math.sign" => {
            let value = expect_int_arg(name, &args, 0)?;
            Ok(Value::Int(value.signum()))
        }
        "Foundation.String.length" => {
            let value = expect_string_arg(name, &args, 0)?;
            Ok(Value::Int(value.chars().count() as i64))
        }
        "Foundation.String.concat" => {
            let left = expect_string_arg(name, &args, 0)?;
            let right = expect_string_arg(name, &args, 1)?;
            Ok(Value::String(format!("{left}{right}")))
        }
        "Foundation.String.contains" => {
            let value = expect_string_arg(name, &args, 0)?;
            let sub = expect_string_arg(name, &args, 1)?;
            Ok(Value::Bool(value.contains(sub)))
        }
        "Foundation.String.uppercase" => {
            let value = expect_string_arg(name, &args, 0)?;
            Ok(Value::String(value.to_uppercase()))
        }
        "Foundation.String.lowercase" => {
            let value = expect_string_arg(name, &args, 0)?;
            Ok(Value::String(value.to_lowercase()))
        }
        "Foundation.String.repeat" => {
            let value = expect_string_arg(name, &args, 0)?;
            let count = expect_int_arg(name, &args, 1)?;
            if count < 0 {
                return Err(RuntimeError(format!(
                    "`{name}` requires a non-negative repeat count"
                )));
            }
            Ok(Value::String(value.repeat(count as usize)))
        }
        "Foundation.Random.int" => {
            let min = expect_int_arg(name, &args, 0)?;
            let max = expect_int_arg(name, &args, 1)?;
            if max < min {
                return Err(RuntimeError(format!(
                    "`{name}` requires `max >= min`"
                )));
            }
            let span = (max - min + 1) as u64;
            let value = min + (next_random_u64(vm) % span) as i64;
            Ok(Value::Int(value))
        }
        "Foundation.Random.float" => {
            let min = expect_float_arg(name, &args, 0)?;
            let max = expect_float_arg(name, &args, 1)?;
            if max < min {
                return Err(RuntimeError(format!(
                    "`{name}` requires `max >= min`"
                )));
            }
            if (max - min).abs() < f64::EPSILON {
                return Ok(Value::Float(OrderedFloat(min)));
            }
            let unit = (next_random_u64(vm) as f64) / (u64::MAX as f64);
            Ok(Value::Float(OrderedFloat(min + ((max - min) * unit))))
        }
        "Foundation.Random.bool" => Ok(Value::Bool(next_random_u64(vm) & 1 == 0)),
        "Foundation.Time.now" => {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| RuntimeError(format!("`{name}` failed: {error}")))?;
            Ok(Value::Int(now.as_secs() as i64))
        }
        "Foundation.Time.delta" => Ok(Value::Float(OrderedFloat(
            vm.started_at.elapsed().as_secs_f64(),
        ))),
        _ => Err(RuntimeError(format!("unknown builtin `{name}`"))),
    }
}

fn next_random_u64(vm: &mut Vm) -> u64 {
    vm.rng_state = vm
        .rng_state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    vm.rng_state
}

fn expect_int_arg(name: &str, args: &[Value], index: usize) -> Result<i64, RuntimeError> {
    match args.get(index) {
        Some(Value::Int(value)) => Ok(*value),
        Some(value) => Err(RuntimeError(format!(
            "`{name}` expected int at argument {index}, got {:?}",
            value
        ))),
        None => Err(RuntimeError(format!(
            "`{name}` expected argument {index}, but it was missing"
        ))),
    }
}

fn expect_float_arg(name: &str, args: &[Value], index: usize) -> Result<f64, RuntimeError> {
    match args.get(index) {
        Some(Value::Float(value)) => Ok(value.0),
        Some(value) => Err(RuntimeError(format!(
            "`{name}` expected float at argument {index}, got {:?}",
            value
        ))),
        None => Err(RuntimeError(format!(
            "`{name}` expected argument {index}, but it was missing"
        ))),
    }
}

fn expect_string_arg<'a>(
    name: &str,
    args: &'a [Value],
    index: usize,
) -> Result<&'a str, RuntimeError> {
    match args.get(index) {
        Some(Value::String(value)) => Ok(value),
        Some(value) => Err(RuntimeError(format!(
            "`{name}` expected string at argument {index}, got {:?}",
            value
        ))),
        None => Err(RuntimeError(format!(
            "`{name}` expected argument {index}, but it was missing"
        ))),
    }
}
