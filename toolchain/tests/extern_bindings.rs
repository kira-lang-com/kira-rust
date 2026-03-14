use toolchain::ast::TopLevelItem;
use toolchain::parser::parse;

#[test]
fn parses_opaque_and_extern_functions() {
    let source = r#"
opaque FooPtr;

extern func add64(a: int, b: int) -> int;
extern func foo_new() -> FooPtr;
"#;

    let file = parse(source).expect("source should parse");
    let mut saw_opaque = false;
    let mut extern_count = 0usize;

    for item in file.items {
        match item {
            TopLevelItem::OpaqueType(definition) => {
                assert_eq!(definition.name.name, "FooPtr");
                saw_opaque = true;
            }
            TopLevelItem::ExternFunction(definition) => {
                extern_count += 1;
                assert!(!definition.name.name.is_empty());
            }
            _ => {}
        }
    }

    assert!(saw_opaque, "expected an opaque type declaration");
    assert_eq!(extern_count, 2, "expected two extern functions");
}
