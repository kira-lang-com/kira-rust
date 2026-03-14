use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{compiler::compile, runtime::vm::Vm};

use super::load_project;

#[test]
fn loads_a_multi_file_project() {
    let root = create_temp_project("multi_file_ok");
    write(
        &root,
        "kira.project",
        r#"
name = "my_app"
version = "0.1.0"
entry = "main.kira"
platforms = "platform.kira"
"#,
    );
    write(
        &root,
        "platform.kira",
        r#"
#platforms {
    mobile = [ios, android];
}
"#,
    );
    write(
        &root,
        "math.kira",
        r#"
func square(x: int) -> int {
    return x * x;
}
"#,
    );
    write(
        &root,
        "game.kira",
        r#"
@Runtime
func final_score(x: int, y: int, z: int, t: int) -> int {
    return square(x) + y + z + t;
}
"#,
    );
    write(
        &root,
        "main.kira",
        r#"
func main() {
    printIn(square(9));
    printIn(final_score(50, 5, 300, 3));
}
"#,
    );

    let project = load_project(&root).expect("project should load");

    assert_eq!(project.entry_symbol, "main");
    assert!(project.program.platforms.is_some());
    assert_eq!(project.program.items.len(), 3);
}

#[test]
fn rejects_internal_project_imports() {
    let root = create_temp_project("multi_file_internal_import");
    write(
        &root,
        "kira.project",
        r#"
name = "internal_import"
version = "0.1.0"
entry = "main.kira"
"#,
    );
    write(
        &root,
        "main.kira",
        r#"
import math;

func main() {
    printIn(1);
}
"#,
    );
    write(
        &root,
        "math.kira",
        r#"
func square(x: int) -> int {
    return x * x;
}
"#,
    );

    let error = load_project(&root).expect_err("project should fail");
    assert!(error
        .to_string()
        .contains("internal project import `import math;` is not allowed"));
}

