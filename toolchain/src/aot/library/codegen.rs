use std::collections::{HashMap, HashSet};
use std::path::Path;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};

use crate::compiler::{Chunk, CompiledFunction, CompiledModule, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId};
use crate::runtime::Value;

use crate::aot::error::AotError;
use crate::aot::stack::{infer_stack_layout, StackState};

pub struct ExportSpec {
    pub exported_functions: HashSet<String>,
    pub exported_structs: HashSet<String>,
    pub closure_functions: HashSet<String>,
}

pub struct CAbiCodegen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    context: &'ctx Context,
    target_machine: TargetMachine,
    compiled: &'ctx CompiledModule,
    export_spec: &'ctx ExportSpec,
    function_values: HashMap<String, FunctionValue<'ctx>>,
    struct_types: HashMap<TypeId, StructType<'ctx>>,
    building_structs: Vec<TypeId>,
}

impl<'ctx> CAbiCodegen<'ctx> {
    pub fn new(
        project_name: &str,
        compiled: &'ctx CompiledModule,
        export_spec: &'ctx ExportSpec,
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
            export_spec,
            function_values: HashMap::new(),
            struct_types: HashMap::new(),
            building_structs: Vec::new(),
        };

        codegen.declare_functions()?;
        codegen.emit_functions()?;
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

    fn declare_functions(&mut self) -> Result<(), AotError> {
        for name in &self.export_spec.closure_functions {
            let function = self
                .compiled
                .functions
                .get(name)
                .ok_or_else(|| AotError(format!("missing function `{name}` in compiled module")))?;
            let fn_type = self.llvm_function_type(&function.signature)?;

            let is_exported = self.export_spec.exported_functions.contains(name);
            let symbol = if is_exported {
                name.clone()
            } else {
                format!("kira_internal_{name}")
            };

            let value = self.module.add_function(&symbol, fn_type, None);
            if !is_exported {
                value.set_linkage(inkwell::module::Linkage::Internal);
            }
            self.function_values.insert(name.clone(), value);
        }
        Ok(())
    }

    fn emit_functions(&mut self) -> Result<(), AotError> {
        for name in &self.export_spec.closure_functions {
            let function = self
                .compiled
                .functions
                .get(name)
                .ok_or_else(|| AotError(format!("missing function `{name}` in compiled module")))?;
            let chunk = function.artifacts.bytecode.as_ref().ok_or_else(|| {
                AotError(format!(
                    "missing bytecode shadow for library function `{}`",
                    function.name
                ))
            })?;
            self.emit_function(function, chunk)?;
        }
        Ok(())
    }

    fn emit_function(&mut self, function: &CompiledFunction, chunk: &Chunk) -> Result<(), AotError> {
        self.ensure_c_abi_signature(&function.signature, &function.name)?;
        self.ensure_c_abi_chunk(function, chunk)?;

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

        self.builder.position_at_end(entry);

        let locals = self.build_local_allocas(chunk)?;
        let stack_layout = infer_stack_layout(self.compiled, chunk).map_err(|error| {
            AotError(format!(
                "failed to infer native stack layout for `{}`: {}",
                function.name, error
            ))
        })?;
        let stack_slots = self.build_stack_allocas(&stack_layout.stack_slot_types)?;

        for (index, local) in locals
            .iter()
            .enumerate()
            .take(function.signature.params.len())
        {
            let param = function_value
                .get_nth_param(index as u32)
                .ok_or_else(|| AotError(format!("missing parameter {index} for `{}`", function.name)))?;
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
                &function.signature,
                chunk,
                index,
                state,
                &locals,
                &stack_slots,
                &blocks,
            )?;
        }

