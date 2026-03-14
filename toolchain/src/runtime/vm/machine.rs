use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::compiler::{BackendKind, Chunk, CompiledModule, Instruction};
use crate::runtime::{value::StructValue, Value};

use super::{builtins::call_builtin, RuntimeError};

pub type NativeHandler = fn(&mut Vm, &CompiledModule, Vec<Value>) -> Result<Value, RuntimeError>;

pub struct Vm {
    pub(crate) output: Vec<String>,
    pub(crate) rng_state: u64,
    pub(crate) started_at: Instant,
    native_registry: HashMap<String, NativeHandler>,
}

impl Default for Vm {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x5eed_u64);

        Self {
            output: Vec::new(),
            rng_state: seed ^ 0x9E37_79B9_7F4A_7C15,
            started_at: Instant::now(),
            native_registry: HashMap::new(),
        }
    }
}

impl Vm {
    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn register_native(&mut self, name: impl Into<String>, handler: NativeHandler) {
        self.native_registry.insert(name.into(), handler);
    }

    pub fn run_entry(
        &mut self,
        module: &CompiledModule,
        entry: &str,
    ) -> Result<Value, RuntimeError> {
        self.run_function(module, entry, Vec::new())
    }

    pub fn run_function(
        &mut self,
        module: &CompiledModule,
        name: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if let Some(builtin) = module.builtins.get(name) {
            if builtin.signature.params.len() != args.len() {
                return Err(RuntimeError(format!(
                    "`{name}` expects {} arguments but got {}",
                    builtin.signature.params.len(),
                    args.len()
                )));
            }
            return call_builtin(self, name, args);
        }

        let function = module
            .functions
            .get(name)
            .ok_or_else(|| RuntimeError(format!("unknown function `{name}`")))?;

        if function.signature.params.len() != args.len() {
            return Err(RuntimeError(format!(
                "function `{name}` expects {} arguments but got {}",
                function.signature.params.len(),
                args.len()
            )));
        }

        if let Some(chunk) = function.artifacts.bytecode.as_ref() {
            if function.selected_backend == BackendKind::Native {
                if let Some(handler) = self.native_registry.get(name).copied() {
                    return handler(self, module, args);
                }
            }
            return self.execute_chunk(module, chunk, args);
        }

        match function.selected_backend {
            BackendKind::Native => {
                if let Some(handler) = self.native_registry.get(name).copied() {
                    return handler(self, module, args);
                }
                let artifact = function.artifacts.aot.as_ref().ok_or_else(|| {
                    RuntimeError(format!("function `{name}` has no AOT artifact"))
                })?;
                Err(RuntimeError(format!(
                    "function `{name}` is build-time AOT only and has no VM shadow (symbol `{}`)",
                    artifact.symbol
                )))
            }
            BackendKind::Vm => Err(RuntimeError(format!(
                "function `{name}` is missing bytecode"
            ))),
        }
    }

