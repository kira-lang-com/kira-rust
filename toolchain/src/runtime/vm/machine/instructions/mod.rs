mod arithmetic;
mod collections;
mod control_flow;

use crate::compiler::{Chunk, CompiledModule, Instruction};
use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

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
        Instruction::Negate => arithmetic::execute_negate(stack),
        Instruction::CastIntToFloat => arithmetic::execute_cast_int_to_float(stack),
        Instruction::Add => arithmetic::execute_add(stack),
        Instruction::Subtract => arithmetic::execute_subtract(stack),
        Instruction::Multiply => arithmetic::execute_multiply(stack),
        Instruction::Divide => arithmetic::execute_divide(stack),
        Instruction::Modulo => arithmetic::execute_modulo(stack),
        Instruction::Less => arithmetic::execute_less(stack),
        Instruction::Greater => arithmetic::execute_greater(stack),
        Instruction::Equal => arithmetic::execute_equal(stack),
        Instruction::NotEqual => arithmetic::execute_not_equal(stack),
        Instruction::LessEqual => arithmetic::execute_less_equal(stack),
        Instruction::GreaterEqual => arithmetic::execute_greater_equal(stack),
        Instruction::BuildArray { element_count, .. } => {
            collections::execute_build_array(stack, *element_count)
        }
        Instruction::BuildStruct {
            type_id,
            field_count,
        } => collections::execute_build_struct(stack, module, *type_id, *field_count),
        Instruction::ArrayLength => collections::execute_array_length(stack),
        Instruction::ArrayIndex => collections::execute_array_index(stack),
        Instruction::StructField(index) => collections::execute_struct_field(stack, *index),
        Instruction::StoreLocalField { local, path } => {
            collections::execute_store_local_field(stack, locals, *local, path)
        }
        Instruction::ArrayAppendLocal(index) => {
            collections::execute_array_append_local(stack, locals, *index)
        }
        Instruction::JumpIfFalse(target) => control_flow::execute_jump_if_false(stack, *target),
        Instruction::Jump(target) => Ok(Some(*target)),
        Instruction::Call { function, arg_count } => {
            control_flow::execute_call(stack, module, vm, function, *arg_count)
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
