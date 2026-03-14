use std::collections::{BTreeSet, HashMap};

use crate::ast::PlatformsMetadata;

use super::CompileError;

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
