// Bridge between native and bytecode execution

mod bridge;
mod wrappers;

pub use bridge::{collect_runtime_bridges, generate_bridge_function};
pub use wrappers::{generate_extern_decl, generate_native_wrapper};
