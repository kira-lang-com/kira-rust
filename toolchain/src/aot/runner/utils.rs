use std::fs;
use std::path::{Path, PathBuf};

use crate::aot::error::AotError;

pub fn resolve_output_root(out_root: &Path) -> Result<PathBuf, AotError> {
    if out_root.is_absolute() {
        Ok(out_root.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(out_root))
            .map_err(|error| {
                AotError(format!(
                    "failed to resolve output root `{}`: {}",
                    out_root.display(),
                    error
                ))
            })
    }
}

pub fn remove_path_if_exists(path: &Path, label: &str) -> Result<(), AotError> {
    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
        .map_err(|error| {
            AotError(format!(
                "failed to remove {label} at `{}`: {}",
                path.display(),
                error
            ))
        })?;
    }
    Ok(())
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<(), AotError> {
    let should_write = if path.exists() {
        fs::read_to_string(path)
            .map(|existing| existing != content)
            .unwrap_or(true)
    } else {
        true
    };

    if should_write {
        fs::write(path, content).map_err(|error| {
            AotError(format!("failed to write `{}`: {}", path.display(), error))
        })?;
    }

    Ok(())
}

pub fn indent(source: &str, width: usize) -> String {
    let prefix = " ".repeat(width);
    source
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn mangle_ident(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}
