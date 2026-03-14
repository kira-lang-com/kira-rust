use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

use crate::compiler::{BackendKind, CompiledModule};

use crate::aot::bridge::{collect_runtime_bridges, generate_bridge_function};
use crate::aot::bridge::{generate_extern_decl, generate_native_wrapper};
use crate::aot::error::AotError;
use super::utils::{indent, mangle_ident, write_if_changed};

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

    let crate_root = resolve_toolchain_crate_root()?;
    let cargo_toml = format!(
        "[package]\nname = \"{name}_runner\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[[bin]]\nname = \"{name}\"\npath = \"src/main.rs\"\n\n[dependencies]\nbincode = \"1.3.3\"\nordered-float = \"5.1.0\"\ntoolchain = {{ path = \"{toolchain}\" }}\n",
        name = project_name,
        toolchain = crate_root.display()
    );
    write_if_changed(&runner_dir.join("Cargo.toml"), &cargo_toml)?;

    let mut build_rs = String::new();
    build_rs.push_str("fn main() {\n");
    
    // Only link native archive if it exists (for native-compiled functions)
    if native_archive.exists() {
        build_rs.push_str(&format!(
            "    println!(\"cargo:rustc-link-search=native={}\");\n",
            native_archive.parent().unwrap().display()
        ));
        build_rs.push_str("    println!(\"cargo:rustc-link-lib=static=kira_native\");\n");
    }

    let mut seen_paths = std::collections::HashSet::new();
    let mut seen_libs = std::collections::HashSet::new();
    let emit_rpath = !cfg!(target_os = "windows");
    for link in &module.ffi.links {
        if seen_libs.insert(link.library.clone()) {
            build_rs.push_str(&format!(
                "    println!(\"cargo:rustc-link-lib=dylib={}\");\n",
                link.library
            ));
        }
        for path in &link.search_paths {
            if seen_paths.insert(path.clone()) {
                build_rs.push_str(&format!(
                    "    println!(\"cargo:rustc-link-search=native={}\");\n",
                    path
                ));
                if emit_rpath {
                    build_rs.push_str(&format!(
                        "    println!(\"cargo:rustc-link-arg=-Wl,-rpath,{}\");\n",
                        path
                    ));
                }
            }
        }
    }
    build_rs.push_str("}\n");
    write_if_changed(&runner_dir.join("build.rs"), &build_rs)?;

    let runner_source = generate_runner_source(module, module_bin, entry_symbol)?;
    write_if_changed(&runner_dir.join("src/main.rs"), &runner_source)?;

    Ok(())
}

fn resolve_toolchain_crate_root() -> Result<PathBuf, AotError> {
    if let Ok(value) = env::var("KIRA_TOOLCHAIN_SRC") {
        let path = PathBuf::from(value);
        if path.join("Cargo.toml").is_file() {
            return Ok(path);
        }
    }

    // Installed layout: `<install_root>/kira` and `<install_root>/toolchain/Cargo.toml`.
    if let Ok(exe) = env::current_exe() {
        let exe = fs::canonicalize(&exe).unwrap_or(exe);
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join("toolchain");
            if candidate.join("Cargo.toml").is_file() {
                return Ok(candidate);
            }
            // Repo layout: `toolchain/target/<profile>/kira`.
            if let Some(toolchain_dir) = exe_dir.parent().and_then(|p| p.parent()) {
                if toolchain_dir.file_name().and_then(|n| n.to_str()) == Some("toolchain")
                    && toolchain_dir.join("Cargo.toml").is_file()
                {
                    return Ok(toolchain_dir.to_path_buf());
                }
            }
        }
    }

    // Dev fallback: compile-time crate root.
    let compile_time = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if compile_time.join("Cargo.toml").is_file() {
        return Ok(compile_time);
    }

    Err(AotError(
        "could not resolve toolchain crate root for runner generation".to_string(),
    ))
}

pub fn build_runner_project(runner_dir: &Path) -> Result<(), AotError> {
    // Use a shared target directory to cache toolchain builds across all projects
    let shared_target = get_shared_target_dir()?;
    
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .arg("--offline")
        .env("CARGO_TARGET_DIR", &shared_target)
        .current_dir(runner_dir);
    let status = cmd
        .status()
        .map_err(|error| AotError(format!("failed to execute runner cargo build: {error}")))?;
    if status.success() {
        return Ok(());
    }

    // Fallback: allow Cargo to fetch missing deps when offline mode can't resolve them.
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .env("CARGO_TARGET_DIR", &shared_target)
        .current_dir(runner_dir)
        .status()
        .map_err(|error| AotError(format!("failed to execute runner cargo build: {error}")))?;
    if !status.success() {
        return Err(AotError("runner cargo build failed".to_string()));
    }
    Ok(())
}

pub fn get_shared_target_dir() -> Result<PathBuf, AotError> {
    // Use a shared target directory in the toolchain installation
    if let Ok(exe) = env::current_exe() {
        let exe = fs::canonicalize(&exe).unwrap_or(exe);
        if let Some(exe_dir) = exe.parent() {
            // For installed toolchains: use <install_root>/.cargo-cache
            let cache_dir = exe_dir.join(".cargo-cache");
            return Ok(cache_dir);
        }
    }
    
    // Fallback: use a directory in the user's home
    if let Some(home) = dirs::home_dir() {
        return Ok(home.join(".kira").join("cargo-cache"));
    }
    
    Err(AotError("could not determine shared target directory".to_string()))
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