#[test]
fn resolves_unqualified_project_function_calls() {
    let root = create_temp_project("multi_file_global_scope");
    write(
        &root,
        "kira.project",
        r#"
name = "unqualified"
version = "0.1.0"
entry = "main.kira"
"#,
    );
    write(
        &root,
        "math.kira",
        r#"
func square(x: int) -> int {
    return x * x;
}
"#,
    );
    write(
        &root,
        "main.kira",
        r#"
func main() {
    printIn(square(9));
}
"#,
    );

    let project = load_project(&root).expect("project should load");
    let module = compile(&project.program).expect("resolved program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, &project.entry_symbol)
        .expect("project entry should run");

    assert_eq!(vm.output(), ["81"]);
}

#[test]
fn rejects_duplicate_global_function_names_with_file_names() {
    let root = create_temp_project("multi_file_duplicate_names");
    write(
        &root,
        "kira.project",
        r#"
name = "duplicates"
version = "0.1.0"
entry = "main.kira"
"#,
    );
    write(
        &root,
        "math.kira",
        r#"
func score(x: int) -> int {
    return x * x;
}
"#,
    );
    write(
        &root,
        "other.kira",
        r#"
func score(x: int) -> int {
    return x + 1;
}
"#,
    );
    write(
        &root,
        "main.kira",
        r#"
func main() {
    printIn(score(9));
}
"#,
    );

    let error = load_project(&root).expect_err("project should fail");
    assert!(error
        .to_string()
        .contains("duplicate function `score` defined in `math.kira` and `other.kira`"));
}

#[test]
fn rejects_internal_module_qualification() {
    let root = create_temp_project("multi_file_qualification");
    write(
        &root,
        "kira.project",
        r#"
name = "qualified"
version = "0.1.0"
entry = "main.kira"
"#,
    );
    write(
        &root,
        "math.kira",
        r#"
func square(x: int) -> int {
    return x * x;
}
"#,
    );
    write(
        &root,
        "main.kira",
        r#"
func main() {
    printIn(math.square(9));
}
"#,
    );

    let error = load_project(&root).expect_err("project should fail");
    assert!(error
        .to_string()
        .contains("does not need qualification; use `square` instead"));
}

#[test]
fn imports_foundation_namespaces() {
    let root = create_temp_project("foundation_namespaces");
    write_manifest(&root, "foundation_namespaces");
    write(
        &root,
        "main.kira",
        r#"
import Foundation.Math;
import Foundation.String;

func main() {
    printIn(Math.sqrt(16.0));
    printIn(Math.clamp(150, 0, 100));
    printIn(String.concat("Hello", " Flint!"));
}
"#,
    );

    let output = run_project(&root).expect("project should run");
    assert_eq!(output, ["4", "100", "Hello Flint!"]);
}

#[test]
fn imports_entire_foundation_package() {
    let root = create_temp_project("foundation_wildcard");
    write_manifest(&root, "foundation_wildcard");
    write(
        &root,
        "main.kira",
        r#"
import Foundation;

func main() {
    printIn(Math.pi());
    printIn(String.uppercase("flint"));
}
"#,
    );

    let output = run_project(&root).expect("project should run");
    assert_eq!(output, ["3.141592653589793", "FLINT"]);
}

#[test]
fn rejects_unknown_foundation_imports() {
    let root = create_temp_project("foundation_unknown");
    write_manifest(&root, "foundation_unknown");
    write(
        &root,
        "main.kira",
        r#"
import Foundation.Unknown;

func main() {
    printIn("nope");
}
"#,
    );

    let error = load_project(&root).expect_err("project should fail");
    assert!(error
        .to_string()
        .contains("Foundation.Unknown does not exist"));
}

#[test]
fn foundation_math_functions_execute_in_vm() {
    let root = create_temp_project("foundation_math");
    write_manifest(&root, "foundation_math");
    write(
        &root,
        "main.kira",
        r#"
import Foundation.Math;

func main() {
    printIn(Math.sqrt(16.0));
    printIn(Math.floor(3.9));
    printIn(Math.ceil(3.1));
    printIn(Math.round(3.6));
    printIn(Math.pi());
    printIn(Math.clamp(150, 0, 100));
    printIn(Math.lerp(10.0, 20.0, 0.25));
    printIn(Math.sign(-42));
}
"#,
    );

    let output = run_project(&root).expect("project should run");
    assert_eq!(
        output,
        ["4", "3", "4", "4", "3.141592653589793", "100", "12.5", "-1"]
    );
}

#[test]
fn foundation_string_functions_execute_in_vm() {
    let root = create_temp_project("foundation_string");
    write_manifest(&root, "foundation_string");
    write(
        &root,
        "main.kira",
        r#"
import Foundation.String;

func main() {
    printIn(String.length("Flint"));
    printIn(String.concat("Hello", " Flint!"));
    printIn(String.contains("toolchain", "chain"));
    printIn(String.uppercase("Flint"));
    printIn(String.lowercase("FLINT"));
    printIn(String.repeat("ha", 3));
}
"#,
    );

    let output = run_project(&root).expect("project should run");
    assert_eq!(
        output,
        ["5", "Hello Flint!", "true", "FLINT", "flint", "hahaha"]
    );
}

#[test]
fn foundation_random_and_time_functions_execute_in_vm() {
    let root = create_temp_project("foundation_random_time");
    write_manifest(&root, "foundation_random_time");
    write(
        &root,
        "main.kira",
        r#"
import Foundation.Random;
import Foundation.Time;

func main() {
    printIn(Random.int(10, 20));
    printIn(Random.float(1.5, 2.5));
    printIn(Random.bool());
    printIn(Time.now());
    printIn(Time.delta());
}
"#,
    );

    let output = run_project(&root).expect("project should run");
    let random_int = output[0].parse::<i64>().expect("int output should parse");
    let random_float = output[1].parse::<f64>().expect("float output should parse");
    assert!((10..=20).contains(&random_int));
    assert!((1.5..=2.5).contains(&random_float));
    assert!(matches!(output[2].as_str(), "true" | "false"));
    assert!(output[3].parse::<i64>().expect("time should parse") > 0);
    assert!(output[4].parse::<f64>().expect("delta should parse") >= 0.0);
}

fn write_manifest(root: &PathBuf, name: &str) {
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

fn run_project(root: &PathBuf) -> Result<Vec<String>, String> {
    let project = load_project(root).map_err(|error| error.to_string())?;
    let module = compile(&project.program).map_err(|error| error.to_string())?;
    let mut vm = Vm::default();
    vm.run_entry(&module, &project.entry_symbol)
        .map_err(|error| error.to_string())?;
    Ok(vm.output().to_vec())
}

fn create_temp_project(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be valid")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("kira_{name}_{nonce}"));
    fs::create_dir_all(&root).expect("temp project dir should be created");
    root
}

fn write(root: &PathBuf, file: &str, contents: &str) {
    fs::write(root.join(file), contents).expect("file should be written");
}
