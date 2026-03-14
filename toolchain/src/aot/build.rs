use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::compile;
use crate::project::load_project;

use super::archive::build_native_archive;
use super::error::AotError;
use super::runner::{build_runner_project, write_runner_project};
use super::utils::{remove_path_if_exists, resolve_output_root};

pub fn build_default_project(project_root: &Path, out_root: &Path) -> Result<PathBuf, AotError> {
    let project = load_project(project_root).map_err(|error| AotError(error.to_string()))?;
    let module = compile(&project.program).map_err(|error| AotError(error.to_string()))?;

    let out_root = resolve_output_root(out_root)?;
    remove_path_if_exists(&out_root.join("build"), "legacy build output")?;
    remove_path_if_exists(&out_root.join("compiled_module.bin"), "legacy compiled module")?;

    let staging_root = out_root.join(".kira-build").join(&project.manifest.name);
    fs::create_dir_all(&staging_root).map_err(|error| {
        AotError(format!(
            "failed to create staging directory `{}`: {}",
            staging_root.display(),
            error
        ))
    })?;

    let final_bundle_dir = out_root.join(&project.manifest.name);
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
            "failed to copy module from `{}` to `{}`: {}",
            module_bin.display(),
            final_module.display(),
            error
        ))
    })?;

    Ok(final_binary)
}

pub fn run_default_project(project_root: &Path, out_root: &Path) -> Result<i32, AotError> {
    let binary = build_default_project(project_root, out_root)?;
    let status = std::process::Command::new(&binary)
        .status()
        .map_err(|error| AotError(format!("failed to run `{}`: {}", binary.display(), error)))?;

    Ok(status.code().unwrap_or(-1))
}
