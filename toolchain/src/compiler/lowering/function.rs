use std::collections::HashMap;

use crate::ast::FunctionDefinition;
use crate::compiler::{Chunk, CompileError, FunctionSignature, Instruction};
use crate::runtime::type_system::TypeSystem;
use crate::runtime::Value;

use super::statements::lower_block;
use super::types::LocalBinding;

pub fn lower_function_body(
    function: &FunctionDefinition,
    types: &mut TypeSystem,
    signatures: &HashMap<String, FunctionSignature>,
) -> Result<Chunk, CompileError> {
    let signature = signatures
        .get(&function.name.name)
        .ok_or_else(|| CompileError(format!("missing signature for `{}`", function.name.name)))?;

    let mut chunk = Chunk {
        instructions: Vec::new(),
        constants: Vec::new(),
        local_count: function.params.len(),
        local_types: signature.params.clone(),
    };
    let mut locals = HashMap::new();

    for (slot, parameter) in function.params.iter().enumerate() {
        locals.insert(
            parameter.name.name.clone(),
            LocalBinding {
                slot,
                type_id: signature.params[slot],
            },
        );
    }

    lower_block(
        &function.body.statements,
        &mut chunk,
        &mut locals,
        signature.return_type,
        types,
        signatures,
        &function.name.name,
        &mut Vec::new(),
    )?;

    if !matches!(chunk.instructions.last(), Some(Instruction::Return)) {
        if signature.return_type != types.unit() {
            let unit = chunk.push_constant(Value::Unit);
            chunk.instructions.push(Instruction::LoadConst(unit));
        }
        chunk.instructions.push(Instruction::Return);
    }

    Ok(chunk)
}
