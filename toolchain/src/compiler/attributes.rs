use crate::ast::syntax::{ExecutionMode, FunctionDefinition};

use super::{platforms::PlatformModel, CompileError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedFunctionAttributes {
    pub declared_mode: ExecutionMode,
    pub target_platforms: Vec<String>,
}

fn host_platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        // Treat all other supported hosts as Linux-like for now.
        "linux"
    }
}

pub(super) fn resolve_function_attributes(
    function: &FunctionDefinition,
    platforms: Option<&PlatformModel>,
) -> Result<ResolvedFunctionAttributes, CompileError> {
    let mut declared_mode = function.execution_hint.unwrap_or(ExecutionMode::Auto);
    let mut mode_source = function.execution_hint.map(|_| "execution hint");
    let mut platform_selectors = Vec::new();

    for attribute in &function.attributes {
        match attribute.name.name.as_str() {
            "Native" => {
                assign_mode(
                    &mut declared_mode,
                    &mut mode_source,
                    ExecutionMode::Native,
                    "attribute `@Native`",
                )?;
            }
            "Runtime" => {
                assign_mode(
                    &mut declared_mode,
                    &mut mode_source,
                    ExecutionMode::Runtime,
                    "attribute `@Runtime`",
                )?;
            }
            "Platforms" => {
                if attribute.arguments.is_empty() {
                    return Err(CompileError(
                        "`@Platforms` requires at least one selector".to_string(),
                    ));
                }

                platform_selectors = attribute
                    .arguments
                    .iter()
                    .map(|argument| argument.name.clone())
                    .collect();
            }
            "Export" => {
                if !attribute.arguments.is_empty() {
                    return Err(CompileError("`@Export` does not take arguments".to_string()));
                }
            }
            unknown => {
                return Err(CompileError(format!(
                    "unknown function attribute `@{unknown}`"
                )))
            }
        }
    }

    let target_platforms = if platform_selectors.is_empty() {
        // Default platform selection is host-based when a `#platforms` block exists:
        // If the host is part of the declared platform universe, treat the function as host-only
        // unless explicitly annotated with `@Platforms(...)`.
        if let Some(platforms) = platforms {
            let host = host_platform_name();
            let all = platforms.all_targets();
            if all.iter().any(|target| target == host) {
                vec![host.to_string()]
            } else {
                all
            }
        } else {
            Vec::new()
        }
    } else {
        let Some(platforms) = platforms else {
            return Err(CompileError(
                "`@Platforms` requires a global `#platforms` block".to_string(),
            ));
        };
        platforms.expand_selectors(&platform_selectors)?
    };

    Ok(ResolvedFunctionAttributes {
        declared_mode,
        target_platforms,
    })
}

fn assign_mode(
    current: &mut ExecutionMode,
    source: &mut Option<&'static str>,
    next: ExecutionMode,
    next_source: &'static str,
) -> Result<(), CompileError> {
    if let Some(existing_source) = source {
        if *current != next {
            return Err(CompileError(format!(
                "conflicting execution directives: {} and {}",
                existing_source, next_source
            )));
        }
    }

    *current = next;
    *source = Some(next_source);
    Ok(())
}
