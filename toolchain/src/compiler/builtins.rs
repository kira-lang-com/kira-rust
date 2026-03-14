use std::collections::HashMap;

use crate::library::all_library_modules;
use crate::runtime::type_system::{TypeId, TypeSystem};

use super::{BackendKind, BuiltinFunction, FunctionSignature};

pub(super) fn builtin_functions(types: &mut TypeSystem) -> HashMap<String, BuiltinFunction> {
    let mut builtins = HashMap::new();
    let dynamic = types.dynamic();
    let unit = types.unit();
    let int = types.int();

    insert_builtin(&mut builtins, types, "printIn", vec![dynamic], unit);
    insert_builtin(&mut builtins, types, "abs", vec![int], int);
    insert_builtin(&mut builtins, types, "pow", vec![int, int], int);
    insert_builtin(&mut builtins, types, "max", vec![int, int], int);
    insert_builtin(&mut builtins, types, "min", vec![int, int], int);

    for module in all_library_modules() {
        for function in module.functions {
            insert_named_builtin(
                &mut builtins,
                types,
                function.full_name,
                function.params,
                function.return_type,
                function.backend,
            );
        }
    }

    builtins
}

fn insert_builtin(
    builtins: &mut HashMap<String, BuiltinFunction>,
    types: &mut TypeSystem,
    name: &str,
    params: Vec<TypeId>,
    return_type: TypeId,
) {
    insert_resolved_builtin(
        builtins,
        types,
        name,
        params,
        return_type,
        BackendKind::Native,
    );
}

fn insert_named_builtin(
    builtins: &mut HashMap<String, BuiltinFunction>,
    types: &mut TypeSystem,
    name: &str,
    param_names: &[&str],
    return_name: &str,
    backend: BackendKind,
) {
    let params = param_names
        .iter()
        .map(|name| {
            types
                .resolve_named(name)
                .unwrap_or_else(|| panic!("unknown built-in type `{name}`"))
        })
        .collect::<Vec<_>>();
    let return_type = types
        .resolve_named(return_name)
        .unwrap_or_else(|| panic!("unknown built-in type `{return_name}`"));

    insert_resolved_builtin(builtins, types, name, params, return_type, backend);
}

fn insert_resolved_builtin(
    builtins: &mut HashMap<String, BuiltinFunction>,
    types: &mut TypeSystem,
    name: &str,
    params: Vec<TypeId>,
    return_type: TypeId,
    backend: BackendKind,
) {
    let function_type = types.register_function(params.clone(), return_type);
    builtins.insert(
        name.to_string(),
        BuiltinFunction {
            name: name.to_string(),
            signature: FunctionSignature {
                params,
                return_type,
                function_type,
            },
            backend,
        },
    );
}
