use std::collections::VecDeque;

use crate::compiler::{CompiledModule, Chunk, Instruction};
use crate::runtime::type_system::{KiraType, TypeId};
use crate::runtime::Value;

use super::error::AotError;

pub struct StackLayout {
    pub states: Vec<StackState>,
    pub stack_slot_types: Vec<TypeId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackState {
    pub stack: Vec<TypeId>,
}

impl StackState {
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

pub fn infer_stack_layout(module: &CompiledModule, chunk: &Chunk) -> Result<StackLayout, AotError> {
    let mut states = vec![None::<StackState>; chunk.instructions.len()];
    if chunk.instructions.is_empty() {
        return Ok(StackLayout {
            states: Vec::new(),
            stack_slot_types: Vec::new(),
        });
    }
    states[0] = Some(StackState { stack: Vec::new() });
    let mut queue = VecDeque::from([0usize]);

    while let Some(index) = queue.pop_front() {
        let state = states[index]
            .clone()
            .ok_or_else(|| AotError(format!("missing inferred state for instruction {index}")))?;
        let (next_states, _) = transfer_state(module, chunk, index, &state)?;
        for (target, next_state) in next_states {
            if let Some(existing) = &states[target] {
                if existing != &next_state {
                    return Err(AotError(format!(
                        "inconsistent stack state at instruction {target}"
                    )));
                }
            } else {
                states[target] = Some(next_state);
                queue.push_back(target);
            }
        }
    }

    let states = states
        .into_iter()
        .enumerate()
        .map(|(index, state)| {
            state.ok_or_else(|| {
                AotError(format!("instruction {index} is unreachable in native code"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let max_depth = states.iter().map(StackState::depth).max().unwrap_or(0);
    let mut slot_types: Vec<Option<TypeId>> = vec![None; max_depth];
    for state in &states {
        for (index, type_id) in state.stack.iter().copied().enumerate() {
            if slot_types[index].is_none() {
                slot_types[index] = Some(type_id);
            }
        }
    }

    Ok(StackLayout {
        states,
        stack_slot_types: slot_types
            .into_iter()
            .map(|type_id| type_id.ok_or_else(|| AotError("missing stack slot type".to_string())))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn transfer_state(
    module: &CompiledModule,
    chunk: &Chunk,
    index: usize,
    state: &StackState,
) -> Result<(Vec<(usize, StackState)>, Option<TypeId>), AotError> {
    let instruction = &chunk.instructions[index];
    let mut stack = state.stack.clone();
    let next_index = index + 1;

    let successors = match instruction {
        Instruction::LoadConst(const_index) => {
            let value = chunk
                .constants
                .get(*const_index)
                .ok_or_else(|| AotError(format!("invalid constant index {const_index}")))?;
            stack.push(value_type(module, value)?);
            vec![(next_index, StackState { stack })]
        }
        Instruction::LoadLocal(local_index) => {
            let type_id = *chunk
                .local_types
                .get(*local_index)
                .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
            stack.push(type_id);
            vec![(next_index, StackState { stack })]
        }
        Instruction::StoreLocal(_) => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on store".to_string()))?;
            vec![(next_index, StackState { stack })]
        }
        Instruction::Negate => vec![(next_index, StackState { stack })],
        Instruction::CastIntToFloat => {
            *stack
                .last_mut()
                .ok_or_else(|| AotError("stack underflow on cast".to_string()))? =
                module.types.float();
            vec![(next_index, StackState { stack })]
        }
        Instruction::Add
        | Instruction::Subtract
        | Instruction::Multiply
        | Instruction::Divide
        | Instruction::Modulo => {
            let right = stack
                .pop()
                .ok_or_else(|| AotError("stack underflow".to_string()))?;
            let left = stack
                .pop()
                .ok_or_else(|| AotError("stack underflow".to_string()))?;
            if left != right {
                return Err(AotError("arithmetic stack types mismatch".to_string()));
            }
            stack.push(left);
            vec![(next_index, StackState { stack })]
        }
        Instruction::Less
        | Instruction::Greater
        | Instruction::Equal
        | Instruction::NotEqual
        | Instruction::LessEqual
        | Instruction::GreaterEqual => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow".to_string()))?;
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow".to_string()))?;
            stack.push(module.types.bool());
            vec![(next_index, StackState { stack })]
        }
        Instruction::BuildArray {
            type_id,
            element_count,
        } => {
            for _ in 0..*element_count {
                stack
                    .pop()
                    .ok_or_else(|| AotError("stack underflow while building array".to_string()))?;
            }
            stack.push(*type_id);
            vec![(next_index, StackState { stack })]
        }
        Instruction::ArrayLength => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on array length".to_string()))?;
            stack.push(module.types.int());
            vec![(next_index, StackState { stack })]
        }
        Instruction::ArrayIndex => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on array index".to_string()))?;
            let target = stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on array target".to_string()))?;
            let KiraType::Array(element) = module.types.get(target) else {
                return Err(AotError("array index expected array target".to_string()));
            };
            stack.push(*element);
            vec![(next_index, StackState { stack })]
        }
        Instruction::ArrayAppendLocal(_) => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on array append".to_string()))?;
            vec![(next_index, StackState { stack })]
        }
        Instruction::BuildStruct { type_id, field_count } => {
            for _ in 0..*field_count {
                stack
                    .pop()
                    .ok_or_else(|| AotError("stack underflow while building struct".to_string()))?;
            }
            stack.push(*type_id);
            vec![(next_index, StackState { stack })]
        }
        Instruction::StructField(field_index) => {
            let target = stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on struct field".to_string()))?;
            let field_type = module
                .types
                .struct_fields(target)
                .and_then(|fields| fields.get(*field_index))
                .map(|field| field.type_id)
                .ok_or_else(|| AotError(format!("invalid struct field index {}", field_index)))?;
            stack.push(field_type);
            vec![(next_index, StackState { stack })]
        }
        Instruction::StoreLocalField { .. } => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on struct field store".to_string()))?;
            vec![(next_index, StackState { stack })]
        }
        Instruction::JumpIfFalse(target) => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow".to_string()))?;
            vec![
                (
                    next_index,
                    StackState {
                        stack: stack.clone(),
                    },
                ),
                (*target, StackState { stack }),
            ]
        }
        Instruction::Jump(target) => vec![(*target, StackState { stack })],
        Instruction::Call {
            function,
            arg_count,
        } => {
            let signature = module
                .functions
                .get(function)
                .map(|function| function.signature.clone())
                .or_else(|| {
                    module
                        .builtins
                        .get(function)
                        .map(|builtin| builtin.signature.clone())
                })
                .ok_or_else(|| AotError(format!("missing signature for `{function}`")))?;
            for _ in 0..*arg_count {
                stack
                    .pop()
                    .ok_or_else(|| AotError("stack underflow on call".to_string()))?;
            }
            if module.types.get(signature.return_type) != &KiraType::Unit {
                stack.push(signature.return_type);
            }
            vec![(next_index, StackState { stack })]
        }
        Instruction::Pop => {
            stack
                .pop()
                .ok_or_else(|| AotError("stack underflow on pop".to_string()))?;
            vec![(next_index, StackState { stack })]
        }
        Instruction::Return => Vec::new(),
    };

    Ok((successors, None))
}

fn value_type(module: &CompiledModule, value: &Value) -> Result<TypeId, AotError> {
    Ok(match value {
        Value::Unit => module.types.unit(),
        Value::Bool(_) => module.types.bool(),
        Value::Int(_) => module.types.int(),
        Value::Float(_) => module.types.float(),
        Value::String(_) => module
            .types
            .resolve_named("string")
            .ok_or_else(|| AotError("missing string type".to_string()))?,
        Value::Array(_) => {
            return Err(AotError(
                "array constants are not supported in LLVM AOT".to_string(),
            ))
        }
        Value::Struct(_) => {
            return Err(AotError(
                "struct constants are not supported in LLVM AOT".to_string(),
            ))
        }
    })
}
