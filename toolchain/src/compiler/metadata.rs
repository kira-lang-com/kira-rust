// Platform and attribute metadata processing

use std::collections::{BTreeSet, HashMap};

use crate::ast::{ExecutionMode, FunctionDefinition, PlatformsMetadata};

use super::CompileError;

// Platform model

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlatformModel {
    groups: HashMap<String, Vec<String>>,
    all_targets: Vec<String>,
}

impl PlatformModel {
    pub(super) fn all_targets(&self) -> Vec<String> {
        self.all_targets.clone()
    }

    pub(super) fn expand_selectors(
        &self,
        selectors: &[String],
    ) -> Result<Vec<String>, CompileError> {
        let mut resolved = BTreeSet::new();

        for selector in selectors {
            if let Some(targets) = self.groups.get(selector) {
                for target in targets {
                    resolved.insert(target.clone());
                }
                continue;
            }

            if self.all_targets.iter().any(|target| target == selector) {
                resolved.insert(selector.clone());
                continue;
            }

            return Err(CompileError(format!(
                "unknown platform selector `{selector}`"
            )));
        }

        Ok(resolved.into_iter().collect())
    }
}

pub(super) fn build_platform_model(
    metadata: Option<&PlatformsMetadata>,
) -> Result<Option<PlatformModel>, CompileError> {
    let Some(metadata) = metadata else {
        return Ok(None);
    };

    let mut groups = HashMap::new();
    let mut all_targets = BTreeSet::new();

    for group in &metadata.groups {
        if groups.contains_key(&group.name.name) {
            return Err(CompileError(format!(
                "duplicate platform group `{}`",
                group.name.name
            )));
        }

        let mut members = Vec::new();
        for member in &group.members {
            members.push(member.name.clone());
            all_targets.insert(member.name.clone());
        }

        groups.insert(group.name.name.clone(), members);
    }

    Ok(Some(PlatformModel {
        groups,
        all_targets: all_targets.into_iter().collect(),
    }))
}

// Attribute resolution

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
