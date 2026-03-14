use super::super::load_project;
use super::{create_temp_project, write};

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
fn rejects_unknown_foundation_imports() {
    let root = create_temp_project("foundation_unknown");
    use super::write_manifest;
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
