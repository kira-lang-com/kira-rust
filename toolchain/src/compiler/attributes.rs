use crate::ast::syntax::{ExecutionMode, FunctionDefinition};

use super::{platforms::PlatformModel, CompileError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedFunctionAttributes {
    pub declared_mode: ExecutionMode,
    pub target_platforms: Vec<String>,
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
            unknown => {
                return Err(CompileError(format!(
                    "unknown function attribute `@{unknown}`"
                )))
            }
        }
    }

    let target_platforms = if platform_selectors.is_empty() {
        platforms
            .map(PlatformModel::all_targets)
            .unwrap_or_default()
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
