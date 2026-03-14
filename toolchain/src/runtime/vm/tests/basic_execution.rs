use ordered_float::OrderedFloat;

use crate::compiler::compile;
use crate::runtime::{Value, Vm};

use super::parse_program;

#[test]
fn runs_scripted_functions_through_the_vm() {
    let source = r#"
        func add(a: int, b: int) -> int {
            return a + b;
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    let result = vm
        .run_function(&module, "add", vec![Value::Int(20), Value::Int(22)])
        .expect("vm should execute");

    assert_eq!(result, Value::Int(42));
}

#[test]
fn vm_uses_bytecode_shadow_for_auto_native_functions() {
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
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["Hello, world!"]);
}

#[test]
fn vm_executes_complex_math_programs() {
    let source = r#"
        func square(x: int) -> int {
            return x * x;
        }

        script func advanced(seed: int) -> int {
            let folded: int = -(seed % 7);
            let powered: int = pow(seed + 2, 3);
            let mixed: int = max(square(seed), powered + abs(folded));
            return min(mixed, 400);
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    let result = vm
        .run_function(&module, "advanced", vec![Value::Int(5)])
        .expect("vm should execute");

    assert_eq!(result, Value::Int(348));
}

#[test]
fn vm_executes_factorial_with_if_else() {
    let source = r#"
        @Runtime
        func factorial(n: int) -> int {
            if n <= 1 {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    let result = vm
        .run_function(&module, "factorial", vec![Value::Int(5)])
        .expect("vm should execute");

    assert_eq!(result, Value::Int(120));
}

#[test]
fn vm_executes_explicit_float_casts() {
    let source = r#"
        func ratio(a: int, b: int) -> float {
            return float(a) / float(b);
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    let result = vm
        .run_function(&module, "ratio", vec![Value::Int(7), Value::Int(2)])
        .expect("vm should execute");

    assert_eq!(result, Value::Float(OrderedFloat(3.5)));
}
