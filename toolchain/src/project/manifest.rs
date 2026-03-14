use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::ProjectError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySource {
    Registry,
    Path(String),
    Git { url: String, rev: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    Application,
    Library,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    pub kind: ProjectKind,
    pub entry: String,
    pub platforms: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
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
    let mut dependencies = Vec::new();
    let mut authors = Vec::new();
    let mut in_section = None;

    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        // Check for section headers
        if line.starts_with('[') && line.ends_with(']') {
            let section = line[1..line.len() - 1].trim();
            in_section = Some(section.to_string());
            continue;
        }

        let (key, value) = line.split_once('=').ok_or_else(|| {
            ProjectError(format!(
                "invalid manifest line {}: expected `key = \"value\"`",
                index + 1
            ))
        })?;

        let key = key.trim();
        let value = value.trim();

        match in_section.as_deref() {
            Some("dependencies") => {
                let dep = parse_dependency(key, value, index + 1)?;
                dependencies.push(dep);
            }
            Some("authors") => {
                let author = parse_manifest_string(value, index + 1)?;
                authors.push(author);
            }
            None => {
                let parsed_value = parse_manifest_string(value, index + 1)?;
                values.insert(key.to_string(), parsed_value);
            }
            _ => {
                return Err(ProjectError(format!(
                    "unknown section on line {}: [{}]",
                    index + 1,
                    in_section.unwrap()
                )));
            }
        }
    }

    let kind = match values.get("kind").map(|s| s.as_str()) {
        Some("library") => ProjectKind::Library,
        Some("application") | None => ProjectKind::Application,
        Some(other) => {
            return Err(ProjectError(format!(
                "invalid project kind: '{}' (expected 'application' or 'library')",
                other
            )));
        }
    };

    Ok(ProjectManifest {
        name: require_key(&values, "name")?,
        version: require_key(&values, "version")?,
        kind,
        entry: require_key(&values, "entry")?,
        platforms: values.get("platforms").cloned(),
        dependencies,
        authors,
        description: values.get("description").cloned(),
        license: values.get("license").cloned(),
    })
}

fn parse_dependency(name: &str, value: &str, line: usize) -> Result<Dependency, ProjectError> {
    // Simple version string: math = "1.0.0"
    if value.starts_with('"') && value.ends_with('"') {
        return Ok(Dependency {
            name: name.to_string(),
            version: value[1..value.len() - 1].to_string(),
            source: DependencySource::Registry,
        });
    }

    // Object-style dependency: math = { version = "1.0.0", path = "../math" }
    if value.starts_with('{') && value.ends_with('}') {
        let inner = &value[1..value.len() - 1];
        let mut dep_values = HashMap::new();

        for part in inner.split(',') {
            let (k, v) = part.split_once('=').ok_or_else(|| {
                ProjectError(format!(
                    "invalid dependency format on line {}: expected `key = \"value\"`",
                    line
                ))
            })?;
            let k = k.trim();
            let v = parse_manifest_string(v.trim(), line)?;
            dep_values.insert(k.to_string(), v);
        }

        let version = dep_values
            .get("version")
            .cloned()
            .unwrap_or_else(|| "*".to_string());

        let source = if let Some(path) = dep_values.get("path") {
            DependencySource::Path(path.clone())
        } else if let Some(git) = dep_values.get("git") {
            DependencySource::Git {
                url: git.clone(),
                rev: dep_values.get("rev").cloned(),
            }
        } else {
            DependencySource::Registry
        };

        return Ok(Dependency {
            name: name.to_string(),
            version,
            source,
        });
    }

    Err(ProjectError(format!(
        "invalid dependency format on line {}: expected version string or object",
        line
    )))
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
