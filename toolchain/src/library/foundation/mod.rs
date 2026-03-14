mod math;
mod random;
mod string;
mod time;

use super::registry::LibraryModuleSpec;

pub fn modules() -> Vec<LibraryModuleSpec> {
    vec![
        math::module(),
        string::module(),
        random::module(),
        time::module(),
    ]
}
