pub(crate) mod header;

pub(crate) use header::{CFunction, CType, ParsedHeader, parse_header};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::LinkDirective;
use super::{Chunk, CompileError, FfiFunction, FfiLink, FfiMetadata, FunctionSignature, Instruction};
use crate::runtime::type_system::{KiraType, TypeId, TypeSystem};

pub fn build_ffi_metadata(
    types: &mut TypeSystem,
    links: &[LinkDirective],
    project_root: Option<&Path>,
) -> Result<(FfiMetadata, HashMap<String, FunctionSignature>), CompileError> {
    if links.is_empty() {
        return Ok((FfiMetadata::default(), HashMap::new()));
    }

    let project_root = project_root.ok_or_else(|| {
        CompileError("@Link directives require a project root to resolve header paths".to_string())
    })?;

    let mut metadata = FfiMetadata::default();
    let mut signatures = HashMap::new();

    for link in links {
        let header_path = resolve_header_path(project_root, &link.header)?;
        let header_dir = header_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| project_root.to_path_buf());

        let source = fs::read_to_string(&header_path).map_err(|error| {
            CompileError(format!(
                "failed to read linked header `{}`: {}",
                header_path.display(),
                error
            ))
        })?;

        let parsed = parse_header(&source).map_err(CompileError)?;

        for name in parsed.opaque_typedefs {
            if let Some(existing) = types.resolve_named(&name) {
                if matches!(types.get(existing), KiraType::Opaque(_)) {
                    continue;
                }
                return Err(CompileError(format!(
                    "linked opaque type `{}` conflicts with an existing type",
                    name
                )));
            }
            types.declare_opaque(&name).map_err(CompileError)?;
        }

        // Parse typedef'd structs so we can detect them, but treat them as opaque in Kira for now.
        // This keeps ABI sound: passing structs by value in extern C calls is not supported yet.
        let header_struct_names = parsed
            .structs
            .iter()
            .map(|s| s.name.clone())
            .collect::<std::collections::HashSet<_>>();
        for cstruct in parsed.structs {
            if types.resolve_named(&cstruct.name).is_none() {
                types.declare_opaque(&cstruct.name).map_err(CompileError)?;
            }
        }

        for function in parsed.functions {
            let name = function.name;
            if signatures.contains_key(&name) {
                // Best-effort de-dupe: allow identical declarations.
                continue;
            }

            if uses_struct_by_value(&function.return_type, &header_struct_names)
                || function
                    .params
                    .iter()
                    .any(|param| uses_struct_by_value(&param.ty, &header_struct_names))
            {
                return Err(CompileError(format!(
                    "linked C function `{}` uses a struct by value; pass-by-value structs are not supported yet in @Link bindings",
                    name
                )));
            }

            let return_type = kira_type_from_ctype(types, &function.return_type)?;
            let params = function
                .params
                .iter()
                .map(|param| kira_type_from_ctype(types, &param.ty))
                .collect::<Result<Vec<_>, _>>()?;
            let function_type = types.register_function(params.clone(), return_type);

            let signature = FunctionSignature {
                params,
                return_type,
                function_type,
            };

            signatures.insert(name.clone(), signature.clone());
            metadata.functions.insert(
                name.clone(),
                FfiFunction {
                    symbol: name.clone(),
                    signature,
                },
            );
        }

        let search_paths = vec![header_dir.to_string_lossy().into_owned()];
        metadata.links.push(FfiLink {
            library: link.library.clone(),
            header: link.header.clone(),
            search_paths,
        });
    }

    Ok((metadata, signatures))
}

fn resolve_header_path(project_root: &Path, header: &str) -> Result<PathBuf, CompileError> {
    let candidate = project_root.join(header);
    if candidate.is_file() {
        return Ok(candidate);
    }
    Err(CompileError(format!(
        "linked header `{}` does not exist (looked for `{}`)",
        header,
        candidate.display()
    )))
}

fn kira_type_from_ctype(types: &mut TypeSystem, ty: &CType) -> Result<TypeId, CompileError> {
    Ok(match ty {
        CType::Void => types.unit(),
        CType::Bool => types.bool(),
        CType::Int64 => types.int(),
        CType::Double => types.float(),
        CType::Named(name) => types.ensure_named(name).ok_or_else(|| {
            CompileError(format!("unknown linked type `{}`", name))
        })?,
        CType::Pointer { base, .. } => {
            let opaque_name = opaque_pointer_name(base);
            match types.resolve_named(&opaque_name) {
                Some(id) => id,
                None => types.declare_opaque(&opaque_name).map_err(CompileError)?,
            }
        }
    })
}

fn opaque_pointer_name(base: &CType) -> String {
    match base {
        CType::Void => "void_ptr".to_string(),
        CType::Named(name) => format!("{name}Ptr"),
        CType::Bool => "bool_ptr".to_string(),
        CType::Int64 => "int64_ptr".to_string(),
        CType::Double => "double_ptr".to_string(),
        CType::Pointer { .. } => "ptr_ptr".to_string(),
    }
}

fn uses_struct_by_value(ty: &CType, header_struct_names: &std::collections::HashSet<String>) -> bool {
    match ty {
        CType::Named(name) => header_struct_names.contains(name),
        CType::Pointer { .. } => false,
        _ => false,
    }
}

pub fn chunk_contains_ffi_calls(chunk: &Chunk, ffi: &FfiMetadata) -> bool {
    chunk.instructions.iter().any(|instruction| {
        matches!(
            instruction,
            Instruction::Call { function, .. } if ffi.functions.contains_key(function)
        )
    })
}

pub fn type_is_vm_compatible(types: &TypeSystem, type_id: TypeId) -> bool {
    match types.get(type_id) {
        // VM has no representation for opaque handles today; treat as native-only.
        KiraType::Opaque(_) => false,
        KiraType::Array(element) => type_is_vm_compatible(types, *element),
        KiraType::Struct(_) => types
            .struct_fields(type_id)
            .into_iter()
            .flatten()
            .all(|field| type_is_vm_compatible(types, field.type_id)),
        _ => true,
    }
}
