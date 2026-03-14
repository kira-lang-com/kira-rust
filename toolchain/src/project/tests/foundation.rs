use super::{create_temp_project, run_project, write, write_manifest};

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
