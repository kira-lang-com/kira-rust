use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    compile, BackendKind, Chunk, CompiledFunction, CompiledModule, FunctionSignature, Instruction,
};
use crate::project::load_project;
use crate::runtime::{
    type_system::{KiraType, TypeId},
    Value,
};

#[derive(Debug)]
pub struct AotError(pub String);

impl std::fmt::Display for AotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AotError {}

pub fn build_default_project(project_root: &Path, out_root: &Path) -> Result<PathBuf, AotError> {
    let project = load_project(project_root).map_err(|error| AotError(error.to_string()))?;
    let module = compile(&project.program).map_err(|error| AotError(error.to_string()))?;

    let out_root = resolve_output_root(out_root)?;
    remove_path_if_exists(&out_root.join("build"), "legacy build output")?;
    remove_path_if_exists(&out_root.join("compiled_module.bin"), "legacy compiled module")?;

    let staging_root = out_root.join(".kira-build").join(&project.manifest.name);
    recreate_dir(&staging_root, "staging build")?;

    let final_bundle_dir = out_root.join(&project.manifest.name);
    remove_path_if_exists(&final_bundle_dir, "old app bundle")?;
    fs::create_dir_all(&final_bundle_dir).map_err(|error| {
        AotError(format!(
            "failed to create final app bundle `{}`: {}",
            final_bundle_dir.display(),
            error
        ))
    })?;

    let native_archive = build_native_archive(&project.manifest.name, &module, &staging_root)?;
    let module_bin = staging_root.join("compiled_module.bin");
    fs::write(
        &module_bin,
        bincode::serialize(&module)
            .map_err(|error| AotError(format!("module serialization failed: {error}")))?,
    )
    .map_err(|error| {
        AotError(format!(
            "failed to write `{}`: {}",
            module_bin.display(),
            error
        ))
    })?;

    let runner_dir = staging_root.join("runner");
    write_runner_project(
        &runner_dir,
        &project.manifest.name,
        &module,
        &module_bin,
        &native_archive,
        &project.entry_symbol,
    )?;
    build_runner_project(&runner_dir)?;

    let binary_name = project.manifest.name.clone();
    let built_binary = runner_dir.join("target/release").join(&binary_name);
    let final_binary = final_bundle_dir.join(&binary_name);
    let final_module = final_bundle_dir.join("compiled_module.bin");

    fs::copy(&built_binary, &final_binary).map_err(|error| {
        AotError(format!(
            "failed to copy built binary from `{}` to `{}`: {}",
            built_binary.display(),
            final_binary.display(),
            error
        ))
    })?;
    fs::copy(&module_bin, &final_module).map_err(|error| {
        AotError(format!(
            "failed to copy compiled module from `{}` to `{}`: {}",
            module_bin.display(),
            final_module.display(),
            error
        ))
    })?;

    Ok(final_binary)
}

fn resolve_output_root(out_root: &Path) -> Result<PathBuf, AotError> {
    if out_root.exists() {
        return std::fs::canonicalize(out_root)
            .map_err(|error| AotError(format!("failed to resolve out directory: {error}")));
    }

    let candidate = std::env::current_dir()
        .map_err(|error| AotError(format!("failed to resolve current directory: {error}")))?
        .join(out_root);
    fs::create_dir_all(&candidate).map_err(|error| {
        AotError(format!(
            "failed to create output directory `{}`: {}",
            candidate.display(),
            error
        ))
    })?;
    std::fs::canonicalize(candidate)
        .map_err(|error| AotError(format!("failed to resolve out directory: {error}")))
}

fn recreate_dir(path: &Path, label: &str) -> Result<(), AotError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| {
            AotError(format!(
                "failed to clean {label} directory `{}`: {}",
                path.display(),
                error
            ))
        })?;
    }
    fs::create_dir_all(path).map_err(|error| {
        AotError(format!(
            "failed to create {label} directory `{}`: {}",
            path.display(),
            error
        ))
    })
}

fn remove_path_if_exists(path: &Path, label: &str) -> Result<(), AotError> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| {
            AotError(format!(
                "failed to remove {label} directory `{}`: {}",
                path.display(),
                error
            ))
        })?;
    } else {
        fs::remove_file(path).map_err(|error| {
            AotError(format!(
                "failed to remove {label} file `{}`: {}",
                path.display(),
                error
            ))
        })?;
    }

    Ok(())
}

pub fn run_default_project(project_root: &Path, out_root: &Path) -> Result<i32, AotError> {
    let binary = build_default_project(project_root, out_root)?;
    let status = Command::new(&binary).status().map_err(|error| {
        AotError(format!(
            "failed to execute `{}`: {}",
            binary.display(),
            error
        ))
    })?;
    Ok(status.code().unwrap_or(1))
}

fn build_native_archive(
    project_name: &str,
    module: &CompiledModule,
    build_root: &Path,
) -> Result<PathBuf, AotError> {
    let object_path = build_root.join("kira_native.o");
    if module.aot_plan.jobs.is_empty() {
        return create_empty_archive(build_root);
    }

    let context = Context::create();
    let codegen = NativeCodegen::new(project_name, module, &context)?;
    codegen.write_object(&object_path)?;

    let archive_path = build_root.join("libkira_native.a");
    let status = Command::new("libtool")
        .arg("-static")
        .arg("-o")
        .arg(&archive_path)
        .arg(&object_path)
        .status()
        .map_err(|error| AotError(format!("failed to invoke libtool: {error}")))?;
    if !status.success() {
        return Err(AotError(
            "libtool failed to create native archive".to_string(),
        ));
    }
    Ok(archive_path)
}

fn create_empty_archive(build_root: &Path) -> Result<PathBuf, AotError> {
    let empty_c = build_root.join("empty.c");
    let empty_o = build_root.join("empty.o");
    let archive = build_root.join("libkira_native.a");
    fs::write(&empty_c, "void kira_native_archive_placeholder(void) {}\n").map_err(|error| {
        AotError(format!(
            "failed to write `{}`: {}",
            empty_c.display(),
            error
        ))
    })?;
    let status = Command::new("clang")
        .arg("-c")
        .arg(&empty_c)
        .arg("-o")
        .arg(&empty_o)
        .status()
        .map_err(|error| AotError(format!("failed to compile empty archive stub: {error}")))?;
    if !status.success() {
        return Err(AotError(
            "clang failed to compile empty archive stub".to_string(),
        ));
    }
    let status = Command::new("libtool")
        .arg("-static")
        .arg("-o")
        .arg(&archive)
        .arg(&empty_o)
        .status()
        .map_err(|error| AotError(format!("failed to archive empty native stub: {error}")))?;
    if !status.success() {
        return Err(AotError(
            "libtool failed to archive empty native stub".to_string(),
        ));
    }
    Ok(archive)
}

