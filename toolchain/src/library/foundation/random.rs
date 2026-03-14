use crate::library::registry::{LibraryFunctionSpec, LibraryModuleSpec};

pub fn module() -> LibraryModuleSpec {
    LibraryModuleSpec {
        package: "Foundation",
        namespace: "Random",
        functions: vec![
            LibraryFunctionSpec::native("Foundation.Random.int", &["int", "int"], "int"),
            LibraryFunctionSpec::native("Foundation.Random.float", &["float", "float"], "float"),
            LibraryFunctionSpec::native("Foundation.Random.bool", &[], "bool"),
        ],
    }
}
