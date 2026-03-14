use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use inkwell::context::Context;

use crate::compiler::CompiledModule;

use super::codegen::NativeCodegen;
use super::error::AotError;

pub fn build_native_archive(
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
