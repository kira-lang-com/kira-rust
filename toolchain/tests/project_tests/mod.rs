mod foundation;
mod project_loading;
mod resolution;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use toolchain::{compiler::compile, runtime::vm::Vm};

use toolchain::project::load_project;

pub fn write_manifest(root: &PathBuf, name: &str) {
    write(
        root,
        "kira.project",
        &format!(
            r#"
name = "{name}"
version = "0.1.0"
entry = "main.kira"
"#
        ),
    );
}

pub fn run_project(root: &PathBuf) -> Result<Vec<String>, String> {
    let project = load_project(root).map_err(|error| error.to_string())?;
    let module = compile(&project.program).map_err(|error| error.to_string())?;
    let mut vm = Vm::default();
    vm.run_entry(&module, &project.entry_symbol)
        .map_err(|error| error.to_string())?;
    Ok(vm.output().to_vec())
}

pub fn create_temp_project(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("kira_{name}_{nonce}"));
    fs::create_dir_all(&root).expect("temp project dir should be created");
    root
}

pub fn write(root: &PathBuf, file: &str, contents: &str) {
    fs::write(root.join(file), contents).expect("file should be written");
}
