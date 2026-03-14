use crate::library::registry::{LibraryFunctionSpec, LibraryModuleSpec};

pub fn module() -> LibraryModuleSpec {
    LibraryModuleSpec {
        package: "Foundation",
        namespace: "Math",
        functions: vec![
            LibraryFunctionSpec::native("Foundation.Math.sqrt", &["float"], "float"),
            LibraryFunctionSpec::native("Foundation.Math.floor", &["float"], "int"),
            LibraryFunctionSpec::native("Foundation.Math.ceil", &["float"], "int"),
            LibraryFunctionSpec::native("Foundation.Math.round", &["float"], "int"),
            LibraryFunctionSpec::native("Foundation.Math.pi", &[], "float"),
            LibraryFunctionSpec::native("Foundation.Math.clamp", &["int", "int", "int"], "int"),
            LibraryFunctionSpec::native("Foundation.Math.lerp", &["float", "float", "float"], "float"),
            LibraryFunctionSpec::native("Foundation.Math.sign", &["int"], "int"),
        ],
    }
}
