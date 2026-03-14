use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::ProjectError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub platforms: Option<String>,
}

pub fn load_manifest(path: &Path) -> Result<ProjectManifest, ProjectError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ProjectError(format!(
            "failed to read manifest `{}`: {}",
            path.display(),
            error
        ))
    })?;

    parse_manifest(&source)
}

fn parse_manifest(source: &str) -> Result<ProjectManifest, ProjectError> {
    let mut values = HashMap::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        let (key, value) = line.split_once('=').ok_or_else(|| {
            ProjectError(format!(
                "invalid manifest line {}: expected `key = \"value\"`",
                index + 1
            ))
        })?;

        let key = key.trim().to_string();
        let value = parse_manifest_string(value.trim(), index + 1)?;
        values.insert(key, value);
    }

    Ok(ProjectManifest {
        name: require_key(&values, "name")?,
        version: require_key(&values, "version")?,
        entry: require_key(&values, "entry")?,
        platforms: values.get("platforms").cloned(),
    })
}

fn require_key(values: &HashMap<String, String>, key: &str) -> Result<String, ProjectError> {
    values
        .get(key)
        .cloned()
        .ok_or_else(|| ProjectError(format!("manifest is missing required key `{key}`")))
}

fn parse_manifest_string(value: &str, line: usize) -> Result<String, ProjectError> {
    if !value.starts_with('"') || !value.ends_with('"') || value.len() < 2 {
        return Err(ProjectError(format!(
            "invalid manifest string on line {}: expected quoted value",
            line
        )));
    }

    Ok(value[1..value.len() - 1].to_string())
}
