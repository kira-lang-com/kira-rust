use crate::compiler::{Chunk, CompiledModule, Instruction};
use crate::runtime::vm::RuntimeError;
use crate::runtime::Value;

use super::instructions::execute_instruction;
use super::vm::Vm;

pub fn execute_chunk(
    vm: &mut Vm,
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
        let jump_target = execute_instruction(
            instruction,
            &mut stack,
            &mut locals,
            chunk,
            module,
            vm,
        )?;

        if let Some(target) = jump_target {
            if target == usize::MAX {
                // Return instruction
                return Ok(stack.pop().unwrap_or(Value::Unit));
            }
            ip = target;
        } else {
            ip += 1;
        }
    }

    Ok(Value::Unit)
}
