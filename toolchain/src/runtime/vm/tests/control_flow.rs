use crate::compiler::compile;
use crate::runtime::Vm;

use super::parse_program;

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
