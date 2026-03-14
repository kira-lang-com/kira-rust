use crate::{compiler::compile, runtime::vm::Vm};

use super::super::load_project;
use super::{create_temp_project, write};

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
