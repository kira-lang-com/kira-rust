// LLVM context and codegen initialization

use std::collections::HashMap;
use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::FunctionValue;
use inkwell::OptimizationLevel;

use crate::compiler::{BackendKind, CompiledModule};

use crate::aot::error::AotError;

pub struct NativeCodegen<'ctx> {
    pub(super) module: Module<'ctx>,
    pub(super) builder: Builder<'ctx>,
    pub(super) context: &'ctx Context,
    pub(super) target_machine: TargetMachine,
    pub(super) compiled: &'ctx CompiledModule,
    pub(super) function_values: HashMap<String, FunctionValue<'ctx>>,
    pub(super) bridge_values: HashMap<String, FunctionValue<'ctx>>,
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
}
