use ordered_float::OrderedFloat;

use crate::{
    ast::syntax::Program,
    compiler::compile,
    parser::parse,
    runtime::{value::StructValue, Value},
};

use super::Vm;

fn parse_program(source: &str) -> Program {
    let file = parse(source).expect("source should parse");
    assert!(
        file.imports.is_empty(),
        "single-file VM tests should not include imports"
    );
    Program {
        platforms: file.platforms,
        items: file.items,
    }
}

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

#[test]
fn vm_executes_arrays_and_loops_end_to_end() {
    let source = r#"
        func sum_array(arr: [int]) -> int {
            let total: int = 0;
            for n in arr {
                total = total + n;
            }
            return total;
        }

        func main() {
            let numbers: [int] = [1, 2, 3, 4, 5];
            printIn(sum_array(numbers));

            for i in 0..5 {
                printIn(i);
            }

            let i: int = 0;
            while i < 3 {
                printIn(i);
                i = i + 1;
            }
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["15", "0", "1", "2", "3", "4", "0", "1", "2"]);
}

#[test]
fn vm_executes_array_operations_and_inclusive_ranges() {
    let source = r#"
        func main() {
            let numbers: [int] = [];
            numbers.append(1);
            numbers.append(2);
            numbers.append(3);

            printIn(numbers.length);
            printIn(numbers[0]);
            printIn(numbers[2]);

            for i in 0..=2 {
                printIn(i);
            }
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["3", "1", "3", "0", "1", "2"]);
}

#[test]
fn vm_executes_break_continue_and_nested_loops() {
    let source = r#"
        func main() {
            let values: [int] = [1, 2, 3, 4, 5];
            let total: int = 0;

            for n in values {
                if n == 2 {
                    continue;
                } else {
                    if n == 5 {
                        break;
                    } else {
                        total = total + n;
                    }
                }
            }

            printIn(total);

            for i in 0..3 {
                for j in 0..3 {
                    if j == 1 {
                        continue;
                    } else {
                        if i == 2 {
                            break;
                        } else {
                            printIn((i * 10) + j);
                        }
                    }
                }
            }
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["8", "0", "2", "10", "12"]);
}

#[test]
fn vm_executes_boolean_while_conditions() {
    let source = r#"
        func main() {
            let flag: bool = true;
            let count: int = 0;

            while flag {
                printIn(count);
                count = count + 1;

                if count >= 2 {
                    flag = false;
                } else {
                    printIn(99);
                }
            }
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["0", "99", "1"]);
}

#[test]
fn vm_executes_structs_nested_fields_and_copy_semantics() {
    let source = r#"
        struct Point {
            x: int,
            y: int,
        }

        struct Player {
            name: string,
            health: int,
            position: Point,
        }

        func heal(player: Player, amount: int) -> Player {
            player.health = player.health + amount;
            player.position.x = player.position.x + 1;
            return player;
        }

        func main() {
            let point: Point = Point { x: 10, y: 20 };
            let moved: Point = point;
            moved.x = 99;

            let player: Player = Player {
                name: "Kira",
                health: 100,
                position: Point { x: 0, y: 0 },
            };
            let healed: Player = heal(player, 25);

            printIn(point.x);
            printIn(moved.x);
            printIn(player.health);
            printIn(player.position.x);
            printIn(healed.health);
            printIn(healed.position.x);
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["10", "99", "100", "0", "125", "1"]);
}

#[test]
fn vm_runs_struct_validation_program_end_to_end() {
    let source = r#"
        struct Point {
            x: int,
            y: int,
        }

        func make_point(x: int, y: int) -> Point {
            return Point { x: x, y: y };
        }

        func distance_squared(a: Point, b: Point) -> int {
            let dx: int = b.x - a.x;
            let dy: int = b.y - a.y;
            return dx * dx + dy * dy;
        }

        func main() {
            let a: Point = make_point(0, 0);
            let b: Point = make_point(3, 4);
            printIn(distance_squared(a, b));
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    vm.run_entry(&module, "main").expect("vm should execute");

    assert_eq!(vm.output(), ["25"]);
}

#[test]
fn vm_returns_struct_values_from_runtime_functions() {
    let source = r#"
        struct Point {
            x: int,
            y: int,
        }

        func make_point(x: int, y: int) -> Point {
            return Point { x: x, y: y };
        }
    "#;

    let program = parse_program(source);
    let module = compile(&program).expect("program should compile");
    let mut vm = Vm::default();

    let result = vm
        .run_function(&module, "make_point", vec![Value::Int(3), Value::Int(4)])
        .expect("vm should execute");

    assert_eq!(
        result,
        Value::Struct(StructValue {
            type_name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Value::Int(3)),
                ("y".to_string(), Value::Int(4)),
            ],
        })
    );
}
