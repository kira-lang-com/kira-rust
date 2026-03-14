// Note: This file is still large (~1400 lines) and represents the LLVM codegen implementation
// It's been extracted from the main mod.rs but could be further split into:
// - codegen/emit.rs (instruction emission)
// - codegen/helpers.rs (helper functions, type conversions)
// - codegen/values.rs (value boxing/unboxing, stack operations)
// For now, keeping it as one file to maintain the refactoring momentum

use std::collections::HashMap;
use std::path::Path;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};

use crate::compiler::{
    BackendKind, Chunk, CompiledFunction, CompiledModule, FunctionSignature, Instruction,
};
use crate::runtime::type_system::{KiraType, TypeId};
use crate::runtime::Value;

use super::super::error::AotError;
use super::super::stack::{infer_stack_layout, StackState};
use super::super::utils::mangle_ident;

pub struct NativeCodegen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    context: &'ctx Context,
    target_machine: TargetMachine,
    compiled: &'ctx CompiledModule,
    function_values: HashMap<String, FunctionValue<'ctx>>,
    bridge_values: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> NativeCodegen<'ctx> {
    pub fn new(
        project_name: &str,
        compiled: &'ctx CompiledModule,
        context: &'ctx Context,
    ) -> Result<Self, AotError> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|error| AotError(format!("failed to initialize native target: {error}")))?;
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|error| AotError(format!("failed to resolve LLVM target: {error}")))?;
        let target_machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| AotError("failed to create target machine".to_string()))?;

        let module = context.create_module(project_name);
        module.set_triple(&triple);
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        let builder = context.create_builder();

        let mut codegen = Self {
            module,
            builder,
            context,
            target_machine,
            compiled,
            function_values: HashMap::new(),
            bridge_values: HashMap::new(),
        };
        codegen.declare_native_functions()?;
        codegen.emit_native_functions()?;
        Ok(codegen)
    }

    pub fn write_object(&self, object_path: &Path) -> Result<(), AotError> {
        let ir_path = object_path.with_extension("ll");
        self.module
            .print_to_file(&ir_path)
            .map_err(|error| AotError(format!("failed to emit LLVM IR: {error}")))?;

        self.target_machine
            .write_to_file(&self.module, FileType::Object, object_path)
            .map_err(|error| AotError(format!("failed to emit object file: {error}")))
    }

    fn declare_native_functions(&mut self) -> Result<(), AotError> {
        for function in self.compiled.functions.values() {
            if function.selected_backend != BackendKind::Native {
                continue;
            }
            let fn_type = self.llvm_function_type(&function.signature)?;
            let symbol = function
                .artifacts
                .aot
                .as_ref()
                .ok_or_else(|| AotError(format!("missing AOT artifact for `{}`", function.name)))?
                .symbol
                .clone();
            let value = self.module.add_function(&symbol, fn_type, None);
            self.function_values.insert(function.name.clone(), value);
        }
        Ok(())
    }

    fn emit_native_functions(&mut self) -> Result<(), AotError> {
        for function in self.compiled.functions.values() {
            if function.selected_backend != BackendKind::Native {
                continue;
            }
            let chunk = function.artifacts.bytecode.as_ref().ok_or_else(|| {
                AotError(format!("missing bytecode shadow for `{}`", function.name))
            })?;
            self.emit_function(function, chunk)?;
        }
        Ok(())
    }

    fn emit_function(
        &mut self,
        function: &CompiledFunction,
        chunk: &Chunk,
    ) -> Result<(), AotError> {
        self.ensure_supported_signature(&function.signature, &function.name)?;
        self.ensure_supported_chunk(function, chunk)?;

        let function_value = *self
            .function_values
            .get(&function.name)
            .ok_or_else(|| AotError(format!("missing LLVM declaration for `{}`", function.name)))?;

        let entry = self.context.append_basic_block(function_value, "entry");
        let blocks = (0..chunk.instructions.len())
            .map(|index| {
                self.context
                    .append_basic_block(function_value, &format!("bb{index}"))
            })
            .collect::<Vec<_>>();

        let ctx_arg = function_value
            .get_first_param()
            .ok_or_else(|| {
                AotError(format!(
                    "missing runtime context parameter for `{}`",
                    function.name
                ))
            })?
            .into_pointer_value();

        self.builder.position_at_end(entry);

        let locals =
            self.build_local_allocas_in_place(function_value, &function.signature, chunk)?;
        let stack_layout = infer_stack_layout(self.compiled, chunk).map_err(|error| {
            AotError(format!(
                "failed to infer native stack layout for `{}`: {}",
                function.name, error
            ))
        })?;
        let stack_slots =
            self.build_stack_allocas_in_place(function_value, &stack_layout.stack_slot_types)?;

        for (index, local) in locals
            .iter()
            .enumerate()
            .take(function.signature.params.len())
        {
            let param = function_value
                .get_nth_param((index + 1) as u32)
                .ok_or_else(|| {
                    AotError(format!("missing parameter {index} for `{}`", function.name))
                })?;
            self.builder
                .build_store(*local, param)
                .map_err(|error| AotError(error.to_string()))?;
        }

        self.builder
            .build_unconditional_branch(blocks[0])
            .map_err(|error| AotError(error.to_string()))?;

        for (index, block) in blocks.iter().enumerate() {
            self.builder.position_at_end(*block);
            let state = &stack_layout.states[index];
            self.emit_instruction(
                function_value,
                ctx_arg,
                &function.signature,
                chunk,
                index,
                state,
                &locals,
                &stack_slots,
                &stack_layout.stack_slot_types,
                &blocks,
            )?;
        }

        Ok(())
    }

    fn build_local_allocas_in_place(
        &mut self,
        _function: FunctionValue<'ctx>,
        _signature: &FunctionSignature,
        chunk: &Chunk,
    ) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut locals = Vec::with_capacity(chunk.local_count);
        for (index, type_id) in chunk.local_types.iter().enumerate().take(chunk.local_count) {
            let llvm_type = self.llvm_basic_type(*type_id).ok_or_else(|| {
                AotError(format!(
                    "AOT backend does not yet support local type {:?} in slot {}",
                    self.compiled.types.get(*type_id),
                    index
                ))
            })?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, &format!("local_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            locals.push(alloca);
        }
        Ok(locals)
    }

    fn build_stack_allocas_in_place(
        &mut self,
        _function: FunctionValue<'ctx>,
        stack_slot_types: &[TypeId],
    ) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut slots = Vec::with_capacity(stack_slot_types.len());
        for (index, type_id) in stack_slot_types.iter().copied().enumerate() {
            let llvm_type = self.llvm_basic_type(type_id).ok_or_else(|| {
                AotError(format!(
                    "AOT backend does not yet support stack type {:?} in slot {}",
                    self.compiled.types.get(type_id),
                    index
                ))
            })?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, &format!("stack_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            slots.push(alloca);
        }
        Ok(slots)
    }

    // Note: emit_instruction, emit_binary_instruction, emit_call and other methods
    // are implemented in the original file. Due to space constraints, this is a
    // placeholder showing the structure. The full implementation would be copied here.

    fn emit_instruction(
        &mut self,
        function_value: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        signature: &FunctionSignature,
        chunk: &Chunk,
        index: usize,
        state: &StackState,
        locals: &[PointerValue<'ctx>],
        stack_slots: &[PointerValue<'ctx>],
        stack_slot_types: &[TypeId],
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        let instruction = chunk
            .instructions
            .get(index)
            .ok_or_else(|| AotError(format!("invalid instruction index {index}")))?;
        let depth = state.depth();

        match instruction {
            Instruction::LoadConst(const_index) => {
                let value = chunk
                    .constants
                    .get(*const_index)
                    .ok_or_else(|| AotError(format!("invalid constant index {const_index}")))?;
                let (type_id, llvm_value) = self.llvm_const(value)?;
                self.store_stack(stack_slots, depth, type_id, llvm_value)?;
            }
            Instruction::LoadLocal(local_index) => {
                let type_id = *chunk
                    .local_types
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
                let local = *locals
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("missing local slot {local_index}")))?;
                let mut value =
                    self.load_typed_ptr(local, type_id, &format!("local_{local_index}_value"))?;
                if self.is_value_handle_type(type_id) {
                    value = self.clone_value_handle(value)?;
                }
                self.store_stack(stack_slots, depth, type_id, value)?;
            }
            Instruction::StoreLocal(local_index) => {
                let type_id = *chunk
                    .local_types
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
                let value = self.load_stack(stack_slots, depth - 1, type_id, "store_value")?;
                let local = *locals
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("missing local slot {local_index}")))?;
                self.store_ptr(local, value)?;
            }
            Instruction::Negate => {
                let type_id = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on negate".to_string()))?;
                let value = self.load_stack(stack_slots, depth - 1, type_id, "neg_arg")?;
                let result = match self.compiled.types.get(type_id) {
                    KiraType::Int => {
                        let v = value.into_int_value();
                        self.builder
                            .build_int_neg(v, "neg")
                            .map_err(|e| AotError(e.to_string()))?
                            .into()
                    }
                    KiraType::Float => {
                        let v = value.into_float_value();
                        self.builder
                            .build_float_neg(v, "fneg")
                            .map_err(|e| AotError(e.to_string()))?
                            .into()
                    }
                    other => {
                        return Err(AotError(format!(
                            "negation is not supported for type {:?}",
                            other
                        )));
                    }
                };
                self.store_stack(stack_slots, depth - 1, type_id, result)?;
            }
            Instruction::CastIntToFloat => {
                let src = self.load_stack(stack_slots, depth - 1, self.compiled.types.int(), "int")?;
                let float = self
                    .builder
                    .build_signed_int_to_float(
                        src.into_int_value(),
                        self.context.f64_type(),
                        "i2f",
                    )
                    .map_err(|e| AotError(e.to_string()))?;
                self.store_stack(stack_slots, depth - 1, self.compiled.types.float(), float.into())?;
            }
            Instruction::Add
            | Instruction::Subtract
            | Instruction::Multiply
            | Instruction::Divide
            | Instruction::Modulo => {
                let right_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow".to_string()))?;
                let left_type = *state
                    .stack
                    .get(depth - 2)
                    .ok_or_else(|| AotError("stack underflow".to_string()))?;
                if left_type != right_type {
                    return Err(AotError("binary operand type mismatch".to_string()));
                }
                let left = self.load_stack(stack_slots, depth - 2, left_type, "lhs")?;
                let right = self.load_stack(stack_slots, depth - 1, right_type, "rhs")?;

                let result = match (instruction, self.compiled.types.get(left_type)) {
                    (Instruction::Add, KiraType::Int) => self
                        .builder
                        .build_int_add(left.into_int_value(), right.into_int_value(), "add")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Subtract, KiraType::Int) => self
                        .builder
                        .build_int_sub(left.into_int_value(), right.into_int_value(), "sub")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Multiply, KiraType::Int) => self
                        .builder
                        .build_int_mul(left.into_int_value(), right.into_int_value(), "mul")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Divide, KiraType::Int) => self
                        .builder
                        .build_int_signed_div(left.into_int_value(), right.into_int_value(), "div")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Modulo, KiraType::Int) => self
                        .builder
                        .build_int_signed_rem(left.into_int_value(), right.into_int_value(), "rem")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Add, KiraType::Float) => self
                        .builder
                        .build_float_add(left.into_float_value(), right.into_float_value(), "fadd")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Subtract, KiraType::Float) => self
                        .builder
                        .build_float_sub(left.into_float_value(), right.into_float_value(), "fsub")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Multiply, KiraType::Float) => self
                        .builder
                        .build_float_mul(left.into_float_value(), right.into_float_value(), "fmul")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Divide, KiraType::Float) => self
                        .builder
                        .build_float_div(left.into_float_value(), right.into_float_value(), "fdiv")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    (Instruction::Modulo, KiraType::Float) => {
                        return Err(AotError("modulo is not supported for float".to_string()));
                    }
                    (_, other) => {
                        return Err(AotError(format!(
                            "arithmetic is not supported for type {:?}",
                            other
                        )));
                    }
                };

                self.store_stack(stack_slots, depth - 2, left_type, result)?;
            }
            Instruction::Less
            | Instruction::Greater
            | Instruction::Equal
            | Instruction::NotEqual
            | Instruction::LessEqual
            | Instruction::GreaterEqual => {
                let right_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow".to_string()))?;
                let left_type = *state
                    .stack
                    .get(depth - 2)
                    .ok_or_else(|| AotError("stack underflow".to_string()))?;
                let left = self.load_stack(stack_slots, depth - 2, left_type, "cmp_lhs")?;
                let right = self.load_stack(stack_slots, depth - 1, right_type, "cmp_rhs")?;

                let result = self.emit_comparison(instruction, left_type, left, right)?;
                self.store_stack(
                    stack_slots,
                    depth - 2,
                    self.compiled.types.bool(),
                    result.into(),
                )?;
            }
            Instruction::BuildArray {
                type_id,
                element_count,
            } => {
                let KiraType::Array(element_type) = self.compiled.types.get(*type_id) else {
                    return Err(AotError("BuildArray target type is not an array".to_string()));
                };

                let array_handle = self.call_runtime_new_array()?;

                let mut elements = Vec::with_capacity(*element_count);
                for offset in 0..*element_count {
                    let slot = depth - 1 - offset;
                    let value = self.load_stack(stack_slots, slot, *element_type, "arr_elem")?;
                    let boxed = self.box_value_as_handle(*element_type, value)?;
                    elements.push(boxed);
                }
                elements.reverse();
                for boxed in elements {
                    self.call_runtime_array_push(array_handle, boxed)?;
                }

                self.store_stack(stack_slots, depth - *element_count, *type_id, array_handle.into())?;
            }
            Instruction::BuildStruct { type_id, field_count } => {
                let KiraType::Struct(struct_type) = self.compiled.types.get(*type_id) else {
                    return Err(AotError("BuildStruct target type is not a struct".to_string()));
                };
                if struct_type.fields.len() != *field_count {
                    return Err(AotError(format!(
                        "struct `{}` expects {} fields but bytecode provided {}",
                        struct_type.name,
                        struct_type.fields.len(),
                        field_count
                    )));
                }

                let struct_handle = self.call_runtime_new_struct(ctx_arg, *type_id)?;

                let mut values = Vec::with_capacity(*field_count);
                for offset in 0..*field_count {
                    let field_index = *field_count - 1 - offset;
                    let slot = depth - 1 - offset;
                    let field_type = struct_type
                        .fields
                        .get(field_index)
                        .ok_or_else(|| AotError("invalid struct field index".to_string()))?
                        .type_id;
                    let value = self.load_stack(stack_slots, slot, field_type, "field")?;
                    let boxed = self.box_value_as_handle(field_type, value)?;
                    values.push((field_index, boxed));
                }
                values.reverse();

                for (field_index, boxed) in values {
                    self.call_runtime_struct_set_field(struct_handle, field_index, boxed)?;
                }

                self.store_stack(stack_slots, depth - *field_count, *type_id, struct_handle.into())?;
            }
            Instruction::ArrayLength => {
                let target_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on array length".to_string()))?;
                let array_handle = self.load_stack(stack_slots, depth - 1, target_type, "array")?;
                let len = self.call_runtime_array_length(array_handle.into_pointer_value())?;
                self.store_stack(stack_slots, depth - 1, self.compiled.types.int(), len.into())?;
            }
            Instruction::ArrayIndex => {
                let index_value = self.load_stack(stack_slots, depth - 1, self.compiled.types.int(), "idx")?;
                let array_type = *state
                    .stack
                    .get(depth - 2)
                    .ok_or_else(|| AotError("stack underflow on array index".to_string()))?;
                let array_handle = self.load_stack(stack_slots, depth - 2, array_type, "array")?;
                let KiraType::Array(element_type) = self.compiled.types.get(array_type) else {
                    return Err(AotError("array index expected array target".to_string()));
                };
                let handle = self.call_runtime_array_index(
                    array_handle.into_pointer_value(),
                    index_value.into_int_value(),
                )?;
                let element = self.unbox_handle_if_needed(*element_type, handle)?;
                self.store_stack(stack_slots, depth - 2, *element_type, element)?;
            }
            Instruction::StructField(field_index) => {
                let target_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on struct field".to_string()))?;
                let struct_handle = self.load_stack(stack_slots, depth - 1, target_type, "struct")?;
                let field_type = self
                    .compiled
                    .types
                    .struct_fields(target_type)
                    .and_then(|fields| fields.get(*field_index))
                    .map(|field| field.type_id)
                    .ok_or_else(|| AotError(format!("invalid struct field index {}", field_index)))?;
                let handle =
                    self.call_runtime_struct_field(struct_handle.into_pointer_value(), *field_index)?;
                let field_value = self.unbox_handle_if_needed(field_type, handle)?;
                self.store_stack(stack_slots, depth - 1, field_type, field_value)?;
            }
            Instruction::StoreLocalField { local, path } => {
                let value_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on field store".to_string()))?;
                let value = self.load_stack(stack_slots, depth - 1, value_type, "field_value")?;
                let boxed = self.box_value_as_handle(value_type, value)?;
                let target_type = *chunk
                    .local_types
                    .get(*local)
                    .ok_or_else(|| AotError(format!("invalid local index {local}")))?;
                let target_local = *locals
                    .get(*local)
                    .ok_or_else(|| AotError(format!("missing local slot {local}")))?;
                let target_handle =
                    self.load_typed_ptr(target_local, target_type, &format!("local_{local}_target"))?
                        .into_pointer_value();
                let (path_ptr, path_len) = self.const_usize_path(path, &format!("path_{index}"))?;
                self.call_runtime_store_struct_field(target_handle, path_ptr, path_len, boxed)?;
            }
            Instruction::ArrayAppendLocal(local_index) => {
                let value_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on array append".to_string()))?;
                let value = self.load_stack(stack_slots, depth - 1, value_type, "append_value")?;

                let array_type = *chunk
                    .local_types
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
                let KiraType::Array(element_type) = self.compiled.types.get(array_type) else {
                    return Err(AotError("array append expected array local".to_string()));
                };
                if *element_type != value_type {
                    // The compiler should ensure exact match for append.
                    return Err(AotError("array append type mismatch".to_string()));
                }

                let boxed = self.box_value_as_handle(value_type, value)?;
                let local = *locals
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("missing local slot {local_index}")))?;
                let array_handle = self
                    .load_typed_ptr(local, array_type, "array_local")?
                    .into_pointer_value();
                self.call_runtime_array_append(array_handle, boxed)?;
            }
            Instruction::JumpIfFalse(target) => {
                if index + 1 >= blocks.len() {
                    return Err(AotError("JumpIfFalse at end of function".to_string()));
                }
                let cond = self.load_stack(
                    stack_slots,
                    depth - 1,
                    self.compiled.types.bool(),
                    "cond",
                )?;
                self.builder
                    .build_conditional_branch(
                        cond.into_int_value(),
                        blocks[*target],
                        blocks[index + 1],
                    )
                    .map_err(|e| AotError(e.to_string()))?;
                return Ok(());
            }
            Instruction::Jump(target) => {
                self.builder
                    .build_unconditional_branch(blocks[*target])
                    .map_err(|e| AotError(e.to_string()))?;
                return Ok(());
            }
            Instruction::Call { function, arg_count } => {
                // `printIn` is typed as `dynamic` but is allowed to accept any argument type.
                // For native codegen we must not reinterpret stack slot bits as a pointer, so
                // we route it through the native support printing helpers using the real type.
                if function.as_str() == "printIn" {
                    if *arg_count != 1 {
                        return Err(AotError(format!(
                            "`printIn` expects 1 argument but got {}",
                            arg_count
                        )));
                    }
                    let base = depth
                        .checked_sub(*arg_count)
                        .ok_or_else(|| AotError("stack underflow on call".to_string()))?;
                    let arg_type = *stack_slot_types
                        .get(base)
                        .ok_or_else(|| AotError("missing stack slot type for printIn".to_string()))?;
                    let arg = self.load_stack(stack_slots, base, arg_type, "print_arg")?;
                    self.emit_print_in(ctx_arg, arg_type, arg)?;
                } else {
                    let signature = self
                        .compiled
                        .functions
                        .get(function)
                        .map(|function| function.signature.clone())
                        .or_else(|| {
                            self.compiled
                                .ffi
                                .functions
                                .get(function)
                                .map(|function| function.signature.clone())
                        })
                        .or_else(|| {
                            self.compiled
                                .builtins
                                .get(function)
                                .map(|builtin| builtin.signature.clone())
                        })
                        .ok_or_else(|| AotError(format!("missing signature for `{function}`")))?;

                    let base = depth
                        .checked_sub(*arg_count)
                        .ok_or_else(|| AotError("stack underflow on call".to_string()))?;

                    let args = (0..*arg_count)
                        .map(|offset| {
                            let type_id = signature.params[offset];
                            self.load_stack(stack_slots, base + offset, type_id, "call_arg")
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let result =
                        self.emit_call(function_value, ctx_arg, function, &signature, &args)?;
                    if self.compiled.types.get(signature.return_type) != &KiraType::Unit {
                        let result_slot = base;
                        self.store_stack(stack_slots, result_slot, signature.return_type, result)?;
                    }
                }
            }
            Instruction::Pop => {
                // Stack slot values are left as-is; the stack layout determines liveness.
            }
            Instruction::Return => {
                match self.compiled.types.get(signature.return_type) {
                    KiraType::Unit => {
                        self.builder
                            .build_return(None)
                            .map_err(|e| AotError(e.to_string()))?;
                    }
                    _ => {
                        let type_id = signature.return_type;
                        let value =
                            self.load_stack(stack_slots, depth - 1, type_id, "ret_value")?;
                        self.builder
                            .build_return(Some(&value))
                            .map_err(|e| AotError(e.to_string()))?;
                    }
                }
                return Ok(());
            }
        }

        // Default fallthrough.
        if index + 1 < blocks.len() {
            let block = self
                .builder
                .get_insert_block()
                .ok_or_else(|| AotError("missing insert block".to_string()))?;
            if block.get_terminator().is_none() {
                self.builder
                    .build_unconditional_branch(blocks[index + 1])
                    .map_err(|e| AotError(e.to_string()))?;
            }
        } else {
            // A well-formed chunk should end with `Return`.
            let block = self
                .builder
                .get_insert_block()
                .ok_or_else(|| AotError("missing insert block".to_string()))?;
            if block.get_terminator().is_none() {
                return Err(AotError(format!(
                    "`{}` has no terminator at end of function",
                    function_value.get_name().to_string_lossy()
                )));
            }
        }

        Ok(())
    }

    fn emit_print_in(
        &mut self,
        ctx_arg: PointerValue<'ctx>,
        type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), AotError> {
        match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let f = self.declare_runtime_print_int();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_int")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::Bool => {
                let f = self.declare_runtime_print_bool();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_bool")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::Float => {
                let f = self.declare_runtime_print_float();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_float")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::Opaque(_) => {
                // Opaque handles are represented as pointers; print their address.
                let f = self.declare_runtime_print_int();
                let ptr = value.into_pointer_value();
                let as_int = self.ptr_to_int(ptr)?;
                self.builder
                    .build_call(f, &[ctx_arg.into(), as_int.into()], "print_opaque")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                let f = self.declare_runtime_print_value();
                self.builder
                    .build_call(f, &[ctx_arg.into(), value.into()], "print_value")
                    .map_err(|e| AotError(e.to_string()))?;
            }
            other => {
                return Err(AotError(format!(
                    "`printIn` is not supported for type {:?} in native codegen",
                    other
                )));
            }
        }
        Ok(())
    }

    fn emit_comparison(
        &mut self,
        instruction: &Instruction,
        type_id: TypeId,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        match self.compiled.types.get(type_id) {
            KiraType::Int | KiraType::Bool => {
                let pred = match instruction {
                    Instruction::Less => IntPredicate::SLT,
                    Instruction::Greater => IntPredicate::SGT,
                    Instruction::Equal => IntPredicate::EQ,
                    Instruction::NotEqual => IntPredicate::NE,
                    Instruction::LessEqual => IntPredicate::SLE,
                    Instruction::GreaterEqual => IntPredicate::SGE,
                    _ => return Err(AotError("invalid comparison opcode".to_string())),
                };
                self.builder
                    .build_int_compare(pred, left.into_int_value(), right.into_int_value(), "icmp")
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::Float => {
                let pred = match instruction {
                    Instruction::Less => FloatPredicate::OLT,
                    Instruction::Greater => FloatPredicate::OGT,
                    Instruction::Equal => FloatPredicate::OEQ,
                    Instruction::NotEqual => FloatPredicate::ONE,
                    Instruction::LessEqual => FloatPredicate::OLE,
                    Instruction::GreaterEqual => FloatPredicate::OGE,
                    _ => return Err(AotError("invalid comparison opcode".to_string())),
                };
                self.builder
                    .build_float_compare(
                        pred,
                        left.into_float_value(),
                        right.into_float_value(),
                        "fcmp",
                    )
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::Opaque(_) => {
                let pred = match instruction {
                    Instruction::Equal => IntPredicate::EQ,
                    Instruction::NotEqual => IntPredicate::NE,
                    _ => {
                        return Err(AotError(
                            "only == and != are supported for opaque handles".to_string(),
                        ))
                    }
                };
                self.builder
                    .build_int_compare(
                        pred,
                        self.ptr_to_int(left.into_pointer_value())?,
                        self.ptr_to_int(right.into_pointer_value())?,
                        "pcmp",
                    )
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                if !matches!(instruction, Instruction::Equal | Instruction::NotEqual) {
                    return Err(AotError(
                        "only == and != are supported for value handles".to_string(),
                    ));
                }
                let eq = self.call_runtime_value_eq(
                    left.into_pointer_value(),
                    right.into_pointer_value(),
                )?;
                if matches!(instruction, Instruction::Equal) {
                    Ok(eq)
                } else {
                    self.builder
                        .build_not(eq, "not")
                        .map_err(|e| AotError(e.to_string()))
                }
            }
            other => Err(AotError(format!(
                "comparison is not supported for type {:?}",
                other
            ))),
        }
    }

    fn emit_call(
        &mut self,
        _function_value: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        callee: &str,
        signature: &FunctionSignature,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        if let Some(ffi) = self.compiled.ffi.functions.get(callee) {
            let fn_value = self.declare_ffi_function(callee, &ffi.signature)?;
            let call_site = self
                .builder
                .build_call(
                    fn_value,
                    &args.iter().copied().map(Into::into).collect::<Vec<_>>(),
                    "ffi_call",
                )
                .map_err(|e| AotError(e.to_string()))?;
            return if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
                Ok(self.context.i64_type().const_zero().into())
            } else {
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing ffi call result".to_string()))
            };
        }

        if let Some(function) = self.compiled.functions.get(callee) {
            if function.selected_backend == BackendKind::Native {
                let fn_value = *self.function_values.get(callee).ok_or_else(|| {
                    AotError(format!("missing native callee `{}`", callee))
                })?;
                let mut full_args = Vec::with_capacity(args.len() + 1);
                full_args.push(ctx_arg.into());
                for value in args.iter().copied() {
                    full_args.push(value.into());
                }
                let call_site = self
                    .builder
                    .build_call(fn_value, &full_args, "call_native")
                    .map_err(|e| AotError(e.to_string()))?;
                return if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
                    Ok(self.context.i64_type().const_zero().into())
                } else {
                    call_site
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| AotError("missing call result".to_string()))
                };
            }
        }

        let bridge_symbol = format!("kira_bridge_{}", mangle_ident(callee));
        let bridge_fn = self.declare_bridge_function(&bridge_symbol, signature)?;
        let mut full_args = Vec::with_capacity(args.len() + 1);
        full_args.push(ctx_arg.into());
        for value in args.iter().copied() {
            full_args.push(value.into());
        }
        let call_site = self
            .builder
            .build_call(bridge_fn, &full_args, "call_bridge")
            .map_err(|e| AotError(e.to_string()))?;
        if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
            Ok(self.context.i64_type().const_zero().into())
        } else {
            call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| AotError("missing bridge call result".to_string()))
        }
    }

    fn declare_bridge_function(
        &mut self,
        symbol: &str,
        signature: &FunctionSignature,
    ) -> Result<FunctionValue<'ctx>, AotError> {
        if let Some(existing) = self.bridge_values.get(symbol).copied() {
            return Ok(existing);
        }
        let fn_type = self.llvm_function_type(signature)?;
        let value = self.module.add_function(symbol, fn_type, None);
        self.bridge_values.insert(symbol.to_string(), value);
        Ok(value)
    }

    fn declare_ffi_function(
        &mut self,
        symbol: &str,
        signature: &FunctionSignature,
    ) -> Result<FunctionValue<'ctx>, AotError> {
        if let Some(existing) = self.module.get_function(symbol) {
            return Ok(existing);
        }
        let mut params = Vec::with_capacity(signature.params.len());
        for type_id in &signature.params {
            params.push(
                self.llvm_basic_type(*type_id)
                    .ok_or_else(|| AotError(format!("unsupported FFI parameter type")))?,
            );
        }
        let fn_type = match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => self.context.void_type().fn_type(
                &params.iter().copied().map(Into::into).collect::<Vec<_>>(),
                false,
            ),
            _ => self
                .llvm_basic_type(signature.return_type)
                .ok_or_else(|| AotError("unsupported FFI return type".to_string()))?
                .fn_type(&params.iter().copied().map(Into::into).collect::<Vec<_>>(), false),
        };
        Ok(self.module.add_function(symbol, fn_type, None))
    }

    fn is_value_handle_type(&self, type_id: TypeId) -> bool {
        matches!(
            self.compiled.types.get(type_id),
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_)
        )
    }

    fn llvm_const(&mut self, value: &Value) -> Result<(TypeId, BasicValueEnum<'ctx>), AotError> {
        Ok(match value {
            Value::Bool(b) => (self.compiled.types.bool(), self.context.bool_type().const_int(*b as u64, false).into()),
            Value::Int(i) => (self.compiled.types.int(), self.context.i64_type().const_int(*i as u64, true).into()),
            Value::Float(f) => (self.compiled.types.float(), self.context.f64_type().const_float(f.0).into()),
            Value::String(s) => {
                let handle = self.const_string_handle(s)?;
                (
                    self.compiled
                        .types
                        .resolve_named("string")
                        .ok_or_else(|| AotError("missing string type".to_string()))?,
                    handle.into(),
                )
            }
            Value::Unit => {
                return Err(AotError(
                    "unit constants are not supported as stack values in AOT".to_string(),
                ))
            }
            Value::Array(_) | Value::Struct(_) => {
                return Err(AotError("aggregate constants are not supported in AOT".to_string()))
            }
        })
    }

    fn const_string_handle(&mut self, value: &str) -> Result<PointerValue<'ctx>, AotError> {
        let global = self
            .builder
            .build_global_string_ptr(value, "kira_str")
            .map_err(|e| AotError(e.to_string()))?;
        let bytes_ptr = global.as_pointer_value();
        let len = self
            .ptr_sized_int_type()
            .const_int(value.as_bytes().len() as u64, false);
        let make = self.declare_runtime_make_string();
        let call_site = self
            .builder
            .build_call(make, &[bytes_ptr.into(), len.into()], "make_string")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing string handle result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    fn ptr_sized_int_type(&self) -> inkwell::types::IntType<'ctx> {
        self.target_machine
            .get_target_data()
            .ptr_sized_int_type_in_context(self.context, None)
    }

    fn ptr_to_int(&self, ptr: PointerValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        self.builder
            .build_ptr_to_int(ptr, self.ptr_sized_int_type(), "ptrtoint")
            .map_err(|e| AotError(e.to_string()))
    }

    fn load_typed_ptr(
        &self,
        ptr: PointerValue<'ctx>,
        type_id: TypeId,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ty = self.llvm_basic_type(type_id).ok_or_else(|| {
            AotError(format!(
                "LLVM AOT backend does not yet support type {:?} for load",
                self.compiled.types.get(type_id)
            ))
        })?;
        self.builder
            .build_load(ty, ptr, name)
            .map_err(|e| AotError(e.to_string()))
    }

    fn store_ptr(&self, ptr: PointerValue<'ctx>, value: BasicValueEnum<'ctx>) -> Result<(), AotError> {
        self.builder
            .build_store(ptr, value)
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    fn load_stack(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        type_id: TypeId,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        let value = self.load_typed_ptr(ptr, type_id, name)?;
        // For bool slots, ensure we use i1.
        match self.compiled.types.get(type_id) {
            KiraType::Bool => Ok(value.into_int_value().into()),
            _ => Ok(value),
        }
    }

    fn store_stack(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        _type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        self.store_ptr(ptr, value)
    }

    fn clone_value_handle(&mut self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, AotError> {
        let clone = self.declare_runtime_clone_value();
        let call_site = self
            .builder
            .build_call(clone, &[value.into()], "clone")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing clone result".to_string()))
    }

    fn box_value_as_handle(
        &mut self,
        type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, AotError> {
        Ok(match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let f = self.declare_runtime_box_int();
                let call_site = self
                    .builder
                    .build_call(f, &[value.into()], "box_int")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing box result".to_string()))?
                    .into_pointer_value()
            }
            KiraType::Bool => {
                let f = self.declare_runtime_box_bool();
                let call_site = self
                    .builder
                    .build_call(f, &[value.into()], "box_bool")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing box result".to_string()))?
                    .into_pointer_value()
            }
            KiraType::Float => {
                let f = self.declare_runtime_box_float();
                let call_site = self
                    .builder
                    .build_call(f, &[value.into()], "box_float")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing box result".to_string()))?
                    .into_pointer_value()
            }
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                value.into_pointer_value()
            }
            KiraType::Opaque(_) => {
                return Err(AotError(
                    "opaque handles cannot be boxed into Kira runtime values".to_string(),
                ))
            }
            other => return Err(AotError(format!("cannot box type {:?}", other))),
        })
    }

    fn unbox_handle_if_needed(
        &mut self,
        type_id: TypeId,
        handle: PointerValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        Ok(match self.compiled.types.get(type_id) {
            KiraType::Int => {
                let f = self.declare_runtime_unbox_int();
                let call_site = self
                    .builder
                    .build_call(f, &[handle.into()], "unbox_int")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing unbox result".to_string()))?
            }
            KiraType::Bool => {
                let f = self.declare_runtime_unbox_bool();
                let call_site = self
                    .builder
                    .build_call(f, &[handle.into()], "unbox_bool")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing unbox result".to_string()))?
            }
            KiraType::Float => {
                let f = self.declare_runtime_unbox_float();
                let call_site = self
                    .builder
                    .build_call(f, &[handle.into()], "unbox_float")
                    .map_err(|e| AotError(e.to_string()))?;
                call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| AotError("missing unbox result".to_string()))?
            }
            _ => handle.into(),
        })
    }

    fn const_usize_path(
        &mut self,
        path: &[usize],
        name: &str,
    ) -> Result<(PointerValue<'ctx>, inkwell::values::IntValue<'ctx>), AotError> {
        let usize_ty = self.ptr_sized_int_type();
        let elements = path
            .iter()
            .map(|value| usize_ty.const_int(*value as u64, false))
            .collect::<Vec<_>>();
        let array = usize_ty.const_array(&elements);
        let global = self.module.add_global(array.get_type(), None, name);
        global.set_initializer(&array);
        global.set_constant(true);
        let ptr = global.as_pointer_value();
        let len = usize_ty.const_int(path.len() as u64, false);
        Ok((ptr, len))
    }

    // Runtime helper declarations and calls.

    fn declare_runtime_box_int(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_int",
            handle.fn_type(&[self.context.i64_type().into()], false),
        )
    }

    fn declare_runtime_box_bool(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_bool",
            handle.fn_type(&[self.context.bool_type().into()], false),
        )
    }

    fn declare_runtime_box_float(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_box_float",
            handle.fn_type(&[self.context.f64_type().into()], false),
        )
    }

    fn declare_runtime_unbox_int(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_int",
            self.context.i64_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    fn declare_runtime_unbox_bool(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_bool",
            self.context.bool_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    fn declare_runtime_unbox_float(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_unbox_float",
            self.context.f64_type().fn_type(&[self.value_handle_type().into()], false),
        )
    }

    fn declare_runtime_clone_value(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_clone_value",
            handle.fn_type(&[handle.into()], false),
        )
    }

    fn declare_runtime_make_string(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_make_string",
            handle.fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                ],
                false,
            ),
        )
    }

    fn declare_runtime_print_int(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_int",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.i64_type().into()], false),
        )
    }

    fn declare_runtime_print_bool(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_bool",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.bool_type().into()], false),
        )
    }

    fn declare_runtime_print_float(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_print_float",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), self.context.f64_type().into()], false),
        )
    }

    fn declare_runtime_print_value(&self) -> FunctionValue<'ctx> {
        let ctx = self.context.i8_type().ptr_type(AddressSpace::default());
        let handle = self.value_handle_type();
        self.declare_runtime_function(
            "kira_native_print_value",
            self.context
                .void_type()
                .fn_type(&[ctx.into(), handle.into()], false),
        )
    }

    fn declare_runtime_value_eq(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_value_eq",
            self.context.bool_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    fn call_runtime_value_eq(
        &mut self,
        left: PointerValue<'ctx>,
        right: PointerValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        let f = self.declare_runtime_value_eq();
        let call_site = self
            .builder
            .build_call(f, &[left.into(), right.into()], "value_eq")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing eq result".to_string()))
            .map(|v| v.into_int_value())
    }

    fn declare_runtime_new_array(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_new_array",
            handle.fn_type(&[], false),
        )
    }

    fn call_runtime_new_array(&mut self) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_new_array();
        let call_site = self
            .builder
            .build_call(f, &[], "new_array")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing new array result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    fn declare_runtime_array_push(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_push",
            self.context.void_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    fn call_runtime_array_push(
        &mut self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_array_push();
        self.builder
            .build_call(f, &[array.into(), value.into()], "push")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    fn declare_runtime_array_append(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_append",
            self.context.void_type().fn_type(
                &[self.value_handle_type().into(), self.value_handle_type().into()],
                false,
            ),
        )
    }

    fn call_runtime_array_append(
        &mut self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_array_append();
        self.builder
            .build_call(f, &[array.into(), value.into()], "append")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    fn declare_runtime_array_length(&self) -> FunctionValue<'ctx> {
        self.declare_runtime_function(
            "kira_native_array_length",
            self.context
                .i64_type()
                .fn_type(&[self.value_handle_type().into()], false),
        )
    }

    fn call_runtime_array_length(
        &mut self,
        array: PointerValue<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        let f = self.declare_runtime_array_length();
        let call_site = self
            .builder
            .build_call(f, &[array.into()], "len")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing array length result".to_string()))
            .map(|v| v.into_int_value())
    }

    fn declare_runtime_array_index(&self) -> FunctionValue<'ctx> {
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_array_index",
            handle.fn_type(
                &[handle.into(), self.context.i64_type().into()],
                false,
            ),
        )
    }

    fn call_runtime_array_index(
        &mut self,
        array: PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_array_index();
        let call_site = self
            .builder
            .build_call(f, &[array.into(), index.into()], "idx")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing array index result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    fn declare_runtime_new_struct(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_new_struct",
            handle.fn_type(
                &[
                    self.context.i8_type().ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                ],
                false,
            ),
        )
    }

    fn call_runtime_new_struct(
        &mut self,
        ctx: PointerValue<'ctx>,
        type_id: TypeId,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_new_struct();
        let usize_ty = self.ptr_sized_int_type();
        let type_id_value = usize_ty.const_int(type_id.0 as u64, false);
        let call_site = self
            .builder
            .build_call(f, &[ctx.into(), type_id_value.into()], "new_struct")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing new struct result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    fn declare_runtime_struct_set_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        self.declare_runtime_function(
            "kira_native_struct_set_field",
            self.context.void_type().fn_type(
                &[
                    self.value_handle_type().into(),
                    usize_ty.into(),
                    self.value_handle_type().into(),
                ],
                false,
            ),
        )
    }

    fn call_runtime_struct_set_field(
        &mut self,
        target: PointerValue<'ctx>,
        field_index: usize,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_struct_set_field();
        let usize_ty = self.ptr_sized_int_type();
        let index = usize_ty.const_int(field_index as u64, false);
        self.builder
            .build_call(f, &[target.into(), index.into(), value.into()], "set_field")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    fn declare_runtime_struct_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        let handle = self.context.i8_type().ptr_type(AddressSpace::default());
        self.declare_runtime_function(
            "kira_native_struct_field",
            handle.fn_type(
                &[handle.into(), usize_ty.into()],
                false,
            ),
        )
    }

    fn call_runtime_struct_field(
        &mut self,
        target: PointerValue<'ctx>,
        field_index: usize,
    ) -> Result<PointerValue<'ctx>, AotError> {
        let f = self.declare_runtime_struct_field();
        let usize_ty = self.ptr_sized_int_type();
        let index = usize_ty.const_int(field_index as u64, false);
        let call_site = self
            .builder
            .build_call(f, &[target.into(), index.into()], "get_field")
            .map_err(|e| AotError(e.to_string()))?;
        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| AotError("missing struct field result".to_string()))
            .map(|v| v.into_pointer_value())
    }

    fn declare_runtime_store_struct_field(&self) -> FunctionValue<'ctx> {
        let usize_ty = self.ptr_sized_int_type();
        self.declare_runtime_function(
            "kira_native_store_struct_field",
            self.context.void_type().fn_type(
                &[
                    self.value_handle_type().into(),
                    usize_ty.ptr_type(AddressSpace::default()).into(),
                    usize_ty.into(),
                    self.value_handle_type().into(),
                ],
                false,
            ),
        )
    }

    fn call_runtime_store_struct_field(
        &mut self,
        target: PointerValue<'ctx>,
        path: PointerValue<'ctx>,
        len: inkwell::values::IntValue<'ctx>,
        value: PointerValue<'ctx>,
    ) -> Result<(), AotError> {
        let f = self.declare_runtime_store_struct_field();
        self.builder
            .build_call(f, &[target.into(), path.into(), len.into(), value.into()], "store_field")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(())
    }

    fn declare_runtime_function(
        &self,
        name: &str,
        fn_type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionValue<'ctx> {
        self.module
            .get_function(name)
            .unwrap_or_else(|| self.module.add_function(name, fn_type, None))
    }

    fn llvm_function_type(
        &self,
        signature: &FunctionSignature,
    ) -> Result<inkwell::types::FunctionType<'ctx>, AotError> {
        let mut params = vec![self
            .context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .into()];
        for type_id in &signature.params {
            params.push(
                self.llvm_basic_type(*type_id)
                    .ok_or_else(|| {
                        AotError(format!(
                            "LLVM AOT backend does not yet support parameter type {:?}",
                            self.compiled.types.get(*type_id)
                        ))
                    })?
                    .into(),
            );
        }

        match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => Ok(self.context.void_type().fn_type(&params, false)),
            _ => Ok(self
                .llvm_basic_type(signature.return_type)
                .ok_or_else(|| {
                    AotError(format!(
                        "LLVM AOT backend does not yet support return type {:?}",
                        self.compiled.types.get(signature.return_type)
                    ))
                })?
                .fn_type(&params, false)),
        }
    }

    fn llvm_basic_type(&self, type_id: TypeId) -> Option<BasicTypeEnum<'ctx>> {
        match self.compiled.types.get(type_id) {
            KiraType::Int => Some(self.context.i64_type().into()),
            KiraType::Float => Some(self.context.f64_type().into()),
            KiraType::Bool => Some(self.context.bool_type().into()),
            KiraType::String
            | KiraType::Dynamic
            | KiraType::Array(_)
            | KiraType::Struct(_)
            | KiraType::Opaque(_) => {
                Some(self.value_handle_type())
            }
            _ => None,
        }
    }

    fn value_handle_type(&self) -> BasicTypeEnum<'ctx> {
        self.context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .into()
    }

    fn ensure_supported_signature(
        &self,
        signature: &FunctionSignature,
        name: &str,
    ) -> Result<(), AotError> {
        for type_id in &signature.params {
            self.ensure_primitive_type(*type_id, name)?;
        }
        self.ensure_primitive_or_unit_type(signature.return_type, name)
    }

    fn ensure_supported_chunk(
        &self,
        function: &CompiledFunction,
        chunk: &Chunk,
    ) -> Result<(), AotError> {
        for (slot, type_id) in chunk.local_types.iter().enumerate().take(chunk.local_count) {
            self.ensure_primitive_type(*type_id, &format!("{} local {}", function.name, slot))?;
        }
        for constant in &chunk.constants {
            match constant {
                Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Unit | Value::String(_) => {}
                Value::Array(_) | Value::Struct(_) => {
                    return Err(AotError(format!(
                        "LLVM AOT backend does not yet support constant {:?} in `{}`",
                        constant, function.name
                    )))
                }
            }
        }
        Ok(())
    }

    fn ensure_primitive_or_unit_type(&self, type_id: TypeId, name: &str) -> Result<(), AotError> {
        if self.llvm_basic_type(type_id).is_some() || self.compiled.types.get(type_id) == &KiraType::Unit {
            Ok(())
        } else {
            Err(AotError(format!(
                "LLVM AOT backend does not yet support type {:?} in `{}`",
                self.compiled.types.get(type_id),
                name
            )))
        }
    }

    fn ensure_primitive_type(&self, type_id: TypeId, name: &str) -> Result<(), AotError> {
        if self.llvm_basic_type(type_id).is_some() {
            Ok(())
        } else {
            Err(AotError(format!(
                "LLVM AOT backend does not yet support type {:?} in `{}`",
                self.compiled.types.get(type_id),
                name
            )))
        }
    }
}
