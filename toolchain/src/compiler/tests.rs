use crate::{ast::syntax::Program, parser::parse};

use super::{compile, BackendKind, BuildStage};

fn parse_program(source: &str) -> Program {
    let file = parse(source).expect("source should parse");
    assert!(
        file.links.is_empty(),
        "single-file compiler tests should not include @Link directives"
    );
    assert!(
        file.imports.is_empty(),
        "single-file compiler tests should not include imports"
    );
    Program {
        platforms: file.platforms,
        links: file.links,
        items: file.items,
    }
}

#[test]
fn auto_mode_selects_native_for_static_leaf_functions() {
    let source = r#"
        func add(a: int, b: int) -> int {
            return a + b;
        }

        func main() {
            printIn("Hello, world!");
            add(1, 2);
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");

    assert_eq!(
        module.functions["add"].selected_backend,
        BackendKind::Native
    );
    assert_eq!(module.functions["main"].selected_backend, BackendKind::Native);
    assert!(module.functions["add"].artifacts.bytecode.is_some());
    assert!(module.functions["add"].artifacts.aot.is_some());
    assert_eq!(module.aot_plan.stage, BuildStage::BuildTimeOnly);
}

#[test]
fn native_functions_accept_dynamic_values() {
    let source = r#"
        @Native
        func echo(value: dynamic) {
            printIn(value);
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    assert_eq!(module.functions["echo"].selected_backend, BackendKind::Native);
}

#[test]
fn compiles_complex_math_functions() {
    let source = r#"
        func square(x: int) -> int {
            return x * x;
        }

        script func advanced(seed: int) -> int {
            let folded: int = -(seed % 7);
            let powered: int = pow(seed + 2, 3);
            return max(square(seed), powered + abs(folded));
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");

    assert_eq!(
        module.functions["square"].selected_backend,
        BackendKind::Native
    );
    assert_eq!(
        module.functions["advanced"].selected_backend,
        BackendKind::Vm
    );
}

#[test]
fn resolves_attributes_platforms_and_build_time_aot_jobs() {
    let source = r#"
        #platforms {
            mobile = [ios, android];
            desktop = [macos, windows];
        }

        @Native
        @Platforms(mobile, desktop)
        func square(x: int) -> int {
            return x * x;
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");

    assert_eq!(
        module.functions["square"].declared_mode,
        crate::ast::ExecutionMode::Native
    );
    assert_eq!(
        module.functions["square"].target_platforms,
        vec![
            "android".to_string(),
            "ios".to_string(),
            "macos".to_string(),
            "windows".to_string()
        ]
    );
    assert_eq!(module.aot_plan.jobs.len(), 1);
    assert_eq!(
        module.aot_plan.jobs[0].artifact.stage,
        BuildStage::BuildTimeOnly
    );
}

#[test]
fn platforms_default_to_host_when_metadata_exists() {
    let host = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    let source = r#"
        #platforms {
            mobile = [ios, android];
            desktop = [macos, windows, linux];
        }

        // No @Platforms annotation: should default to host-only.
        @Native
        func host_only() -> int {
            return 42;
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");

    assert_eq!(module.functions["host_only"].target_platforms, vec![host.to_string()]);
}

#[test]
fn enforces_explicit_float_casts() {
    let source = r#"
        func ratio(a: int, b: int) -> float {
            return a / b;
        }
    "#;

    let program = parse_program(source);
    let error = compile(&program).expect_err("compile should fail");

    assert!(error.to_string().contains("return type mismatch"));
}

#[test]
fn exposes_foundation_builtins_as_native_library_functions() {
    let program = parse_program(
        r#"
        func main() {
            printIn("Hello, world!");
        }
    "#,
    );
    let module = compile(&program).expect("program should compile");

    assert_eq!(
        module.builtins["Foundation.Math.sqrt"].backend,
        BackendKind::Native
    );
    assert_eq!(
        module.builtins["Foundation.String.concat"].backend,
        BackendKind::Native
    );
    assert_eq!(
        module.builtins["Foundation.Random.int"].backend,
        BackendKind::Native
    );
    assert_eq!(
        module.builtins["Foundation.Time.now"].backend,
        BackendKind::Native
    );
}

#[test]
fn native_array_functions_still_get_aot_plans() {
    let source = r#"
        @Native
        func sum_array(arr: [int]) -> int {
            let total: int = 0;
            for n in arr {
                total = total + n;
            }
            return total;
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");

    assert_eq!(
        module.functions["sum_array"].selected_backend,
        BackendKind::Native
    );
    assert!(module.functions["sum_array"].artifacts.aot.is_some());
}

#[test]
fn native_array_literals_compile_with_known_element_types() {
    let source = r#"
        @Native
        func main() {
            let numbers: [int] = [1, 2, 3];
            printIn(numbers.length);
            printIn(numbers[1]);
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");

    assert_eq!(module.functions["main"].selected_backend, BackendKind::Native);
    assert!(module.functions["main"].artifacts.aot.is_some());
}

#[test]
fn structs_compile_natively_when_field_types_are_native() {
    let auto_program = parse_program(
        r#"
        struct Point {
            x: int,
            y: int,
        }

        func distance_squared(a: Point, b: Point) -> int {
            let dx: int = b.x - a.x;
            let dy: int = b.y - a.y;
            return dx * dx + dy * dy;
        }
    "#,
    );
    let auto_module = compile(&auto_program).expect("program should compile");

    assert_eq!(
        auto_module.functions["distance_squared"].selected_backend,
        BackendKind::Native
    );

    let native_program = parse_program(
        r#"
        struct Point {
            x: int,
            y: int,
        }

        @Native
        func bad(point: Point) -> int {
            return point.x;
        }
    "#,
    );
    let native_module = compile(&native_program).expect("program should compile");
    assert_eq!(native_module.functions["bad"].selected_backend, BackendKind::Native);
}

#[test]
fn reports_unknown_struct_fields_at_compile_time() {
    let program = parse_program(
        r#"
        struct Point {
            x: int,
            y: int,
        }

        func bad(point: Point) -> int {
            return point.z;
        }
    "#,
    );

    let error = compile(&program).expect_err("compile should fail");
    assert_eq!(error.to_string(), "Point has no field 'z'");
}

#[test]
fn native_structs_support_literals_and_field_access() {
    let program = parse_program(
        r#"
        struct Point {
            x: int,
            y: int,
        }

        @Native
        func main() {
            let point: Point = Point { x: 1, y: 2 };
            printIn(point.x);
        }
    "#,
    );

    let module = compile(&program).expect("program should compile");
    assert_eq!(module.functions["main"].selected_backend, BackendKind::Native);
}
