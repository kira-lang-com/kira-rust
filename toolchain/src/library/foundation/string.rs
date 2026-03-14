use crate::library::registry::{LibraryFunctionSpec, LibraryModuleSpec};

pub fn module() -> LibraryModuleSpec {
    LibraryModuleSpec {
        package: "Foundation",
        namespace: "String",
        functions: vec![
            LibraryFunctionSpec::native("Foundation.String.length", &["string"], "int"),
            LibraryFunctionSpec::native("Foundation.String.concat", &["string", "string"], "string"),
            LibraryFunctionSpec::native("Foundation.String.contains", &["string", "string"], "bool"),
            LibraryFunctionSpec::native("Foundation.String.uppercase", &["string"], "string"),
            LibraryFunctionSpec::native("Foundation.String.lowercase", &["string"], "string"),
            LibraryFunctionSpec::native("Foundation.String.repeat", &["string", "int"], "string"),
        ],
    }
}
