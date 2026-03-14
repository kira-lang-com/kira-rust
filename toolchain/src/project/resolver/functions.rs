use std::collections::{BTreeSet, HashMap};

use crate::ast::FunctionDefinition;
use crate::library::ImportedNamespace;

use super::super::ProjectError;
use super::blocks::resolve_block;

#[derive(Debug, Clone)]
pub struct FunctionOrigin {
    pub file_name: String,
}

pub fn resolve_function_body(
    function: &mut FunctionDefinition,
    current_file: &str,
    global_functions: &HashMap<String, FunctionOrigin>,
    builtins: &BTreeSet<String>,
    local_modules: &HashMap<String, String>,
    imported_namespaces: &HashMap<String, ImportedNamespace>,
) -> Result<(), ProjectError> {
    let mut scope = function
        .params
        .iter()
        .map(|parameter| parameter.name.name.clone())
        .collect::<BTreeSet<_>>();

    resolve_block(
        &mut function.body,
        current_file,
        global_functions,
        builtins,
        local_modules,
        imported_namespaces,
        &mut scope,
    )
}
