mod error;
mod ffi_bindings;
mod library;
mod loader;
mod manifest;
mod resolver;

pub use error::ProjectError;
pub use ffi_bindings::generate_ffi_bindings;
pub use library::{package_library, resolve_dependencies, ResolvedDependency};
pub use loader::load_project;
pub use manifest::{load_manifest, Dependency, DependencySource, ProjectKind, ProjectManifest};