    fn execute_chunk(
        &mut self,
        module: &CompiledModule,
        chunk: &Chunk,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let mut stack = Vec::new();
        let mut locals = vec![Value::Unit; chunk.local_count.max(args.len())];

        for (index, value) in args.into_iter().enumerate() {
            locals[index] = value;
        }

        let mut ip = 0;
        while let Some(instruction) = chunk.instructions.get(ip) {
            match instruction {
                Instruction::LoadConst(index) => {
                    stack.push(
                        chunk
                            .constants
                            .get(*index)
                            .cloned()
                            .ok_or_else(|| RuntimeError(format!("invalid constant index {index}")))?,
                    );
                }
                Instruction::LoadLocal(index) => {
                    stack.push(
                        locals
                            .get(*index)
                            .cloned()
                            .ok_or_else(|| RuntimeError(format!("invalid local index {index}")))?,
                    );
                }
                Instruction::StoreLocal(index) => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while storing local".to_string())
                    })?;

                    if *index >= locals.len() {
                        locals.resize(index + 1, Value::Unit);
                    }

                    locals[*index] = value;
                }
                Instruction::Negate => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while negating value".to_string())
                    })?;
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
                }
                Instruction::CastIntToFloat => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while casting value".to_string())
                    })?;
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
                }
                Instruction::Add => {
                    let result =
                        apply_numeric_op(&mut stack, |left, right| left + right, |left, right| {
                            left + right
                        })?;
                    stack.push(result);
                }
                Instruction::Subtract => {
                    let result =
                        apply_numeric_op(&mut stack, |left, right| left - right, |left, right| {
                            left - right
                        })?;
                    stack.push(result);
                }
                Instruction::Multiply => {
                    let result =
                        apply_numeric_op(&mut stack, |left, right| left * right, |left, right| {
                            left * right
                        })?;
                    stack.push(result);
                }
                Instruction::Divide => {
                    let result = apply_numeric_division(&mut stack)?;
                    stack.push(result);
                }
                Instruction::Modulo => {
                    let (left, right) = pop_int_pair(&mut stack)?;
                    if right == 0 {
                        return Err(RuntimeError("modulo by zero".to_string()));
                    }
                    stack.push(Value::Int(left % right));
                }
                Instruction::Less => {
                    let result = compare_ordered_pair(&mut stack, |left, right| left < right)?;
                    stack.push(result);
                }
                Instruction::Greater => {
                    let result = compare_ordered_pair(&mut stack, |left, right| left > right)?;
                    stack.push(result);
                }
                Instruction::Equal => {
                    let (left, right) = pop_pair(&mut stack)?;
                    stack.push(Value::Bool(left == right));
                }
                Instruction::NotEqual => {
                    let (left, right) = pop_pair(&mut stack)?;
                    stack.push(Value::Bool(left != right));
                }
                Instruction::LessEqual => {
                    let result = compare_ordered_pair(&mut stack, |left, right| left <= right)?;
                    stack.push(result);
                }
                Instruction::GreaterEqual => {
                    let result = compare_ordered_pair(&mut stack, |left, right| left >= right)?;
                    stack.push(result);
                }
                Instruction::BuildArray { element_count, .. } => {
                    let mut values = Vec::with_capacity(*element_count);
                    for _ in 0..*element_count {
                        values.push(stack.pop().ok_or_else(|| {
                            RuntimeError("stack underflow while building array".to_string())
                        })?);
                    }
                    values.reverse();
                    stack.push(Value::Array(values));
                }
                Instruction::BuildStruct {
                    type_id,
                    field_count,
                } => {
                    let mut values = Vec::with_capacity(*field_count);
                    for _ in 0..*field_count {
                        values.push(stack.pop().ok_or_else(|| {
                            RuntimeError("stack underflow while building struct".to_string())
                        })?);
                    }
                    values.reverse();

                    let crate::runtime::type_system::KiraType::Struct(struct_type) =
                        module.types.get(*type_id)
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
                }
                Instruction::ArrayLength => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while reading array length".to_string())
                    })?;
                    match value {
                        Value::Array(values) => stack.push(Value::Int(values.len() as i64)),
                        value => {
                            return Err(RuntimeError(format!(
                                "expected array for `.length`, got {:?}",
                                value
                            )))
                        }
                    }
                }
                Instruction::ArrayIndex => {
                    let index = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while reading array index".to_string())
                    })?;
                    let target = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while reading array target".to_string())
                    })?;
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
                }
                Instruction::StructField(index) => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while reading struct field".to_string())
                    })?;
                    match value {
                        Value::Struct(struct_value) => {
                            let (_, field_value) = struct_value.fields.get(*index).ok_or_else(|| {
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
                }
                Instruction::StoreLocalField { local, path } => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while storing struct field".to_string())
                    })?;
                    let target = locals.get_mut(*local).ok_or_else(|| {
                        RuntimeError(format!("invalid local index {local}"))
                    })?;
                    store_struct_field(target, path, value)?;
                }
                Instruction::ArrayAppendLocal(index) => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while appending to array".to_string())
                    })?;
                    let local = locals.get_mut(*index).ok_or_else(|| {
                        RuntimeError(format!("invalid local index {index}"))
                    })?;
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
                }
                Instruction::JumpIfFalse(target) => {
                    let value = stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while evaluating branch".to_string())
                    })?;
                    match value {
                        Value::Bool(false) => {
                            ip = *target;
                            continue;
                        }
                        Value::Bool(true) => {}
                        value => {
                            return Err(RuntimeError(format!(
                                "expected bool condition for branch, got {:?}",
                                value
                            )))
                        }
                    }
                }
                Instruction::Jump(target) => {
                    ip = *target;
                    continue;
                }
                Instruction::Call { function, arg_count } => {
                    let mut call_args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        call_args.push(stack.pop().ok_or_else(|| {
                            RuntimeError("stack underflow while preparing call".to_string())
                        })?);
                    }
                    call_args.reverse();
                    let result = self.run_function(module, function, call_args)?;
                    if result != Value::Unit {
                        stack.push(result);
                    }
                }
                Instruction::Pop => {
                    stack.pop().ok_or_else(|| {
                        RuntimeError("stack underflow while discarding expression result".to_string())
                    })?;
                }
                Instruction::Return => {
                    return Ok(stack.pop().unwrap_or(Value::Unit));
                }
            }

            ip += 1;
        }

        Ok(Value::Unit)
    }
}

fn pop_pair(stack: &mut Vec<Value>) -> Result<(Value, Value), RuntimeError> {
    let right = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading rhs".to_string()))?;
    let left = stack
        .pop()
        .ok_or_else(|| RuntimeError("stack underflow while reading lhs".to_string()))?;
    Ok((left, right))
}

fn apply_numeric_op(
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

fn apply_numeric_division(stack: &mut Vec<Value>) -> Result<Value, RuntimeError> {
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

fn compare_ordered_pair(
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

fn pop_int_pair(stack: &mut Vec<Value>) -> Result<(i64, i64), RuntimeError> {
    let (left, right) = pop_pair(stack)?;
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Ok((left, right)),
        (left, right) => Err(RuntimeError(format!(
            "expected integer operands, got {:?} and {:?}",
            left, right
        ))),
    }
}

fn store_struct_field(
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