fn write_runner_project(
    runner_dir: &Path,
    project_name: &str,
    module: &CompiledModule,
    module_bin: &Path,
    native_archive: &Path,
    entry_symbol: &str,
) -> Result<(), AotError> {
    fs::create_dir_all(runner_dir.join("src")).map_err(|error| {
        AotError(format!(
            "failed to create runner directory `{}`: {}",
            runner_dir.display(),
            error
        ))
    })?;

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = format!(
        "[package]\nname = \"{name}_runner\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[[bin]]\nname = \"{name}\"\npath = \"src/main.rs\"\n\n[dependencies]\nbincode = \"1.3.3\"\nordered-float = \"5.1.0\"\ntoolchain = {{ path = \"{toolchain}\" }}\n",
        name = project_name,
        toolchain = crate_root.display()
    );
    fs::write(runner_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|error| AotError(format!("failed to write runner Cargo.toml: {error}")))?;

    let build_rs = format!(
        "fn main() {{\n    println!(\"cargo:rustc-link-search=native={}\");\n    println!(\"cargo:rustc-link-lib=static=kira_native\");\n}}\n",
        native_archive.parent().unwrap().display()
    );
    fs::write(runner_dir.join("build.rs"), build_rs)
        .map_err(|error| AotError(format!("failed to write runner build.rs: {error}")))?;

    let runner_source = generate_runner_source(module, module_bin, entry_symbol)?;
    fs::write(runner_dir.join("src/main.rs"), runner_source)
        .map_err(|error| AotError(format!("failed to write runner main.rs: {error}")))?;

    Ok(())
}

fn build_runner_project(runner_dir: &Path) -> Result<(), AotError> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(runner_dir)
        .status()
        .map_err(|error| AotError(format!("failed to execute runner cargo build: {error}")))?;
    if !status.success() {
        return Err(AotError("runner cargo build failed".to_string()));
    }
    Ok(())
}

fn generate_runner_source(
    module: &CompiledModule,
    module_bin: &Path,
    entry_symbol: &str,
) -> Result<String, AotError> {
    let native_functions = module
        .functions
        .values()
        .filter(|function| function.selected_backend == BackendKind::Native)
        .collect::<Vec<_>>();

    let runtime_bridges = collect_runtime_bridges(module)?;

    let mut externs = String::new();
    let mut wrappers = String::new();
    let mut registrations = String::new();
    let mut bridges = String::new();

    for function in &native_functions {
        externs.push_str(&generate_extern_decl(module, function)?);
        wrappers.push_str(&generate_native_wrapper(module, function)?);
        registrations.push_str(&format!(
            "    vm.register_native(\"{}\", wrap_{});\n",
            function.name,
            mangle_ident(&function.name)
        ));
    }

    for bridge in runtime_bridges {
        bridges.push_str(&generate_bridge_function(module, &bridge)?);
    }

    // Get the module file name (not the full path, since it will be in the same directory as the binary)
    let module_filename = module_bin
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AotError("invalid module bin path".to_string()))?;

    let source = format!(
        "use std::ffi::c_void;\nuse std::fs;\n\nuse toolchain::compiler::CompiledModule;\nuse toolchain::runtime::{{Value, vm::{{Vm, RuntimeError}}}};\n\n#[repr(C)]\nstruct NativeRuntimeContext {{\n    vm: *mut Vm,\n    module: *const CompiledModule,\n}}\n\n{externs}\n{wrappers}\n{bridges}\nfn register_native_functions(vm: &mut Vm) {{\n{registrations}}}\n\nfn main() {{\n    let exe_dir = std::env::current_exe()\n        .ok()\n        .and_then(|p| p.parent().map(|p| p.to_path_buf()))\n        .expect(\"could not determine executable directory\");\n    let module_path = exe_dir.join(\"{module_bin}\");\n    let module_bytes = fs::read(&module_path).expect(\"failed to read compiled module\");\n    let module: CompiledModule = bincode::deserialize(&module_bytes).expect(\"module should deserialize\");\n    let mut vm = Vm::default();\n    register_native_functions(&mut vm);\n    match vm.run_entry(&module, \"{entry}\") {{\n        Ok(_) => {{\n            for line in vm.output() {{\n                println!(\"{{}}\", line);\n            }}\n        }}\n        Err(error) => {{\n            eprintln!(\"Runtime Error:\\n{{}}\", error);\n            std::process::exit(1);\n        }}\n    }}\n}}\n",
        externs = externs,
        wrappers = wrappers,
        bridges = bridges,
        registrations = registrations,
        module_bin = module_filename,
        entry = entry_symbol,
    );

    Ok(source)
}

