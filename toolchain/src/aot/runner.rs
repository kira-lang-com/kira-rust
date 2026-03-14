use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::compiler::{BackendKind, CompiledModule};

use super::bridge::{collect_runtime_bridges, generate_bridge_function};
use super::error::AotError;
use super::utils::{indent, mangle_ident, write_if_changed};
use super::wrappers::{generate_extern_decl, generate_native_wrapper};

pub fn write_runner_project(
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
    write_if_changed(&runner_dir.join("Cargo.toml"), &cargo_toml)?;

    let build_rs = format!(
        "fn main() {{\n    println!(\"cargo:rustc-link-search=native={}\");\n    println!(\"cargo:rustc-link-lib=static=kira_native\");\n}}\n",
        native_archive.parent().unwrap().display()
    );
    write_if_changed(&runner_dir.join("build.rs"), &build_rs)?;

    let runner_source = generate_runner_source(module, module_bin, entry_symbol)?;
    write_if_changed(&runner_dir.join("src/main.rs"), &runner_source)?;

    Ok(())
}

pub fn build_runner_project(runner_dir: &Path) -> Result<(), AotError> {
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
