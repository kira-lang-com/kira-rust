use ordered_float::OrderedFloat;

use crate::compiler::{Chunk, CompiledModule, Instruction};
use crate::runtime::value::StructValue;
use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

use super::stack::{
    apply_numeric_division, apply_numeric_op, compare_ordered_pair, pop_int_pair, pop_pair,
    store_struct_field,
};
use super::vm::Vm;

pub fn execute_instruction(
    instruction: &Instruction,
    stack: &mut Vec<Value>,
    locals: &mut Vec<Value>,
    chunk: &Chunk,
    module: &CompiledModule,
    vm: &mut Vm,
) -> Result<Option<usize>, RuntimeError> {
    match instruction {
        Instruction::LoadConst(index) => {
            stack.push(
                chunk
                    .constants
                    .get(*index)
                    .cloned()
                    .ok_or_else(|| RuntimeError(format!("invalid constant index {index}")))?,
            );
            Ok(None)
        }
        Instruction::LoadLocal(index) => {
            stack.push(
                locals
                    .get(*index)
                    .cloned()
                    .ok_or_else(|| RuntimeError(format!("invalid local index {index}")))?,
            );
            Ok(None)
        }
        Instruction::StoreLocal(index) => {
            let value = stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while storing local".to_string()))?;

            if *index >= locals.len() {
                locals.resize(index + 1, Value::Unit);
            }

            locals[*index] = value;
            Ok(None)
        }
        Instruction::Negate => {
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
        Instruction::CastIntToFloat => {
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
        Instruction::Add => {
            let result =
                apply_numeric_op(stack, |left, right| left + right, |left, right| left + right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::Subtract => {
            let result =
                apply_numeric_op(stack, |left, right| left - right, |left, right| left - right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::Multiply => {
            let result =
                apply_numeric_op(stack, |left, right| left * right, |left, right| left * right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::Divide => {
            let result = apply_numeric_division(stack)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::Modulo => {
            let (left, right) = pop_int_pair(stack)?;
            if right == 0 {
                return Err(RuntimeError("modulo by zero".to_string()));
            }
            stack.push(Value::Int(left % right));
            Ok(None)
        }
        Instruction::Less => {
            let result = compare_ordered_pair(stack, |left, right| left < right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::Greater => {
            let result = compare_ordered_pair(stack, |left, right| left > right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::Equal => {
            let (left, right) = pop_pair(stack)?;
            stack.push(Value::Bool(left == right));
            Ok(None)
        }
        Instruction::NotEqual => {
            let (left, right) = pop_pair(stack)?;
            stack.push(Value::Bool(left != right));
            Ok(None)
        }
        Instruction::LessEqual => {
            let result = compare_ordered_pair(stack, |left, right| left <= right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::GreaterEqual => {
            let result = compare_ordered_pair(stack, |left, right| left >= right)?;
            stack.push(result);
            Ok(None)
        }
        Instruction::BuildArray { element_count, .. } => {
            execute_build_array(stack, *element_count)?;
            Ok(None)
        }
        Instruction::BuildStruct {
            type_id,
            field_count,
        } => {
            execute_build_struct(stack, module, *type_id, *field_count)?;
            Ok(None)
        }
        Instruction::ArrayLength => {
            execute_array_length(stack)?;
            Ok(None)
        }
        Instruction::ArrayIndex => {
            execute_array_index(stack)?;
            Ok(None)
        }
        Instruction::StructField(index) => {
            execute_struct_field(stack, *index)?;
            Ok(None)
        }
        Instruction::StoreLocalField { local, path } => {
            execute_store_local_field(stack, locals, *local, path)?;
            Ok(None)
        }
        Instruction::ArrayAppendLocal(index) => {
            execute_array_append_local(stack, locals, *index)?;
            Ok(None)
        }
        Instruction::JumpIfFalse(target) => execute_jump_if_false(stack, *target),
        Instruction::Jump(target) => Ok(Some(*target)),
        Instruction::Call { function, arg_count } => {
            execute_call(stack, module, vm, function, *arg_count)?;
            Ok(None)
        }
        Instruction::Pop => {
            stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while discarding expression result".to_string()))?;
            Ok(None)
        }
        Instruction::Return => {
            // Signal return by returning a special marker
            // The execution loop will handle this
            Ok(Some(usize::MAX))
        }
    }
}

fn execute_build_array(stack: &mut Vec<Value>, element_count: usize) -> Result<(), RuntimeError> {
    let mut values = Vec::with_capacity(element_count);
    for _ in 0..element_count {
        values.push(
            stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while building array".to_string()))?,
        );
    }
    values.reverse();
    stack.push(Value::Array(values));
    Ok(())
}

fn execute_build_struct(
    stack: &mut Vec<Value>,
    module: &CompiledModule,
    type_id: crate::runtime::type_system::TypeId,
    field_count: usize,
) -> Result<(), RuntimeError> {
    let mut values = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        values.push(
            stack
                .pop()
                .ok_or_else(|| RuntimeError("stack underflow while building struct".to_string()))?,
        );
    }
    values.reverse();

    let crate::runtime::type_system::KiraType::Struct(struct_type) = module.types.get(type_id)
    else {
        return Err(RuntimeError(format!(
            "invalid struct type id {:?} in bytecode",
            type_id
        )));
    };

    if struct_type.fields.len() != values.len() {
        return Err(RuntimeError(format!(
            "struct `{}` expected {} fields but bytecode provided {}",
            struct_type.name,
            struct_type.fields.len(),
            values.len()
        )));
    }

    stack.push(Value::Struct(StructValue {
        type_name: struct_type.name.clone(),
        fields: struct_type
            .fields
            .iter()
            .map(|field| field.name.clone())
            .zip(values.into_iter())
            .collect(),
    }));
    Ok(())
}

fn execute_array_length(stack: &mut Vec<Value>) -> Result<(), RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading array length".to_string()))?;
    match value {
        Value::Array(values) => stack.push(Value::Int(values.len() as i64)),
        value => {
            return Err(RuntimeError(format!(
                "expected array for `.length`, got {:?}",
                value
            )))
        }
    }
    Ok(())
}

fn execute_array_index(stack: &mut Vec<Value>) -> Result<(), RuntimeError> {
    let index = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading array index".to_string()))?;
    let target = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading array target".to_string()))?;
    let Value::Int(index) = index else {
        return Err(RuntimeError(format!(
            "array index must be int, got {:?}",
            index
        )));
    };
    let Value::Array(values) = target else {
        return Err(RuntimeError(format!(
            "expected array for indexing, got {:?}",
            target
        )));
    };
    if index < 0 || index as usize >= values.len() {
        return Err(RuntimeError(format!(
            "array index {} out of bounds for length {}",
            index,
            values.len()
        )));
    }
    stack.push(values[index as usize].clone());
    Ok(())
}

fn execute_struct_field(stack: &mut Vec<Value>, index: usize) -> Result<(), RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading struct field".to_string()))?;
    match value {
        Value::Struct(struct_value) => {
            let (_, field_value) = struct_value.fields.get(index).ok_or_else(|| {
                RuntimeError(format!(
                    "struct `{}` has no field at index {}",
                    struct_value.type_name, index
                ))
            })?;
            stack.push(field_value.clone());
        }
        value => {
            return Err(RuntimeError(format!(
                "expected struct for field access, got {:?}",
                value
            )))
        }
    }
    Ok(())
}

fn execute_store_local_field(
    stack: &mut Vec<Value>,
    locals: &mut Vec<Value>,
    local: usize,
    path: &[usize],
) -> Result<(), RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while storing struct field".to_string()))?;
    let target = locals
        .get_mut(local)
        .ok_or_else(|| RuntimeError(format!("invalid local index {local}")))?;
    store_struct_field(target, path, value)?;
    Ok(())
}

fn execute_array_append_local(
    stack: &mut Vec<Value>,
    locals: &mut Vec<Value>,
    index: usize,
) -> Result<(), RuntimeError> {
    let value = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while appending to array".to_string()))?;
    let local = locals
        .get_mut(index)
        .ok_or_else(|| RuntimeError(format!("invalid local index {index}")))?;
    match local {
        Value::Array(values) => {
            values.push(value);
            stack.push(Value::Unit);
        }
        other => {
            return Err(RuntimeError(format!(
                "expected array local for append, got {:?}",
                other
            )))
        }
    }
    Ok(())
}

fn execute_jump_if_false(
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

fn execute_call(
    stack: &mut Vec<Value>,
    module: &CompiledModule,
    vm: &mut Vm,
    function: &str,
    arg_count: usize,
) -> Result<(), RuntimeError> {
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
    Ok(())
}
