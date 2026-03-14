use crate::library::registry::{LibraryFunctionSpec, LibraryModuleSpec};

pub fn module() -> LibraryModuleSpec {
    LibraryModuleSpec {
        package: "Foundation",
        namespace: "Time",
        functions: vec![
            LibraryFunctionSpec::native("Foundation.Time.now", &[], "int"),
            LibraryFunctionSpec::native("Foundation.Time.delta", &[], "float"),
        ],
    }
}
