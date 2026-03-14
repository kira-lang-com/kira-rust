use toolchain::ast::{ExecutionMode, Statement, TopLevelItem};

use toolchain::parser::parse;

#[test]
fn parses_the_original_sample_syntax() {
    let source = r#"
        func add(a: int, b: int) -> int {
            return a + b;
        }

        func main() {
            printIn("Hello, world!");
            add(1, 2);
        }
    "#;

    let program = parse(source).expect("source should parse");
    assert!(program.links.is_empty());
    assert!(program.imports.is_empty());
    assert_eq!(program.items.len(), 2);

    let TopLevelItem::Function(main_fn) = &program.items[1] else {
        panic!("expected function");
    };
    assert_eq!(main_fn.execution_hint, None);
    assert!(matches!(
        main_fn.body.statements.first(),
        Some(Statement::Expression(_))
    ));
}

#[test]
fn parses_unary_negation_and_modulo() {
    let source = r#"
        func math(a: int) -> int {
            return -(a % 5);
        }
    "#;

    let program = parse(source).expect("source should parse");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn preserves_spaces_inside_string_literals() {
    let source = r#"
        func main() {
            printIn(" Flint!");
        }
    "#;

    let program = parse(source).expect("source should parse");
    let TopLevelItem::Function(function) = &program.items[0] else {
        panic!("expected function");
    };
    let Statement::Expression(statement) = &function.body.statements[0] else {
        panic!("expected expression statement");
    };
    let toolchain::ast::ExpressionKind::Call { arguments, .. } = &statement.expression.kind else {
        panic!("expected call expression");
    };
    let toolchain::ast::ExpressionKind::Literal(toolchain::ast::Literal::String(value)) =
        &arguments[0].kind
    else {
        panic!("expected string literal");
    };

    assert_eq!(value, " Flint!");
}

#[test]
fn parses_arrays_and_loop_constructs() {
    let source = r#"
        func main() {
            let numbers: [int] = [1, 2, 3];
            let empty: [int] = [];
            let flag: bool = true;

            for n in numbers {
                printIn(n);
            }

            for i in 0..=2 {
                printIn(i);
            }

            while flag {
                break;
            }

            continue;
        }
    "#;

    let file = parse(source).expect("source should parse");
    assert!(file.links.is_empty());
    let TopLevelItem::Function(function) = &file.items[0] else {
        panic!("expected function");
    };
    assert!(matches!(function.body.statements[0], Statement::Let(_)));
    assert!(matches!(function.body.statements[3], Statement::For(_)));
    assert!(matches!(function.body.statements[4], Statement::For(_)));
    assert!(matches!(function.body.statements[5], Statement::While(_)));
    assert!(matches!(function.body.statements[6], Statement::Continue(_)));
}

#[test]
fn parses_attributes_and_platform_metadata() {
    let source = r#"
        #platforms {
            mobile = [ios, android];
            desktop = [macos, windows];
        }

        @Native
        @Platforms(mobile, desktop)
        func build_target() {
            printIn("Hello, world!");
        }
    "#;

    let program = parse(source).expect("source should parse");
    assert!(program.platforms.is_some());
    assert!(program.links.is_empty());

    let TopLevelItem::Function(function) = &program.items[0] else {
        panic!("expected function");
    };
    assert_eq!(function.attributes.len(), 2);
    assert_eq!(function.attributes[0].name.name, "Native");
    assert_eq!(function.attributes[1].name.name, "Platforms");
    assert_eq!(function.attributes[1].arguments.len(), 2);
    assert_eq!(function.execution_hint, None);
}

#[test]
fn parses_legacy_runtime_keyword_as_hint() {
    let source = r#"
        runtime func main() {
            printIn("Hello, world!");
        }
    "#;

    let program = parse(source).expect("source should parse");
    let TopLevelItem::Function(function) = &program.items[0] else {
        panic!("expected function");
    };
    assert_eq!(function.execution_hint, Some(ExecutionMode::Runtime));
}

#[test]
fn parses_imports_and_qualified_calls() {
    let source = r#"
        import Foundation.Math;
        import Foundation.Random;

        func main() {
            printIn(Math.sqrt(16.0));
            printIn(Random.float(0.0, 1.0));
        }
    "#;

    let file = parse(source).expect("source should parse");
    assert!(file.links.is_empty());
    assert_eq!(file.imports.len(), 2);
    assert_eq!(file.imports[0].path[0].name, "Foundation");
    assert_eq!(file.imports[0].path[1].name, "Math");
    assert_eq!(file.imports[1].path[0].name, "Foundation");
    assert_eq!(file.imports[1].path[1].name, "Random");
}

#[test]
fn parses_structs_literals_and_field_mutation() {
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

        func main() {
            let player: Player = Player {
                name: "Kira",
                health: 100,
                position: Point { x: 0, y: 0 },
            };
            player.position.x = 5;
            printIn(player.position.x);
        }
    "#;

    let file = parse(source).expect("source should parse");
    assert!(file.links.is_empty());
    assert!(matches!(file.items[0], TopLevelItem::Struct(_)));
    assert!(matches!(file.items[1], TopLevelItem::Struct(_)));

    let TopLevelItem::Function(function) = &file.items[2] else {
        panic!("expected function");
    };
    assert!(matches!(function.body.statements[0], Statement::Let(_)));
    assert!(matches!(function.body.statements[1], Statement::Assign(_)));
    assert!(matches!(function.body.statements[2], Statement::Expression(_)));
}

#[test]
fn parses_link_directives() {
    let source = r#"
        @Link("yoga", header: "yoga/Yoga.h")
        import Foundation.Math;

        func main() {}
    "#;

    let file = parse(source).expect("source should parse");
    assert_eq!(file.links.len(), 1);
    assert_eq!(file.links[0].library, "yoga");
    assert_eq!(file.links[0].header, "yoga/Yoga.h");
    assert_eq!(file.imports.len(), 1);
    assert_eq!(file.imports[0].path[0].name, "Foundation");
    assert_eq!(file.imports[0].path[1].name, "Math");
}

#[test]
fn parses_export_struct_attribute() {
    let source = r#"
        @Export
        struct Vec2 {
            x: float,
            y: float,
        }
    "#;

    let file = parse(source).expect("source should parse");
    assert_eq!(file.items.len(), 1);
    let TopLevelItem::Struct(definition) = &file.items[0] else {
        panic!("expected struct");
    };
    assert_eq!(definition.attributes.len(), 1);
    assert_eq!(definition.attributes[0].name.name, "Export");
    assert_eq!(definition.name.name, "Vec2");
}
