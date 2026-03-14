use std::path::Path;
use std::process::Command;

use crate::compiler::FfiMetadata;

use super::error::AotError;

pub fn shared_lib_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    }
}

pub fn link_shared_library(
    object: &Path,
    output: &Path,
    ffi: &FfiMetadata,
) -> Result<(), AotError> {
    let mut cmd = Command::new("clang");

    if cfg!(target_os = "macos") {
        cmd.arg("-dynamiclib");
    } else {
        cmd.arg("-shared");
    }

    cmd.arg("-o").arg(output).arg(object);

    for link in &ffi.links {
        for path in &link.search_paths {
            cmd.arg(format!("-L{path}"));
        }
        cmd.arg(format!("-l{}", link.library));
    }

    let status = cmd
        .status()
        .map_err(|error| AotError(format!("failed to invoke clang for shared library link: {error}")))?;
    if !status.success() {
        return Err(AotError("clang failed to link shared library".to_string()));
    }
    Ok(())
}

