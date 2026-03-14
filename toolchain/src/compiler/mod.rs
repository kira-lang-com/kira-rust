mod build_artifacts;
mod builtins;
mod ffi;
mod eligibility;
mod functions;
mod lowering;
mod metadata;
mod native_build;
mod types;

#[cfg(test)]
mod tests;

pub use types::{
    AotArtifact, AotBuildPlan, AotJob, BackendKind, BuildStage, BuiltinFunction, Chunk,
    CompiledFunction, CompiledModule, FfiFunction, FfiLink, FfiMetadata,
    FunctionArtifacts, FunctionSignature, Instruction,
};
pub use native_build::build_all_native_dependencies;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError(pub String);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CompileError {}

use std::collections::HashMap;

use crate::ast::{ExecutionMode, Program, TopLevelItem};
use crate::runtime::type_system::TypeSystem;

use build_artifacts::build_aot_plan;
use builtins::builtin_functions;
use eligibility::is_native_eligible;
use functions::{build_signature, collect_signatures};
use lowering::lower_function_body;
use metadata::{build_platform_model, resolve_function_attributes};

pub fn compile(program: &Program) -> Result<CompiledModule, CompileError> {
    if !program.links.is_empty() {
        return Err(CompileError(
            "@Link directives are only supported when compiling a loaded project".to_string(),
        ));
    }
    compile_impl(program, None)
}

pub fn compile_project(program: &Program, project_root: &std::path::Path) -> Result<CompiledModule, CompileError> {
    compile_impl(program, Some(project_root))
}

fn compile_impl(program: &Program, project_root: Option<&std::path::Path>) -> Result<CompiledModule, CompileError> {
    let platform_model = build_platform_model(program.platforms.as_ref())?;
    let mut types = TypeSystem::default();
    register_struct_types(&mut types, program)?;
    let builtins = builtin_functions(&mut types);
    let mut functions = HashMap::new();

    let (ffi, ffi_signatures) = ffi::build_ffi_metadata(&mut types, &program.links, project_root)?;

    for item in &program.items {
        if let TopLevelItem::Function(function) = item {
            let resolved = resolve_function_attributes(function, platform_model.as_ref())?;
            let signature = build_signature(&mut types, function)?;
            functions.insert(
                function.name.name.clone(),
                CompiledFunction {
                    name: function.name.name.clone(),
                    declared_mode: resolved.declared_mode,
                    target_platforms: resolved.target_platforms,
                    selected_backend: BackendKind::Vm,
                    signature,
                    artifacts: FunctionArtifacts::default(),
                },
            );
        }
    }

    let mut signatures = collect_signatures(&builtins, &functions);
    for (name, signature) in ffi_signatures {
        if builtins.contains_key(&name) {
            return Err(CompileError(format!(
                "linked C function `{}` conflicts with an existing builtin",
                name
            )));
        }
        if functions.contains_key(&name) {
            return Err(CompileError(format!(
                "linked C function `{}` conflicts with a user-defined function",
                name
            )));
        }
        signatures.insert(name, signature);
    }

    for item in &program.items {
        if let TopLevelItem::Function(function) = item {
            let compiled = functions.get_mut(&function.name.name).ok_or_else(|| {
                CompileError(format!(
                    "missing function record for `{}`",
                    function.name.name
                ))
            })?;

            let bytecode = lower_function_body(function, &mut types, &signatures)?;
            let native_eligible =
                is_native_eligible(function, &mut types, &signatures, &builtins)?;
            let calls_ffi = ffi::chunk_contains_ffi_calls(&bytecode, &ffi);

        let selected_backend = match compiled.declared_mode {
            ExecutionMode::Runtime => BackendKind::Vm,
            ExecutionMode::Native => {
                if !native_eligible {
                    return Err(CompileError(format!(
                        "function `{}` is declared native but uses features that are not yet native-lowerable",
                        function.name.name
                    )));
                }
                BackendKind::Native
            }
            ExecutionMode::Auto => {
                if native_eligible {
                        BackendKind::Native
                    } else {
                        BackendKind::Vm
                    }
                }
            };

            if calls_ffi && selected_backend != BackendKind::Native {
                return Err(CompileError(format!(
                    "function `{}` calls a linked C function but is not compiled as native",
                    function.name.name
                )));
            }

            if selected_backend == BackendKind::Vm {
                let sig = &compiled.signature;
                if !ffi::type_is_vm_compatible(&types, sig.return_type)
                    || sig
                        .params
                        .iter()
                        .copied()
                        .any(|type_id| !ffi::type_is_vm_compatible(&types, type_id))
                {
                    return Err(CompileError(format!(
                        "function `{}` uses native-only types and cannot run in the VM",
                        function.name.name
                    )));
                }
            }

            let aot = if selected_backend == BackendKind::Native {
                Some(AotArtifact {
                    symbol: format!("kira_native_{}", function.name.name),
                    target_platforms: compiled.target_platforms.clone(),
                    stage: BuildStage::BuildTimeOnly,
                })
            } else {
                None
            };

            compiled.selected_backend = selected_backend;
            compiled.artifacts = FunctionArtifacts {
                bytecode: Some(bytecode),
                aot,
            };
        }
    }

    let aot_plan = build_aot_plan(functions.values());

    Ok(CompiledModule {
        platforms: program.platforms.clone(),
        aot_plan,
        types,
        builtins,
        ffi,
        functions,
    })
}

fn register_struct_types(types: &mut TypeSystem, program: &Program) -> Result<(), CompileError> {
    for item in &program.items {
        let TopLevelItem::Struct(definition) = item else {
            continue;
        };
        types
            .declare_struct(&definition.name.name)
            .map_err(CompileError)?;
    }

    for item in &program.items {
        let TopLevelItem::Struct(definition) = item else {
            continue;
        };

        let mut fields = Vec::with_capacity(definition.fields.len());
        for field in &definition.fields {
            let type_id = types.ensure_named(&field.type_name.name).ok_or_else(|| {
                CompileError(format!(
                    "unknown field type `{}` on struct `{}`",
                    field.type_name.name, definition.name.name
                ))
            })?;
            fields.push((field.name.name.clone(), type_id));
        }

        types
            .define_struct(&definition.name.name, fields)
            .map_err(CompileError)?;
    }

    Ok(())
}
