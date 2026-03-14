use toolchain::compiler::compile;
use toolchain::runtime::value::StructValue;
use toolchain::runtime::{Value, Vm};

use super::parse_program;

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
