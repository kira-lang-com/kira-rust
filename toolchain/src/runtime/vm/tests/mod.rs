mod basic_execution;
mod collections;
mod control_flow;

use crate::ast::Program;
use crate::parser::parse;

pub fn parse_program(source: &str) -> Program {
    let file = parse(source).expect("source should parse");
    assert!(
        file.links.is_empty(),
        "single-file VM tests should not include @Link directives"
    );
    assert!(
        file.imports.is_empty(),
        "single-file VM tests should not include imports"
    );
    Program {
        platforms: file.platforms,
        links: file.links,
        items: file.items,
    }
}
