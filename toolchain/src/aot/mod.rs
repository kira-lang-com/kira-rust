mod bridge;
mod build_executable;
mod build_library;
mod error;
mod library;
mod native;
mod runner;
mod stack;
mod types;

pub use build_executable::{build_default_project, run_default_project};
pub use build_library::build_library_project;
pub use error::AotError;
