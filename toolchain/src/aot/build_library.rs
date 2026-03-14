use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashSet, VecDeque};

use crate::compiler::{Instruction, compile_project};
use crate::project::load_project;

use super::error::AotError;
use super::library::{generate_c_header, ExportedApi, CAbiCodegen, ExportSpec};
use super::runner::{link_shared_library, resolve_output_root, shared_lib_extension, write_if_changed};

pub fn build_library_project(project_root: &Path, out_root: &Path) -> Result<PathBuf, AotError> {
    let project = load_project(project_root).map_err(|error| AotError(error.to_string()))?;
    let module =
        compile_project(&project.program, project_root).map_err(|error| AotError(error.to_string()))?;

    let out_root = resolve_output_root(out_root)?;
    fs::create_dir_all(&out_root).map_err(|error| {
        AotError(format!(
            "failed to create output directory `{}`: {}",
            out_root.display(),
            error
        ))
    })?;

    let staging_root = out_root.join(".kira-build").join(&project.manifest.name).join("lib");
    fs::create_dir_all(&staging_root).map_err(|error| {
        AotError(format!(
            "failed to create staging directory `{}`: {}",
            staging_root.display(),
            error
        ))
    })?;

    let (api, export_spec) = collect_export_spec(&project.program, &module)?;

    let object_path = staging_root.join("kira_lib.o");
    if export_spec.closure_functions.is_empty() {
        return Err(AotError("no @Export functions found".to_string()));
    }

    let context = inkwell::context::Context::create();
    let codegen = CAbiCodegen::new(&project.manifest.name, &module, &export_spec, &context)?;
    codegen.write_object(&object_path)?;

    let lib_filename = format!("{}.{}", project.manifest.name, shared_lib_extension());
    let lib_path = out_root.join(lib_filename);
    link_shared_library(&object_path, &lib_path, &module.ffi)?;

    let header_path = out_root.join(format!("{}.h", project.manifest.name));
    let header = generate_c_header(&project.manifest.name, &module, &api)?;
    write_if_changed(&header_path, &header)?;

    Ok(lib_path)
}

fn collect_export_spec(
    program: &crate::ast::Program,
    module: &crate::compiler::CompiledModule,
) -> Result<(ExportedApi, ExportSpec), AotError> {
    let mut exported_structs = HashSet::new();
    let mut exported_functions = HashSet::new();

    for item in &program.items {
        match item {
            crate::ast::TopLevelItem::Struct(definition) => {
                if definition
                    .attributes
                    .iter()
                    .any(|attr| attr.name.name == "Export")
                {
                    exported_structs.insert(definition.name.name.clone());
                }
            }
            crate::ast::TopLevelItem::Function(function) => {
                if function
                    .attributes
                    .iter()
                    .any(|attr| attr.name.name == "Export")
                {
                    exported_functions.insert(function.name.name.clone());
                }
            }
        }
    }

    if exported_functions.is_empty() {
        return Err(AotError("no @Export functions found".to_string()));
    }

    let mut closure = HashSet::new();
    let mut queue = VecDeque::new();
    for name in exported_functions.iter().cloned() {
        closure.insert(name.clone());
        queue.push_back(name);
    }

    while let Some(name) = queue.pop_front() {
        let function = module.functions.get(&name).ok_or_else(|| {
            AotError(format!("missing exported function `{}` in compiled module", name))
        })?;
        let chunk = function.artifacts.bytecode.as_ref().ok_or_else(|| {
            AotError(format!("missing bytecode for `{}`", name))
        })?;

        for instruction in &chunk.instructions {
            let Instruction::Call { function: callee, .. } = instruction else {
                continue;
            };
            if module.functions.contains_key(callee) {
                if closure.insert(callee.clone()) {
                    queue.push_back(callee.clone());
                }
                continue;
            }
            if module.ffi.functions.contains_key(callee) {
                continue;
            }
            if module.builtins.contains_key(callee) {
                return Err(AotError(format!(
                    "`--lib` output does not support builtin calls (found call to `{}`)",
                    callee
                )));
            }
            return Err(AotError(format!("unknown callee `{}` in `{}`", callee, name)));
        }
    }

    Ok((
        ExportedApi {
            structs: exported_structs.iter().cloned().collect(),
            functions: exported_functions.iter().cloned().collect(),
        },
        ExportSpec {
            exported_functions,
            exported_structs,
            closure_functions: closure,
        },
    ))
}