        Ok(())
    }

    fn build_local_allocas(&mut self, chunk: &Chunk) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut locals = Vec::with_capacity(chunk.local_count);
        for (index, type_id) in chunk.local_types.iter().copied().enumerate().take(chunk.local_count) {
            let llvm_type = self.llvm_abi_type(type_id)?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, &format!("local_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            locals.push(alloca);
        }
        Ok(locals)
    }

    fn build_stack_allocas(&mut self, stack_slot_types: &[TypeId]) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut slots = Vec::with_capacity(stack_slot_types.len());
        for (index, type_id) in stack_slot_types.iter().copied().enumerate() {
            let llvm_type = self.llvm_abi_type(type_id)?;
            let alloca = self
                .builder
                .build_alloca(llvm_type, &format!("stack_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            slots.push(alloca);
        }
        Ok(slots)
    }

    fn emit_instruction(
        &mut self,
        function_value: FunctionValue<'ctx>,
        signature: &FunctionSignature,
        chunk: &Chunk,
        index: usize,
        state: &StackState,
        locals: &[PointerValue<'ctx>],
        stack_slots: &[PointerValue<'ctx>],
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
                let value = self.load_typed_ptr(local, type_id, "local_value")?;
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
                self.builder
                    .build_store(local, value)
                    .map_err(|e| AotError(e.to_string()))?;
            }
            Instruction::Negate => {
                let type_id = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on negate".to_string()))?;
                let value = self.load_stack(stack_slots, depth - 1, type_id, "neg_arg")?;
                let result = match self.compiled.types.get(type_id) {
                    KiraType::Int => self
                        .builder
                        .build_int_neg(value.into_int_value(), "neg")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    KiraType::Float => self
                        .builder
                        .build_float_neg(value.into_float_value(), "fneg")
                        .map_err(|e| AotError(e.to_string()))?
                        .into(),
                    other => return Err(AotError(format!("negation not supported for {:?}", other))),
                };
                self.store_stack(stack_slots, depth - 1, type_id, result)?;
            }
            Instruction::CastIntToFloat => {
                let src = self.load_stack(stack_slots, depth - 1, self.compiled.types.int(), "int")?;
                let float = self
                    .builder
                    .build_signed_int_to_float(src.into_int_value(), self.context.f64_type(), "i2f")
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
                    (_, other) => return Err(AotError(format!("arithmetic not supported for {:?}", other))),
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
                self.store_stack(stack_slots, depth - 2, self.compiled.types.bool(), result.into())?;
            }
            Instruction::BuildStruct { type_id, field_count } => {
                let KiraType::Struct(struct_type) = self.compiled.types.get(*type_id) else {
                    return Err(AotError("BuildStruct target type is not a struct".to_string()));
                };
                if struct_type.fields.len() != *field_count {
                    return Err(AotError("struct field count mismatch".to_string()));
                }

                let mut values = Vec::with_capacity(*field_count);
                for offset in 0..*field_count {
                    let field_index = *field_count - 1 - offset;
                    let slot = depth - 1 - offset;
                    let field_type_id = struct_type.fields[field_index].type_id;
                    let value = self.load_stack(stack_slots, slot, field_type_id, "field")?;
                    values.push((field_index, field_type_id, value));
                }
                values.reverse();

                let mut struct_value: BasicValueEnum<'ctx> = self
                    .llvm_struct_type(*type_id)?
                    .get_undef()
                    .into();
                for (field_index, field_type, value) in values {
                    let inserted = self.insert_struct_field(*type_id, struct_value, field_index, field_type, value)?;
                    struct_value = inserted;
                }

                self.store_stack(stack_slots, depth - *field_count, *type_id, struct_value)?;
            }
            Instruction::StructField(field_index) => {
                let target_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on struct field".to_string()))?;
                let struct_value = self.load_stack(stack_slots, depth - 1, target_type, "struct")?;
                let field_type = self
                    .compiled
                    .types
                    .struct_fields(target_type)
                    .and_then(|fields| fields.get(*field_index))
                    .map(|field| field.type_id)
                    .ok_or_else(|| AotError("invalid struct field index".to_string()))?;
                let extracted = self.extract_struct_field(target_type, struct_value, *field_index, field_type)?;
                self.store_stack(stack_slots, depth - 1, field_type, extracted)?;
            }
            Instruction::StoreLocalField { local, path } => {
                let value_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow on field store".to_string()))?;
                let value = self.load_stack(stack_slots, depth - 1, value_type, "field_value")?;
                let target_type = *chunk
                    .local_types
                    .get(*local)
                    .ok_or_else(|| AotError("invalid local index".to_string()))?;
                let local_ptr = *locals
                    .get(*local)
                    .ok_or_else(|| AotError("missing local slot".to_string()))?;
                let current = self.load_typed_ptr(local_ptr, target_type, "target")?;
                let updated = self.set_struct_field_path(target_type, current, path, value_type, value)?;
                self.builder
                    .build_store(local_ptr, updated)
                    .map_err(|e| AotError(e.to_string()))?;
            }
            Instruction::JumpIfFalse(target) => {
                if index + 1 >= blocks.len() {
                    return Err(AotError("JumpIfFalse at end of function".to_string()));
                }
                let cond = self.load_stack(stack_slots, depth - 1, self.compiled.types.bool(), "cond")?;
                self.builder
                    .build_conditional_branch(cond.into_int_value(), blocks[*target], blocks[index + 1])
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

                let result = self.emit_call(function_value, function, &signature, &args)?;
                if self.compiled.types.get(signature.return_type) != &KiraType::Unit {
                    self.store_stack(stack_slots, base, signature.return_type, result)?;
                }
            }
            Instruction::Pop => {}
            Instruction::Return => {
                match self.compiled.types.get(signature.return_type) {
                    KiraType::Unit => {
                        self.builder
                            .build_return(None)
                            .map_err(|e| AotError(e.to_string()))?;
                    }
                    _ => {
                        let type_id = signature.return_type;
                        let value = self.load_stack(stack_slots, depth - 1, type_id, "ret_value")?;
                        self.builder
                            .build_return(Some(&value))
                            .map_err(|e| AotError(e.to_string()))?;
                    }
                }
                return Ok(());
            }
            other => {
                return Err(AotError(format!(
                    "instruction {:?} is not supported in --lib output",
                    other
                )));
            }
        }

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
                    .build_float_compare(pred, left.into_float_value(), right.into_float_value(), "fcmp")
                    .map_err(|e| AotError(e.to_string()))
            }
            KiraType::String => {
                if !matches!(instruction, Instruction::Equal | Instruction::NotEqual) {
                    return Err(AotError("only == and != are supported for string".to_string()));
                }
                let pred = if matches!(instruction, Instruction::Equal) {
                    IntPredicate::EQ
                } else {
                    IntPredicate::NE
                };
                let left = left.into_pointer_value();
                let right = right.into_pointer_value();
                self.builder
                    .build_int_compare(
                        pred,
                        self.ptr_to_int(left)?,
                        self.ptr_to_int(right)?,
                        "pcmp",
                    )
                    .map_err(|e| AotError(e.to_string()))
            }
            other => Err(AotError(format!("comparison not supported for {:?}", other))),
        }
    }

    fn emit_call(
        &mut self,
        _function_value: FunctionValue<'ctx>,
        callee: &str,
        signature: &FunctionSignature,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        if let Some(ffi) = self.compiled.ffi.functions.get(callee) {
            let fn_value = self.declare_ffi_function(callee, &ffi.signature)?;
            let mut call_args = Vec::with_capacity(args.len());
            for value in args.iter().copied() {
                call_args.push(value.into());
            }
            let call_site = self
                .builder
                .build_call(
                    fn_value,
                    &call_args,
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

        let fn_value = *self.function_values.get(callee).ok_or_else(|| {
            AotError(format!("missing library callee `{}`", callee))
        })?;
        let mut call_args = Vec::with_capacity(args.len());
        for value in args.iter().copied() {
            call_args.push(value.into());
        }
        let call_site = self
            .builder
            .build_call(
                fn_value,
                &call_args,
                "call",
            )
            .map_err(|e| AotError(e.to_string()))?;

        if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
            Ok(self.context.i64_type().const_zero().into())
        } else {
            call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| AotError("missing call result".to_string()))
        }
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
            params.push(self.llvm_abi_type(*type_id)?);
        }
        let param_types = params.iter().copied().map(Into::into).collect::<Vec<_>>();
        let fn_type = match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => self.context.void_type().fn_type(&param_types, false),
            _ => self.llvm_abi_type(signature.return_type)?.fn_type(&param_types, false),
        };
        Ok(self.module.add_function(symbol, fn_type, None))
    }

    fn llvm_function_type(
        &mut self,
        signature: &FunctionSignature,
    ) -> Result<inkwell::types::FunctionType<'ctx>, AotError> {
        let mut params = Vec::with_capacity(signature.params.len());
        for type_id in &signature.params {
            params.push(self.llvm_abi_type(*type_id)?);
        }
        let param_types = params.iter().copied().map(Into::into).collect::<Vec<_>>();
        match self.compiled.types.get(signature.return_type) {
            KiraType::Unit => Ok(self.context.void_type().fn_type(&param_types, false)),
            _ => Ok(self.llvm_abi_type(signature.return_type)?.fn_type(&param_types, false)),
        }
    }

    fn llvm_abi_type(&mut self, type_id: TypeId) -> Result<BasicTypeEnum<'ctx>, AotError> {
        Ok(match self.compiled.types.get(type_id) {
            KiraType::Int => self.context.i64_type().into(),
            KiraType::Float => self.context.f64_type().into(),
            KiraType::Bool => self.context.bool_type().into(),
            KiraType::String => self.context.i8_type().ptr_type(AddressSpace::default()).into(),
            KiraType::Struct(_) => self.llvm_struct_type(type_id)?.into(),
            other => {
                return Err(AotError(format!(
                    "type {:?} is not supported in --lib output",
                    other
                )))
            }
        })
    }

    fn llvm_struct_type(&mut self, type_id: TypeId) -> Result<StructType<'ctx>, AotError> {
        if let Some(existing) = self.struct_types.get(&type_id).copied() {
            return Ok(existing);
        }
        if self.building_structs.contains(&type_id) {
            return Err(AotError("recursive structs are not supported in --lib output".to_string()));
        }
        self.building_structs.push(type_id);

        let fields = self
            .compiled
            .types
            .struct_fields(type_id)
            .ok_or_else(|| AotError("missing struct fields".to_string()))?;

        let mut llvm_fields = Vec::with_capacity(fields.len());
        for field in fields {
            let field_ty = match self.compiled.types.get(field.type_id) {
                KiraType::Bool => self.context.i8_type().into(), // C `_Bool` layout
                _ => self.llvm_abi_type(field.type_id)?,
            };
            llvm_fields.push(field_ty);
        }

        let struct_ty = self.context.struct_type(&llvm_fields, false);
        self.struct_types.insert(type_id, struct_ty);
        self.building_structs.pop();
        Ok(struct_ty)
    }

    fn llvm_const(&mut self, value: &Value) -> Result<(TypeId, BasicValueEnum<'ctx>), AotError> {
        Ok(match value {
            Value::Bool(b) => (
                self.compiled.types.bool(),
                self.context
                    .bool_type()
                    .const_int(*b as u64, false)
                    .into(),
            ),
            Value::Int(i) => (
                self.compiled.types.int(),
                self.context.i64_type().const_int(*i as u64, true).into(),
            ),
            Value::Float(f) => (self.compiled.types.float(), self.context.f64_type().const_float(f.0).into()),
            Value::String(s) => {
                let global = self
                    .builder
                    .build_global_string_ptr(s, "kira_cstr")
                    .map_err(|e| AotError(e.to_string()))?;
                let ptr = global.as_pointer_value();
                (
                    self.compiled
                        .types
                        .resolve_named("string")
                        .ok_or_else(|| AotError("missing string type".to_string()))?,
                    ptr.into(),
                )
            }
            Value::Unit => {
                return Err(AotError(
                    "unit constants are not supported as stack values in --lib output".to_string(),
                ))
            }
            Value::Array(_) | Value::Struct(_) => {
                return Err(AotError("aggregate constants are not supported in --lib output".to_string()))
            }
        })
    }

    fn load_typed_ptr(
        &mut self,
        ptr: PointerValue<'ctx>,
        type_id: TypeId,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ty = self.llvm_abi_type(type_id)?;
        self.builder
            .build_load(ty, ptr, name)
            .map_err(|e| AotError(e.to_string()))
    }

    fn load_stack(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        type_id: TypeId,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        let value = self.load_typed_ptr(ptr, type_id, name)?;
        match self.compiled.types.get(type_id) {
            KiraType::Bool => Ok(value.into_int_value().into()),
            _ => Ok(value),
        }
    }

    fn store_stack(
        &mut self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        type_id: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        let ty = self.llvm_abi_type(type_id)?;
        let stored = match self.compiled.types.get(type_id) {
            KiraType::Bool => value.into_int_value().into(),
            _ => value,
        };
        self.builder
            .build_store(ptr, stored)
            .map_err(|e| AotError(e.to_string()))?;
        // Keep the store type stable for bool-in-struct conversions; handled elsewhere.
        let _ = ty;
        Ok(())
    }

    fn insert_struct_field(
        &mut self,
        struct_type: TypeId,
        target: BasicValueEnum<'ctx>,
        field_index: usize,
        field_type: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let field_value = match self.compiled.types.get(field_type) {
            KiraType::Bool => {
                self.builder
                    .build_int_z_extend(value.into_int_value(), self.context.i8_type(), "b2u8")
                    .map_err(|e| AotError(e.to_string()))?
                    .into()
            }
            _ => value,
        };
        let inserted = self
            .builder
            .build_insert_value(
                target.into_struct_value(),
                field_value,
                field_index as u32,
                "ins",
            )
            .map_err(|e| AotError(e.to_string()))?;
        let _ = struct_type;
        Ok(inserted.into_struct_value().into())
    }

    fn extract_struct_field(
        &mut self,
        _struct_type: TypeId,
        target: BasicValueEnum<'ctx>,
        field_index: usize,
        field_type: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let raw = self
            .builder
            .build_extract_value(target.into_struct_value(), field_index as u32, "ext")
            .map_err(|e| AotError(e.to_string()))?;
        Ok(match self.compiled.types.get(field_type) {
            KiraType::Bool => {
                let zero = self.context.i8_type().const_zero();
                self.builder
                    .build_int_compare(
                        IntPredicate::NE,
                        raw.into_int_value(),
                        zero,
                        "u82b",
                    )
                    .map_err(|e| AotError(e.to_string()))?
                    .into()
            }
            _ => raw,
        })
    }

    fn set_struct_field_path(
        &mut self,
        struct_type: TypeId,
        current: BasicValueEnum<'ctx>,
        path: &[usize],
        value_type: TypeId,
        value: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let Some((field_index, rest)) = path.split_first() else {
            return Ok(value);
        };

        let fields = self
            .compiled
            .types
            .struct_fields(struct_type)
            .ok_or_else(|| AotError("missing struct fields".to_string()))?;
        let field_type = fields
            .get(*field_index)
            .ok_or_else(|| AotError("invalid struct field index".to_string()))?
            .type_id;

        if rest.is_empty() {
            return self.insert_struct_field(struct_type, current, *field_index, field_type, value);
        }

        let extracted = self.extract_struct_field(struct_type, current, *field_index, field_type)?;
        let updated_child = self.set_struct_field_path(field_type, extracted, rest, value_type, value)?;
        self.insert_struct_field(struct_type, current, *field_index, field_type, updated_child)
    }

    fn ptr_sized_int_type(&self) -> inkwell::types::IntType<'ctx> {
        self.target_machine
            .get_target_data()
            .ptr_sized_int_type_in_context(self.context, None)
    }

    fn ptr_to_int(&self, ptr: inkwell::values::PointerValue<'ctx>) -> Result<inkwell::values::IntValue<'ctx>, AotError> {
        self.builder
            .build_ptr_to_int(ptr, self.ptr_sized_int_type(), "ptrtoint")
            .map_err(|e| AotError(e.to_string()))
    }

    fn ensure_c_abi_signature(&self, signature: &FunctionSignature, name: &str) -> Result<(), AotError> {
        for type_id in &signature.params {
            self.ensure_c_abi_type(*type_id, name)?;
        }
        self.ensure_c_abi_type(signature.return_type, name)
    }

    fn ensure_c_abi_chunk(&self, function: &CompiledFunction, chunk: &Chunk) -> Result<(), AotError> {
        for (slot, type_id) in chunk.local_types.iter().copied().enumerate().take(chunk.local_count) {
            self.ensure_c_abi_type(type_id, &format!("{} local {}", function.name, slot))?;
        }
        for constant in &chunk.constants {
            match constant {
                Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::String(_) => {}
                Value::Unit => {
                    return Err(AotError(format!(
                        "unit constants are not supported in `{}` for --lib output",
                        function.name
                    )))
                }
                Value::Array(_) | Value::Struct(_) => {
                    return Err(AotError(format!(
                        "aggregate constants are not supported in `{}` for --lib output",
                        function.name
                    )))
                }
            }
        }
        Ok(())
    }

    fn ensure_c_abi_type(&self, type_id: TypeId, name: &str) -> Result<(), AotError> {
        match self.compiled.types.get(type_id) {
            KiraType::Unit
            | KiraType::Int
            | KiraType::Float
            | KiraType::Bool
            | KiraType::String
            | KiraType::Struct(_) => Ok(()),
            other => Err(AotError(format!(
                "--lib output does not support type {:?} in `{}`",
                other, name
            ))),
        }
    }
}
