// Bridge between native and bytecode execution

mod bridge;

pub use bridge::{BridgeSpec, collect_runtime_bridges};
