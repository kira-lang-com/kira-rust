// FFI bindings generation

mod link_collector;
mod renderer;

use std::fs;
use std::path::Path;

use crate::compiler::ffi::parse_header;

use super::ProjectError;
use link_collector::{collect_links, resolve_header_path};
use renderer::render_bindings;

pub fn generate_ffi_bindings(root: &Path) -> Result<(), ProjectError> {
    let links = collect_links(root)?;
    if links.is_empty() {
        return Ok(());
    }

    let bindings_dir = root.join("bindings");
    fs::create_dir_all(&bindings_dir).map_err(|error| {
        ProjectError(format!(
            "failed to create bindings directory `{}`: {}",
            bindings_dir.display(),
            error
        ))
    })?;

    for link in links {
        let header_path = resolve_header_path(root, &link.header)?;
        let source = fs::read_to_string(&header_path).map_err(|error| {
            ProjectError(format!(
                "failed to read linked header `{}`: {}",
                header_path.display(),
                error
            ))
        })?;
        let parsed = parse_header(&source).map_err(ProjectError)?;
        let bindings = render_bindings(&link, &parsed);
        let filename = format!("{}.kira", sanitize_filename(&link.library));
        let path = bindings_dir.join(filename);
        write_if_changed(&path, &bindings).map_err(ProjectError)?;
    }

    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() || ch == '_' { ch } else { '_' })
        .collect()
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), String> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == contents {
            return Ok(());
        }
    }
    fs::write(path, contents)
        .map_err(|error| format!("failed to write `{}`: {}", path.display(), error))
}