fn generate_extern_decl(
    module: &CompiledModule,
    function: &CompiledFunction,
) -> Result<String, AotError> {
    let params = function
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| {
            rust_abi_type_name(module, *type_id).map(|name| format!("arg{index}: {name}"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    let params = if params.is_empty() {
        "ctx: *mut c_void".to_string()
    } else {
        format!("ctx: *mut c_void, {params}")
    };

    let return_type = rust_abi_type_name(module, function.signature.return_type)?;
    let ret = if return_type == "()" {
        "".to_string()
    } else {
        format!(" -> {return_type}")
    };

    Ok(format!(
        "unsafe extern \"C\" {{\n    fn {}({}){};\n}}\n\n",
        function
            .artifacts
            .aot
            .as_ref()
            .map(|artifact| artifact.symbol.as_str())
            .unwrap_or("missing_symbol"),
        params,
        ret
    ))
}

fn generate_native_wrapper(
    module: &CompiledModule,
    function: &CompiledFunction,
) -> Result<String, AotError> {
    let wrapper_name = format!("wrap_{}", mangle_ident(&function.name));
    let arg_extracts = function
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| generate_value_extract(module, index, *type_id))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    let native_args = (0..function.signature.params.len())
        .map(|index| format!("arg{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let call_args = if native_args.is_empty() {
        "&mut ctx as *mut _ as *mut c_void".to_string()
    } else {
        format!("&mut ctx as *mut _ as *mut c_void, {native_args}")
    };
    let call = if rust_abi_type_name(module, function.signature.return_type)? == "()" {
        format!(
            "    unsafe {{ {}({}); }}\n    Ok(Value::Unit)",
            function.artifacts.aot.as_ref().unwrap().symbol,
            call_args
        )
    } else {
        format!(
            "    let result = unsafe {{ {}({}) }};\n    Ok({})",
            function.artifacts.aot.as_ref().unwrap().symbol,
            call_args,
            wrap_rust_result(module, "result", function.signature.return_type)?
        )
    };

    Ok(format!(
        "fn {wrapper_name}(vm: &mut Vm, module: &CompiledModule, args: Vec<Value>) -> Result<Value, RuntimeError> {{\n    if args.len() != {argc} {{\n        return Err(RuntimeError(format!(\"native function `{name}` expects {argc} arguments but got {{}}\", args.len())));\n    }}\n{arg_extracts}\n    let mut ctx = NativeRuntimeContext {{ vm, module }};\n{call}\n}}\n\n",
        wrapper_name = wrapper_name,
        argc = function.signature.params.len(),
        name = function.name,
        arg_extracts = indent(&arg_extracts, 4),
        call = indent(&call, 4),
    ))
}

fn generate_bridge_function(
    module: &CompiledModule,
    bridge: &BridgeSpec,
) -> Result<String, AotError> {
    let args = bridge
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| {
            rust_abi_type_name(module, *type_id).map(|name| format!("arg{index}: {name}"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    let params = if args.is_empty() {
        "ctx: *mut c_void".to_string()
    } else {
        format!("ctx: *mut c_void, {args}")
    };
    let ret = rust_abi_type_name(module, bridge.signature.return_type)?;
    let ret_decl = if ret == "()" {
        "".to_string()
    } else {
        format!(" -> {ret}")
    };

    let arg_values = bridge
        .signature
        .params
        .iter()
        .enumerate()
        .map(|(index, type_id)| wrap_arg_as_value(module, &format!("arg{index}"), *type_id))
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");

    let unwrap = if ret == "()" {
        "    match vm.run_function(module, CALLEE, args) {\n        Ok(Value::Unit) | Ok(_) => {}\n        Err(error) => panic!(\"bridge call failed: {}\", error),\n    }\n".to_string()
    } else {
        format!(
            "    match vm.run_function(module, CALLEE, args) {{\n        Ok(result) => {},\n        Err(error) => panic!(\"bridge call failed: {{}}\", error),\n    }}\n",
            unwrap_value_result(module, "result", bridge.signature.return_type)?
        )
    };

    Ok(format!(
        "#[no_mangle]\npub extern \"C\" fn {symbol}({params}){ret_decl} {{\n    const CALLEE: &str = \"{callee}\";\n    let ctx = unsafe {{ &mut *(ctx as *mut NativeRuntimeContext) }};\n    let vm = unsafe {{ &mut *ctx.vm }};\n    let module = unsafe {{ &*ctx.module }};\n    let args = vec![{arg_values}];\n{unwrap}}}\n\n",
        symbol = bridge.symbol,
        params = params,
        ret_decl = ret_decl,
        callee = bridge.callee,
        arg_values = arg_values,
        unwrap = unwrap
    ))
}

fn collect_runtime_bridges(module: &CompiledModule) -> Result<Vec<BridgeSpec>, AotError> {
    let mut bridges = Vec::new();

    for function in module.functions.values() {
        if function.selected_backend != BackendKind::Native {
            continue;
        }
        let Some(chunk) = function.artifacts.bytecode.as_ref() else {
            continue;
        };
        for instruction in &chunk.instructions {
            let Instruction::Call {
                function: callee, ..
            } = instruction
            else {
                continue;
            };
            if matches!(
                module.functions.get(callee),
                Some(target) if target.selected_backend == BackendKind::Native
            ) {
                continue;
            }
            let signature = module
                .builtins
                .get(callee)
                .map(|builtin| builtin.signature.clone())
                .or_else(|| {
                    module
                        .functions
                        .get(callee)
                        .map(|function| function.signature.clone())
                })
                .ok_or_else(|| {
                    AotError(format!("missing signature for bridge callee `{callee}`"))
                })?;
            let symbol = format!("kira_bridge_{}", mangle_ident(callee));
            if bridges
                .iter()
                .any(|bridge: &BridgeSpec| bridge.symbol == symbol)
            {
                continue;
            }
            bridges.push(BridgeSpec {
                callee: callee.clone(),
                symbol,
                signature,
            });
        }
    }

    Ok(bridges)
}

#[derive(Clone)]
struct BridgeSpec {
    callee: String,
    symbol: String,
    signature: FunctionSignature,
}

fn mangle_ident(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn rust_abi_type_name(module: &CompiledModule, type_id: TypeId) -> Result<&'static str, AotError> {
    match module.types.get(type_id) {
        KiraType::Unit => Ok("()"),
        KiraType::Bool => Ok("bool"),
        KiraType::Int => Ok("i64"),
        KiraType::Float => Ok("f64"),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            Ok("*mut c_void")
        }
        other => Err(AotError(format!("runner ABI does not yet support type {:?}", other))),
    }
}

fn indent(source: &str, width: usize) -> String {
    let prefix = " ".repeat(width);
    source
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_value_extract(
    module: &CompiledModule,
    index: usize,
    type_id: TypeId,
) -> Result<String, AotError> {
    let source = match module.types.get(type_id) {
        KiraType::Bool => format!(
            "let arg{index} = match &args[{index}] {{\n    Value::Bool(value) => *value,\n    other => return Err(RuntimeError(format!(\"expected bool argument {index}, got {{:?}}\", other))),\n}};"
        ),
        KiraType::Int => format!(
            "let arg{index} = match &args[{index}] {{\n    Value::Int(value) => *value,\n    other => return Err(RuntimeError(format!(\"expected int argument {index}, got {{:?}}\", other))),\n}};"
        ),
        KiraType::Float => format!(
            "let arg{index} = match &args[{index}] {{\n    Value::Float(value) => value.0,\n    other => return Err(RuntimeError(format!(\"expected float argument {index}, got {{:?}}\", other))),\n}};"
        ),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => format!(
            "let arg{index} = Box::into_raw(Box::new(args[{index}].clone())) as *mut c_void;"
        ),
        other => {
            return Err(AotError(format!(
                "runner argument extraction does not yet support {:?}",
                other
            )))
        }
    };
    Ok(source)
}

fn wrap_rust_result(
    module: &CompiledModule,
    name: &str,
    type_id: TypeId,
) -> Result<String, AotError> {
    let value = match module.types.get(type_id) {
        KiraType::Bool => format!("Value::Bool({name})"),
        KiraType::Int => format!("Value::Int({name})"),
        KiraType::Float => format!("Value::Float(ordered_float::OrderedFloat({name}))"),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            format!("unsafe {{ *Box::from_raw({name} as *mut Value) }}")
        }
        other => {
            return Err(AotError(format!(
                "runner result wrapping does not yet support {:?}",
                other
            )))
        }
    };
    Ok(value)
}

fn wrap_arg_as_value(
    module: &CompiledModule,
    name: &str,
    type_id: TypeId,
) -> Result<String, AotError> {
    let value = match module.types.get(type_id) {
        KiraType::Bool => format!("Value::Bool({name})"),
        KiraType::Int => format!("Value::Int({name})"),
        KiraType::Float => format!("Value::Float(ordered_float::OrderedFloat({name}))"),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            format!("unsafe {{ *Box::from_raw({name} as *mut Value) }}")
        }
        other => {
            return Err(AotError(format!(
                "runner bridge arguments do not yet support {:?}",
                other
            )))
        }
    };
    Ok(value)
}

fn unwrap_value_result(
    module: &CompiledModule,
    name: &str,
    type_id: TypeId,
) -> Result<String, AotError> {
    let value = match module.types.get(type_id) {
        KiraType::Bool => format!(
            "match {name} {{ Value::Bool(value) => value, other => panic!(\"expected bool return value, got {{:?}}\", other) }}"
        ),
        KiraType::Int => format!(
            "match {name} {{ Value::Int(value) => value, other => panic!(\"expected int return value, got {{:?}}\", other) }}"
        ),
        KiraType::Float => format!(
            "match {name} {{ Value::Float(value) => value.0, other => panic!(\"expected float return value, got {{:?}}\", other) }}"
        ),
        KiraType::String | KiraType::Array(_) | KiraType::Struct(_) | KiraType::Dynamic => {
            format!("Box::into_raw(Box::new({name})) as *mut c_void")
        }
        other => return Err(AotError(format!("runner bridge return does not yet support {:?}", other))),
    };
    Ok(value)
}

struct NativeCodegen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    context: &'ctx Context,
    target_machine: TargetMachine,
    compiled: &'ctx CompiledModule,
    function_values: HashMap<String, FunctionValue<'ctx>>,
    bridge_values: HashMap<String, FunctionValue<'ctx>>,
}

impl<'ctx> NativeCodegen<'ctx> {
    fn new(
        project_name: &str,
        compiled: &'ctx CompiledModule,
        context: &'ctx Context,
    ) -> Result<Self, AotError> {
        Target::initialize_all(&InitializationConfig::default());
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

    fn write_object(&self, object_path: &Path) -> Result<(), AotError> {
        // First try to write LLVM IR to check if module is valid
        let ir_path = object_path.with_extension("ll");
        self.module
            .print_to_file(&ir_path)
            .map_err(|error| AotError(format!("failed to emit LLVM IR: {error}")))?;

        // Now try to write the object file
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

        // Position at the entry block and build all allocas there
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

        // Now store parameters
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

        // Branch to first instruction block
        self.builder
            .build_unconditional_branch(blocks[0])
            .map_err(|error| AotError(error.to_string()))?;

        // Emit all instruction blocks
        for (index, block) in blocks.iter().enumerate() {
            self.builder.position_at_end(*block);
            let state = &stack_layout.states[index];
            self.emit_instruction(
                function_value,
                ctx_arg,
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
        function: FunctionValue<'ctx>,
        signature: &FunctionSignature,
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

        // Silence unused warning in case signature isn't referenced later through locals.
        let _ = signature;
        let _ = function;
        Ok(locals)
    }

    fn build_stack_allocas_in_place(
        &mut self,
        _function: FunctionValue<'ctx>,
        stack_slot_types: &[TypeId],
    ) -> Result<Vec<PointerValue<'ctx>>, AotError> {
        let mut slots = Vec::with_capacity(stack_slot_types.len());
        for index in 0..stack_slot_types.len() {
            let alloca = self
                .builder
                .build_alloca(self.context.i64_type(), &format!("stack_{index}"))
                .map_err(|error| AotError(error.to_string()))?;
            slots.push(alloca);
        }
        Ok(slots)
    }

    fn emit_instruction(
        &mut self,
        function_value: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        chunk: &Chunk,
        index: usize,
        state: &StackState,
        locals: &[PointerValue<'ctx>],
        stack_slots: &[PointerValue<'ctx>],
        stack_slot_types: &[TypeId],
        blocks: &[BasicBlock<'ctx>],
    ) -> Result<(), AotError> {
        let instruction = &chunk.instructions[index];
        match instruction {
            Instruction::LoadConst(const_index) => {
                let value = chunk
                    .constants
                    .get(*const_index)
                    .ok_or_else(|| AotError(format!("invalid constant index {const_index}")))?;
                let llvm_value = match value {
                    Value::String(text) => {
                        let bytes = self
                            .builder
                            .build_global_string_ptr(text, &format!("str_{const_index}"))
                            .map_err(|error| AotError(error.to_string()))?;
                        let helper = self.helper_function(
                            "kira_native_make_string",
                            &[self.value_handle_type(), self.context.i64_type().into()],
                            Some(self.value_handle_type()),
                        );
                        let len = self
                            .context
                            .i64_type()
                            .const_int(text.len() as u64, false)
                            .as_basic_value_enum();
                        let call = self
                            .builder
                            .build_call(
                                helper,
                                &[bytes.as_pointer_value().as_basic_value_enum().into(), len.into()],
                                "string_const",
                            )
                            .map_err(|error| AotError(error.to_string()))?;
                        call.try_as_basic_value()
                            .left()
                            .expect("string helper should return value")
                    }
                    _ => self.const_value(value)?,
                };
                self.store_stack_value(stack_slots, state.depth(), llvm_value)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::LoadLocal(local_index) => {
                let ptr = *locals
                    .get(*local_index)
                    .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?;
                let local_type = self
                    .llvm_basic_type(chunk.local_types[*local_index])
                    .ok_or_else(|| {
                        AotError(format!("unsupported local type for slot {}", local_index))
                    })?;
                let value = self
                    .builder
                    .build_load(local_type, ptr, &format!("load_local_{local_index}"))
                    .map_err(|error| AotError(error.to_string()))?;
                let value = if self.is_handle_type(chunk.local_types[*local_index]) {
                    self.clone_handle(value)?
                } else {
                    value
                };
                self.store_stack_value(stack_slots, state.depth(), value)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::StoreLocal(local_index) => {
                let value =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                self.builder
                    .build_store(
                        *locals.get(*local_index).ok_or_else(|| {
                            AotError(format!("invalid local index {local_index}"))
                        })?,
                        value,
                    )
                    .map_err(|error| AotError(error.to_string()))?;
                self.branch_next(blocks, index)?;
            }
            Instruction::Negate => {
                let operand_type = *state
                    .stack
                    .last()
                    .ok_or_else(|| AotError("stack underflow for negate".to_string()))?;
                let operand =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                let result = match self.compiled.types.get(operand_type) {
                    KiraType::Int => self
                        .builder
                        .build_int_neg(operand.into_int_value(), "neg_int")
                        .map_err(|error| AotError(error.to_string()))?
                        .as_basic_value_enum(),
                    KiraType::Float => self
                        .builder
                        .build_float_neg(operand.into_float_value(), "neg_float")
                        .map_err(|error| AotError(error.to_string()))?
                        .as_basic_value_enum(),
                    other => {
                        return Err(AotError(format!("negation not supported for {:?}", other)))
                    }
                };
                self.store_stack_value(stack_slots, state.depth().saturating_sub(1), result)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::CastIntToFloat => {
                let operand =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                let result = self
                    .builder
                    .build_signed_int_to_float(
                        operand.into_int_value(),
                        self.context.f64_type(),
                        "int_to_float",
                    )
                    .map_err(|error| AotError(error.to_string()))?;
                self.store_stack_value(
                    stack_slots,
                    state.depth().saturating_sub(1),
                    result.as_basic_value_enum(),
                )?;
                self.branch_next(blocks, index)?;
            }
            Instruction::Add
            | Instruction::Subtract
            | Instruction::Multiply
            | Instruction::Divide
            | Instruction::Modulo
            | Instruction::Less
            | Instruction::Greater
            | Instruction::Equal
            | Instruction::NotEqual
            | Instruction::LessEqual
            | Instruction::GreaterEqual => {
                self.emit_binary_instruction(instruction, state, stack_slots, stack_slot_types)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::JumpIfFalse(target) => {
                let cond =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                let condition = self.bool_condition(cond);
                let then_block = *blocks
                    .get(index + 1)
                    .ok_or_else(|| AotError(format!("missing fallthrough block after {index}")))?;
                let else_block = *blocks
                    .get(*target)
                    .ok_or_else(|| AotError(format!("invalid jump target {target}")))?;
                self.builder
                    .build_conditional_branch(condition, then_block, else_block)
                    .map_err(|error| AotError(error.to_string()))?;
            }
            Instruction::Jump(target) => {
                let target_block = *blocks
                    .get(*target)
                    .ok_or_else(|| AotError(format!("invalid jump target {target}")))?;
                self.builder
                    .build_unconditional_branch(target_block)
                    .map_err(|error| AotError(error.to_string()))?;
            }
            Instruction::Call {
                function,
                arg_count,
            } => {
                let result_type = self.emit_call(
                    function_value,
                    ctx_arg,
                    function,
                    *arg_count,
                    state,
                    stack_slots,
                    stack_slot_types,
                )?;
                if let Some(value) = result_type {
                    self.store_stack_value(
                        stack_slots,
                        state.depth().saturating_sub(*arg_count),
                        value,
                    )?;
                }
                self.branch_next(blocks, index)?;
            }
            Instruction::Pop => {
                self.branch_next(blocks, index)?;
            }
            Instruction::Return => {
                if state.stack.is_empty() {
                    self.builder
                        .build_return(None)
                        .map_err(|error| AotError(error.to_string()))?;
                } else {
                    let value = self.load_stack_value_from_state(
                        stack_slots,
                        state,
                        state.depth().saturating_sub(1),
                    )?;
                    self.builder
                        .build_return(Some(&value))
                        .map_err(|error| AotError(error.to_string()))?;
                }
            }
            Instruction::BuildArray { element_count, .. } => {
                let handle_type = self.value_handle_type();
                let new_array = self.helper_function("kira_native_new_array", &[], Some(handle_type));
                let array = self
                    .builder
                    .build_call(new_array, &[], "array_new")
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("array helper should return value");
                let array_push =
                    self.helper_function("kira_native_array_push", &[handle_type, handle_type], None);
                for value_index in state.depth().saturating_sub(*element_count)..state.depth() {
                    let value_type = state.stack[value_index];
                    let value = self.load_stack_value_from_state(stack_slots, state, value_index)?;
                    let handle = self.box_value_handle(value, value_type)?;
                    self.builder
                        .build_call(array_push, &[array.into(), handle.into()], "array_push")
                        .map_err(|error| AotError(error.to_string()))?;
                }
                self.store_stack_value(
                    stack_slots,
                    state.depth().saturating_sub(*element_count),
                    array,
                )?;
                self.branch_next(blocks, index)?;
            }
            Instruction::ArrayLength => {
                let array =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                let array_length = self.helper_function(
                    "kira_native_array_length",
                    &[self.value_handle_type()],
                    Some(self.context.i64_type().into()),
                );
                let length = self
                    .builder
                    .build_call(array_length, &[array.into()], "array_length")
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("array length should return value");
                self.store_stack_value(stack_slots, state.depth().saturating_sub(1), length)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::ArrayIndex => {
                let right_index = state.depth().saturating_sub(1);
                let left_index = state.depth().saturating_sub(2);
                let target_type = state
                    .stack
                    .get(left_index)
                    .copied()
                    .ok_or_else(|| AotError("array index stack underflow".to_string()))?;
                let KiraType::Array(element_type) = self.compiled.types.get(target_type) else {
                    return Err(AotError("array index expected array target".to_string()));
                };
                let target = self.load_stack_value_from_state(stack_slots, state, left_index)?;
                let index_value = self.load_stack_value_from_state(stack_slots, state, right_index)?;
                let array_index = self.helper_function(
                    "kira_native_array_index",
                    &[self.value_handle_type(), self.context.i64_type().into()],
                    Some(self.value_handle_type()),
                );
                let handle = self
                    .builder
                    .build_call(array_index, &[target.into(), index_value.into()], "array_index")
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("array index should return value");
                let result = self.unbox_value_handle(handle, *element_type)?;
                self.store_stack_value(stack_slots, left_index, result)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::ArrayAppendLocal(local_index) => {
                let value_type = state
                    .stack
                    .last()
                    .copied()
                    .ok_or_else(|| AotError("array append stack underflow".to_string()))?;
                let value =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                let handle = self.box_value_handle(value, value_type)?;
                let array = self
                    .builder
                    .build_load(
                        self.value_handle_type(),
                        *locals
                            .get(*local_index)
                            .ok_or_else(|| AotError(format!("invalid local index {local_index}")))?,
                        "array_local",
                    )
                    .map_err(|error| AotError(error.to_string()))?;
                let array_append = self.helper_function(
                    "kira_native_array_append",
                    &[self.value_handle_type(), self.value_handle_type()],
                    None,
                );
                self.builder
                    .build_call(array_append, &[array.into(), handle.into()], "array_append")
                    .map_err(|error| AotError(error.to_string()))?;
                self.branch_next(blocks, index)?;
            }
            Instruction::BuildStruct { type_id, field_count } => {
                let handle_type = self.value_handle_type();
                let new_struct = self.helper_function(
                    "kira_native_new_struct",
                    &[self.value_handle_type(), self.context.i64_type().into()],
                    Some(handle_type),
                );
                let struct_value = self
                    .builder
                    .build_call(
                        new_struct,
                        &[
                            ctx_arg.as_basic_value_enum().into(),
                            self.context.i64_type().const_int(type_id.0 as u64, false).into(),
                        ],
                        "struct_new",
                    )
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("struct helper should return value");
                let struct_set_field = self.helper_function(
                    "kira_native_struct_set_field",
                    &[handle_type, self.context.i64_type().into(), handle_type],
                    None,
                );
                for (field_offset, value_index) in
                    (state.depth().saturating_sub(*field_count)..state.depth()).enumerate()
                {
                    let value_type = state.stack[value_index];
                    let value = self.load_stack_value_from_state(stack_slots, state, value_index)?;
                    let handle = self.box_value_handle(value, value_type)?;
                    self.builder
                        .build_call(
                            struct_set_field,
                            &[
                                struct_value.into(),
                                self.context
                                    .i64_type()
                                    .const_int(field_offset as u64, false)
                                    .into(),
                                handle.into(),
                            ],
                            "struct_set",
                        )
                        .map_err(|error| AotError(error.to_string()))?;
                }
                self.store_stack_value(
                    stack_slots,
                    state.depth().saturating_sub(*field_count),
                    struct_value,
                )?;
                self.branch_next(blocks, index)?;
            }
            Instruction::StructField(field_index) => {
                let target_index = state.depth().saturating_sub(1);
                let target_type = state
                    .stack
                    .get(target_index)
                    .copied()
                    .ok_or_else(|| AotError("struct field stack underflow".to_string()))?;
                let field_type = self
                    .compiled
                    .types
                    .struct_fields(target_type)
                    .and_then(|fields| fields.get(*field_index))
                    .map(|field| field.type_id)
                    .ok_or_else(|| AotError(format!("invalid struct field index {}", field_index)))?;
                let target = self.load_stack_value_from_state(stack_slots, state, target_index)?;
                let struct_field = self.helper_function(
                    "kira_native_struct_field",
                    &[self.value_handle_type(), self.context.i64_type().into()],
                    Some(self.value_handle_type()),
                );
                let handle = self
                    .builder
                    .build_call(
                        struct_field,
                        &[
                            target.into(),
                            self.context.i64_type().const_int(*field_index as u64, false).into(),
                        ],
                        "struct_field",
                    )
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("struct field should return value");
                let result = self.unbox_value_handle(handle, field_type)?;
                self.store_stack_value(stack_slots, target_index, result)?;
                self.branch_next(blocks, index)?;
            }
            Instruction::StoreLocalField { local, path } => {
                let value_type = state
                    .stack
                    .last()
                    .copied()
                    .ok_or_else(|| AotError("store struct field stack underflow".to_string()))?;
                let value =
                    self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
                let handle = self.box_value_handle(value, value_type)?;
                let path_values = path
                    .iter()
                    .map(|index| self.context.i64_type().const_int(*index as u64, false))
                    .collect::<Vec<_>>();
                let path_array = self
                    .builder
                    .build_alloca(
                        self.context.i64_type().array_type(path_values.len() as u32),
                        "field_path",
                    )
                    .map_err(|error| AotError(error.to_string()))?;
                for (index, value) in path_values.iter().enumerate() {
                    let field_ptr = unsafe {
                        self.builder
                            .build_gep(
                                self.context.i64_type().array_type(path_values.len() as u32),
                                path_array,
                                &[
                                    self.context.i64_type().const_int(0, false),
                                    self.context.i64_type().const_int(index as u64, false),
                                ],
                                "field_path_index",
                            )
                            .map_err(|error| AotError(error.to_string()))?
                    };
                    self.builder
                        .build_store(field_ptr, *value)
                        .map_err(|error| AotError(error.to_string()))?;
                }
                let target = self
                    .builder
                    .build_load(
                        self.value_handle_type(),
                        *locals
                            .get(*local)
                            .ok_or_else(|| AotError(format!("invalid local index {local}")))?,
                        "struct_local",
                    )
                    .map_err(|error| AotError(error.to_string()))?;
                let path_ptr = unsafe {
                    self.builder
                        .build_gep(
                            self.context.i64_type().array_type(path_values.len() as u32),
                            path_array,
                            &[
                                self.context.i64_type().const_int(0, false),
                                self.context.i64_type().const_int(0, false),
                            ],
                            "field_path_ptr",
                        )
                        .map_err(|error| AotError(error.to_string()))?
                };
                let store_struct_field = self.helper_function(
                    "kira_native_store_struct_field",
                    &[
                        self.value_handle_type(),
                        self.context.i64_type().ptr_type(AddressSpace::default()).into(),
                        self.context.i64_type().into(),
                        self.value_handle_type(),
                    ],
                    None,
                );
                self.builder
                    .build_call(
                        store_struct_field,
                        &[
                            target.into(),
                            path_ptr.as_basic_value_enum().into(),
                            self.context
                                .i64_type()
                                .const_int(path.len() as u64, false)
                                .into(),
                            handle.into(),
                        ],
                        "store_struct_field",
                    )
                    .map_err(|error| AotError(error.to_string()))?;
                self.branch_next(blocks, index)?;
            }
        }

        let _ = function_value;
        Ok(())
    }

    fn emit_binary_instruction(
        &mut self,
        instruction: &Instruction,
        state: &StackState,
        stack_slots: &[PointerValue<'ctx>],
        _stack_slot_types: &[TypeId],
    ) -> Result<(), AotError> {
        let right_index = state.depth().saturating_sub(1);
        let left_index = state.depth().saturating_sub(2);
        let left_type = state
            .stack
            .get(left_index)
            .copied()
            .ok_or_else(|| AotError("binary op stack underflow".to_string()))?;
        let left = self.load_stack_value_from_state(stack_slots, state, left_index)?;
        let right = self.load_stack_value_from_state(stack_slots, state, right_index)?;

        let result = match (instruction, self.compiled.types.get(left_type)) {
            (Instruction::Add, KiraType::Int) => self
                .builder
                .build_int_add(left.into_int_value(), right.into_int_value(), "add_int")
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Add, KiraType::Float) => self
                .builder
                .build_float_add(
                    left.into_float_value(),
                    right.into_float_value(),
                    "add_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Subtract, KiraType::Int) => self
                .builder
                .build_int_sub(left.into_int_value(), right.into_int_value(), "sub_int")
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Subtract, KiraType::Float) => self
                .builder
                .build_float_sub(
                    left.into_float_value(),
                    right.into_float_value(),
                    "sub_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Multiply, KiraType::Int) => self
                .builder
                .build_int_mul(left.into_int_value(), right.into_int_value(), "mul_int")
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Multiply, KiraType::Float) => self
                .builder
                .build_float_mul(
                    left.into_float_value(),
                    right.into_float_value(),
                    "mul_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Divide, KiraType::Int) => self
                .builder
                .build_int_signed_div(left.into_int_value(), right.into_int_value(), "div_int")
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Divide, KiraType::Float) => self
                .builder
                .build_float_div(
                    left.into_float_value(),
                    right.into_float_value(),
                    "div_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Modulo, KiraType::Int) => self
                .builder
                .build_int_signed_rem(left.into_int_value(), right.into_int_value(), "rem_int")
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Less, KiraType::Int) => self
                .builder
                .build_int_compare(
                    IntPredicate::SLT,
                    left.into_int_value(),
                    right.into_int_value(),
                    "lt_int",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Less, KiraType::Float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::OLT,
                    left.into_float_value(),
                    right.into_float_value(),
                    "lt_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Greater, KiraType::Int) => self
                .builder
                .build_int_compare(
                    IntPredicate::SGT,
                    left.into_int_value(),
                    right.into_int_value(),
                    "gt_int",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Greater, KiraType::Float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::OGT,
                    left.into_float_value(),
                    right.into_float_value(),
                    "gt_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Equal, KiraType::Int | KiraType::Bool) => self
                .builder
                .build_int_compare(
                    IntPredicate::EQ,
                    left.into_int_value(),
                    right.into_int_value(),
                    "eq_int",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Equal, KiraType::Float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::OEQ,
                    left.into_float_value(),
                    right.into_float_value(),
                    "eq_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::Equal, KiraType::String | KiraType::Array(_) | KiraType::Struct(_)) => {
                let value_eq = self.helper_function(
                    "kira_native_value_eq",
                    &[self.value_handle_type(), self.value_handle_type()],
                    Some(self.context.bool_type().into()),
                );
                self.builder
                    .build_call(value_eq, &[left.into(), right.into()], "eq_value")
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("value eq should return value")
            }
            (Instruction::NotEqual, KiraType::Int | KiraType::Bool) => self
                .builder
                .build_int_compare(
                    IntPredicate::NE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "ne_int",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::NotEqual, KiraType::Float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::ONE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "ne_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::NotEqual, KiraType::String | KiraType::Array(_) | KiraType::Struct(_)) => {
                let value_eq = self.helper_function(
                    "kira_native_value_eq",
                    &[self.value_handle_type(), self.value_handle_type()],
                    Some(self.context.bool_type().into()),
                );
                let eq = self
                    .builder
                    .build_call(value_eq, &[left.into(), right.into()], "ne_value_eq")
                    .map_err(|error| AotError(error.to_string()))?
                    .try_as_basic_value()
                    .left()
                    .expect("value eq should return value")
                    .into_int_value();
                self.builder
                    .build_not(eq, "ne_value")
                    .map_err(|error| AotError(error.to_string()))?
                    .as_basic_value_enum()
            }
            (Instruction::LessEqual, KiraType::Int) => self
                .builder
                .build_int_compare(
                    IntPredicate::SLE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "le_int",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::LessEqual, KiraType::Float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::OLE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "le_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::GreaterEqual, KiraType::Int) => self
                .builder
                .build_int_compare(
                    IntPredicate::SGE,
                    left.into_int_value(),
                    right.into_int_value(),
                    "ge_int",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (Instruction::GreaterEqual, KiraType::Float) => self
                .builder
                .build_float_compare(
                    FloatPredicate::OGE,
                    left.into_float_value(),
                    right.into_float_value(),
                    "ge_float",
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            (op, ty) => {
                return Err(AotError(format!(
                    "unsupported AOT binary instruction {:?} for {:?}",
                    op, ty
                )))
            }
        };

        self.store_stack_value(stack_slots, left_index, result)?;
        Ok(())
    }

    fn emit_call(
        &mut self,
        _current_function: FunctionValue<'ctx>,
        ctx_arg: PointerValue<'ctx>,
        callee: &str,
        arg_count: usize,
        state: &StackState,
        stack_slots: &[PointerValue<'ctx>],
        _stack_slot_types: &[TypeId],
    ) -> Result<Option<BasicValueEnum<'ctx>>, AotError> {
        if callee == "printIn" {
            let arg_type = *state
                .stack
                .get(state.depth().saturating_sub(1))
                .ok_or_else(|| AotError("printIn stack underflow".to_string()))?;
            let value =
                self.load_stack_value_from_state(stack_slots, state, state.depth().saturating_sub(1))?;
            let (helper, args) = match self.compiled.types.get(arg_type) {
                KiraType::Int => (
                    self.helper_function(
                        "kira_native_print_int",
                        &[self.value_handle_type(), self.context.i64_type().into()],
                        None,
                    ),
                    vec![ctx_arg.as_basic_value_enum().into(), value.into()],
                ),
                KiraType::Bool => (
                    self.helper_function(
                        "kira_native_print_bool",
                        &[self.value_handle_type(), self.context.bool_type().into()],
                        None,
                    ),
                    vec![ctx_arg.as_basic_value_enum().into(), value.into()],
                ),
                KiraType::Float => (
                    self.helper_function(
                        "kira_native_print_float",
                        &[self.value_handle_type(), self.context.f64_type().into()],
                        None,
                    ),
                    vec![ctx_arg.as_basic_value_enum().into(), value.into()],
                ),
                KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => (
                    self.helper_function(
                        "kira_native_print_value",
                        &[self.value_handle_type(), self.value_handle_type()],
                        None,
                    ),
                    vec![ctx_arg.as_basic_value_enum().into(), value.into()],
                ),
                other => {
                    return Err(AotError(format!(
                        "printIn is not yet supported for native type {:?}",
                        other
                    )))
                }
            };
            self.builder
                .build_call(helper, &args, "print_builtin")
                .map_err(|error| AotError(error.to_string()))?;
            return Ok(None);
        }

        let signature = self.signature_of(callee)?;
        let mut args = vec![ctx_arg.as_basic_value_enum().into()];
        for (index, _type_id) in signature.params.iter().enumerate() {
            let stack_index = state.depth().saturating_sub(arg_count) + index;
            let value = self.load_stack_value_from_state(stack_slots, state, stack_index)?;
            let actual_type = state
                .stack
                .get(stack_index)
                .copied()
                .ok_or_else(|| AotError("call stack underflow".to_string()))?;
            let value = if self.is_handle_type(actual_type) {
                self.clone_handle(value)?
            } else {
                value
            };
            args.push(value.into());
        }

        let callee_value = if matches!(
            self.compiled.functions.get(callee),
            Some(function) if function.selected_backend == BackendKind::Native
        ) {
            *self
                .function_values
                .get(callee)
                .ok_or_else(|| AotError(format!("missing native LLVM callee for `{callee}`")))?
        } else {
            self.bridge_declaration(callee, &signature)?
        };

        let call = self
            .builder
            .build_call(
                callee_value,
                &args,
                &format!("call_{}", mangle_ident(callee)),
            )
            .map_err(|error| AotError(error.to_string()))?;

        if self.compiled.types.get(signature.return_type) == &KiraType::Unit {
            Ok(None)
        } else {
            Ok(call.try_as_basic_value().left())
        }
    }

    fn bridge_declaration(
        &mut self,
        callee: &str,
        signature: &FunctionSignature,
    ) -> Result<FunctionValue<'ctx>, AotError> {
        if let Some(value) = self.bridge_values.get(callee).copied() {
            return Ok(value);
        }
        let fn_type = self.llvm_function_type(signature)?;
        let symbol = format!("kira_bridge_{}", mangle_ident(callee));
        let value = self.module.add_function(&symbol, fn_type, None);
        self.bridge_values.insert(callee.to_string(), value);
        Ok(value)
    }

    fn value_handle_type(&self) -> BasicTypeEnum<'ctx> {
        self.context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .into()
    }

    fn is_handle_type(&self, type_id: TypeId) -> bool {
        matches!(
            self.compiled.types.get(type_id),
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_)
        )
    }

    fn helper_function(
        &mut self,
        name: &str,
        params: &[BasicTypeEnum<'ctx>],
        ret: Option<BasicTypeEnum<'ctx>>,
    ) -> FunctionValue<'ctx> {
        if let Some(value) = self.module.get_function(name) {
            return value;
        }

        let fn_type = match ret {
            Some(ret) => ret.fn_type(&params.iter().copied().map(Into::into).collect::<Vec<_>>(), false),
            None => self
                .context
                .void_type()
                .fn_type(&params.iter().copied().map(Into::into).collect::<Vec<_>>(), false),
        };
        self.module.add_function(name, fn_type, None)
    }

    fn clone_handle(&mut self, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, AotError> {
        let handle_type = self.value_handle_type();
        let helper = self.helper_function(
            "kira_native_clone_value",
            &[handle_type],
            Some(handle_type),
        );
        let call = self
            .builder
            .build_call(helper, &[value.into()], "clone_handle")
            .map_err(|error| AotError(error.to_string()))?;
        Ok(call
            .try_as_basic_value()
            .left()
            .expect("clone helper should return value"))
    }

    fn box_value_handle(
        &mut self,
        value: BasicValueEnum<'ctx>,
        type_id: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        if self.is_handle_type(type_id) {
            return Ok(value);
        }

        let handle_type = self.value_handle_type();
        let helper_name = match self.compiled.types.get(type_id) {
            KiraType::Int => "kira_native_box_int",
            KiraType::Bool => "kira_native_box_bool",
            KiraType::Float => "kira_native_box_float",
            other => {
                return Err(AotError(format!(
                    "cannot box unsupported native type {:?}",
                    other
                )))
            }
        };
        let helper = self.helper_function(
            helper_name,
            &[value.get_type()],
            Some(handle_type),
        );
        let call = self
            .builder
            .build_call(helper, &[value.into()], "box_value")
            .map_err(|error| AotError(error.to_string()))?;
        Ok(call
            .try_as_basic_value()
            .left()
            .expect("box helper should return value"))
    }

    fn unbox_value_handle(
        &mut self,
        value: BasicValueEnum<'ctx>,
        type_id: TypeId,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        if self.is_handle_type(type_id) {
            return Ok(value);
        }

        let handle_type = self.value_handle_type();
        let (helper_name, ret_type) = match self.compiled.types.get(type_id) {
            KiraType::Int => ("kira_native_unbox_int", self.context.i64_type().into()),
            KiraType::Bool => ("kira_native_unbox_bool", self.context.bool_type().into()),
            KiraType::Float => ("kira_native_unbox_float", self.context.f64_type().into()),
            other => {
                return Err(AotError(format!(
                    "cannot unbox unsupported native type {:?}",
                    other
                )))
            }
        };
        let helper = self.helper_function(helper_name, &[handle_type], Some(ret_type));
        let call = self
            .builder
            .build_call(helper, &[value.into()], "unbox_value")
            .map_err(|error| AotError(error.to_string()))?;
        Ok(call
            .try_as_basic_value()
            .left()
            .expect("unbox helper should return value"))
    }

    fn signature_of(&self, name: &str) -> Result<FunctionSignature, AotError> {
        self.compiled
            .functions
            .get(name)
            .map(|function| function.signature.clone())
            .or_else(|| {
                self.compiled
                    .builtins
                    .get(name)
                    .map(|builtin| builtin.signature.clone())
            })
            .ok_or_else(|| AotError(format!("missing signature for `{name}`")))
    }

    fn bool_condition(&self, value: BasicValueEnum<'ctx>) -> inkwell::values::IntValue<'ctx> {
        let int_value = value.into_int_value();
        self.builder
            .build_int_compare(
                IntPredicate::NE,
                int_value,
                int_value.get_type().const_zero(),
                "bool_cond",
            )
            .expect("bool compare should build")
    }

    fn const_value(&self, value: &Value) -> Result<BasicValueEnum<'ctx>, AotError> {
        match value {
            Value::Int(value) => Ok(self
                .context
                .i64_type()
                .const_int(*value as u64, true)
                .as_basic_value_enum()),
            Value::Float(value) => Ok(self
                .context
                .f64_type()
                .const_float(value.0)
                .as_basic_value_enum()),
            Value::Bool(value) => Ok(self
                .context
                .bool_type()
                .const_int(u64::from(*value), false)
                .as_basic_value_enum()),
            Value::Unit => Err(AotError(
                "unit constants are not supported in LLVM AOT".to_string(),
            )),
            Value::String(_) | Value::Array(_) | Value::Struct(_) => Err(AotError(
                "LLVM AOT backend does not yet support string, array, or struct constants"
                    .to_string(),
            )),
        }
    }

    fn store_stack_value(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        slot: usize,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), AotError> {
        let raw_value = match value {
            BasicValueEnum::IntValue(value) => {
                let value = if value.get_type().get_bit_width() == 64 {
                    value
                } else {
                    self.builder
                        .build_int_z_extend(value, self.context.i64_type(), "stack_word_int")
                        .map_err(|error| AotError(error.to_string()))?
                };
                value.as_basic_value_enum()
            }
            BasicValueEnum::FloatValue(value) => self
                .builder
                .build_bitcast(value, self.context.i64_type(), "stack_word_float")
                .map_err(|error| AotError(error.to_string()))?,
            BasicValueEnum::PointerValue(value) => self
                .builder
                .build_ptr_to_int(value, self.context.i64_type(), "stack_word_ptr")
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum(),
            other => {
                return Err(AotError(format!(
                    "unsupported stack value kind for slot {}: {:?}",
                    slot, other
                )))
            }
        };
        self.builder
            .build_store(
                *stack_slots
                    .get(slot)
                    .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?,
                raw_value,
            )
            .map_err(|error| AotError(error.to_string()))?;
        Ok(())
    }

    fn load_stack_value_from_state(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        state: &StackState,
        slot: usize,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let expected_type = *state
            .stack
            .get(slot)
            .ok_or_else(|| AotError(format!("missing stack type for slot {}", slot)))?;
        self.load_stack_value(stack_slots, expected_type, slot)
    }

    fn load_stack_value(
        &self,
        stack_slots: &[PointerValue<'ctx>],
        expected_type: TypeId,
        slot: usize,
    ) -> Result<BasicValueEnum<'ctx>, AotError> {
        let ptr = *stack_slots
            .get(slot)
            .ok_or_else(|| AotError(format!("invalid stack slot {slot}")))?;
        let raw = self
            .builder
            .build_load(self.context.i64_type(), ptr, &format!("stack_load_{slot}"))
            .map_err(|error| AotError(error.to_string()))?
            .into_int_value();
        match self.compiled.types.get(expected_type) {
            KiraType::Bool => Ok(self
                .builder
                .build_int_truncate(raw, self.context.bool_type(), &format!("stack_bool_{slot}"))
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum()),
            KiraType::Int => Ok(raw.as_basic_value_enum()),
            KiraType::Float => Ok(self
                .builder
                .build_bitcast(raw, self.context.f64_type(), &format!("stack_float_{slot}"))
                .map_err(|error| AotError(error.to_string()))?),
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => Ok(self
                .builder
                .build_int_to_ptr(
                    raw,
                    self.value_handle_type().into_pointer_type(),
                    &format!("stack_ptr_{slot}"),
                )
                .map_err(|error| AotError(error.to_string()))?
                .as_basic_value_enum()),
            other => Err(AotError(format!(
                "unsupported stack load type for slot {}: {:?}",
                slot, other
            ))),
        }
    }

    fn branch_next(&self, blocks: &[BasicBlock<'ctx>], index: usize) -> Result<(), AotError> {
        if let Some(next) = blocks.get(index + 1).copied() {
            self.builder
                .build_unconditional_branch(next)
                .map_err(|error| AotError(error.to_string()))?;
        } else {
            self.builder
                .build_return(None)
                .map_err(|error| AotError(error.to_string()))?;
        }
        Ok(())
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
            KiraType::String | KiraType::Dynamic | KiraType::Array(_) | KiraType::Struct(_) => {
                Some(self.value_handle_type())
            }
            _ => None,
        }
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
                Value::Int(_) | Value::Float(_) | Value::Bool(_) => {}
                Value::Unit | Value::String(_) => {}
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

struct StackLayout {
    states: Vec<StackState>,
    stack_slot_types: Vec<TypeId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StackState {
    stack: Vec<TypeId>,
}

impl StackState {
    fn depth(&self) -> usize {
        self.stack.len()
    }
}

fn infer_stack_layout(module: &CompiledModule, chunk: &Chunk) -> Result<StackLayout, AotError> {
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
