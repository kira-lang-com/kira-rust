use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CType {
    Void,
    Bool,
    Int64,
    Double,
    Named(String),
    Pointer { base: Box<CType>, is_const: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CField {
    pub name: String,
    pub ty: CType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<CField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CParam {
    pub name: Option<String>,
    pub ty: CType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CFunction {
    pub name: String,
    pub return_type: CType,
    pub params: Vec<CParam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedHeader {
    pub opaque_typedefs: Vec<String>,
    pub structs: Vec<CStruct>,
    pub functions: Vec<CFunction>,
}

pub fn parse_header(source: &str) -> Result<ParsedHeader, String> {
    let stripped = strip_comments_and_directives(source);
    let tokens = tokenize(&stripped);
    let statements = split_statements(&tokens);

    let mut parsed = ParsedHeader::default();

    for stmt in statements {
        if stmt.first().is_some_and(|t| t == "typedef") {
            parse_typedef_statement(&stmt, &mut parsed)?;
            continue;
        }

        if let Some(function) = parse_function_statement(&stmt, &parsed)? {
            parsed.functions.push(function);
        }
    }

    Ok(parsed)
}

fn strip_comments_and_directives(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '/' {
            match chars.peek().copied() {
                Some('/') => {
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '\0';
                    while let Some(next) = chars.next() {
                        if prev == '*' && next == '/' {
                            break;
                        }
                        prev = next;
                    }
                    continue;
                }
                _ => {}
            }
        }

        out.push(ch);
    }

    // Drop preprocessor directives (`#...`) line-by-line.
    out.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
            continue;
        }

        if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }

        if ch.is_whitespace() {
            continue;
        }

        match ch {
            '(' | ')' | '{' | '}' | ';' | ',' | '*' | ':' => tokens.push(ch.to_string()),
            _ => {
                // Ignore unknown punctuation (macros, attributes, etc).
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn split_statements(tokens: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;

    for token in tokens {
        match token.as_str() {
            "{" => brace_depth += 1,
            "}" => brace_depth = brace_depth.saturating_sub(1),
            "(" => paren_depth += 1,
            ")" => paren_depth = paren_depth.saturating_sub(1),
            ";" if brace_depth == 0 && paren_depth == 0 => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
                continue;
            }
            _ => {}
        }

        current.push(token.clone());
    }

    if !current.is_empty() {
        out.push(current);
    }

    out
}

fn parse_typedef_statement(stmt: &[String], parsed: &mut ParsedHeader) -> Result<(), String> {
    // Supported:
    // - `typedef struct Name* Alias;` (opaque)
    // - `typedef void* Alias;` (opaque)
    // - `typedef struct { ... } Alias;` (struct)
    // - `typedef struct Name { ... } Alias;` (struct)
    if stmt.len() < 3 {
        return Ok(());
    }

    let mut i = 1;
    if stmt.get(i).is_some_and(|t| t == "struct") {
        i += 1;
        let tag = stmt.get(i).cloned().filter(|t| is_ident(t));
        if tag.is_some() {
            i += 1;
        }

        if stmt.get(i).is_some_and(|t| t == "{") {
            let tag = tag.unwrap_or_else(|| "Anonymous".to_string());
            let (struct_def, next) = parse_struct_body(stmt, i, tag)?;
            i = next;
            let alias = stmt
                .get(i)
                .cloned()
                .ok_or_else(|| "missing typedef alias after struct body".to_string())?;
            parsed.structs.push(CStruct {
                name: alias,
                fields: struct_def.fields,
            });
            return Ok(());
        }

        // No body; treat pointer typedef as opaque.
        // Look for the last identifier as the alias name.
        if stmt.iter().any(|t| t == "*") {
            if let Some(alias) = stmt.iter().rev().find(|t| is_ident(t)).cloned() {
                parsed.opaque_typedefs.push(alias);
            }
        }
        return Ok(());
    }

    // `typedef void* Alias;` or `typedef const void * Alias;`
    if stmt.get(i).is_some_and(|t| t == "void") && stmt.iter().any(|t| t == "*") {
        if let Some(alias) = stmt.iter().rev().find(|t| is_ident(t)).cloned() {
            parsed.opaque_typedefs.push(alias);
        }
    }

    Ok(())
}

fn parse_struct_body(
    stmt: &[String],
    open_brace_index: usize,
    name: String,
) -> Result<(CStruct, usize), String> {
    let mut i = open_brace_index + 1;
    let mut fields = Vec::new();
    let mut field_tokens = Vec::new();

    while i < stmt.len() {
        let token = &stmt[i];
        if token == "}" {
            break;
        }

        if token == ";" {
            if let Some(field) = parse_field_decl(&field_tokens)? {
                fields.push(field);
            }
            field_tokens.clear();
            i += 1;
            continue;
        }

        field_tokens.push(token.clone());
        i += 1;
    }

    if i >= stmt.len() || stmt[i] != "}" {
        return Err("unterminated struct body".to_string());
    }

    Ok((CStruct { name, fields }, i + 1))
}

fn parse_field_decl(tokens: &[String]) -> Result<Option<CField>, String> {
    // Expect: `<type> <name>`
    if tokens.is_empty() {
        return Ok(None);
    }
    let name = tokens
        .iter()
        .rev()
        .find(|t| is_ident(t))
        .cloned()
        .ok_or_else(|| "invalid struct field declaration".to_string())?;
    let type_tokens = tokens
        .iter()
        .take_while(|t| *t != &name)
        .cloned()
        .collect::<Vec<_>>();
    let ty = parse_type(&type_tokens, &HashMap::new())?;
    Ok(Some(CField { name, ty }))
}

fn parse_function_statement(
    stmt: &[String],
    parsed: &ParsedHeader,
) -> Result<Option<CFunction>, String> {
    // Heuristic: find `<name> ( ... )` and treat it as a function decl.
    let Some(open_paren) = stmt.iter().position(|t| t == "(") else {
        return Ok(None);
    };
    if open_paren == 0 {
        return Ok(None);
    }
    let func_name = stmt[open_paren - 1].clone();
    if !is_ident(&func_name) {
        return Ok(None);
    }

    let return_tokens = stmt[..open_paren - 1]
        .iter()
        .cloned()
        .filter(|t| t != "extern")
        .collect::<Vec<_>>();
    let return_type = parse_type(&return_tokens, &HashMap::new())?;

    let close_paren = stmt
        .iter()
        .rposition(|t| t == ")")
        .ok_or_else(|| "unterminated parameter list".to_string())?;
    if close_paren < open_paren {
        return Ok(None);
    }
    let params_tokens = &stmt[open_paren + 1..close_paren];
    let params = parse_params(params_tokens, parsed)?;

    Ok(Some(CFunction {
        name: func_name,
        return_type,
        params,
    }))
}

fn parse_params(tokens: &[String], parsed: &ParsedHeader) -> Result<Vec<CParam>, String> {
    if tokens.len() == 1 && tokens[0] == "void" {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;

    for token in tokens {
        match token.as_str() {
            "(" => depth += 1,
            ")" => depth = depth.saturating_sub(1),
            "," if depth == 0 => {
                if let Some(param) = parse_param(&current, parsed)? {
                    out.push(param);
                }
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(token.clone());
    }

    if let Some(param) = parse_param(&current, parsed)? {
        out.push(param);
    }

    Ok(out)
}

fn parse_param(tokens: &[String], parsed: &ParsedHeader) -> Result<Option<CParam>, String> {
    let tokens = tokens.iter().filter(|t| !t.is_empty()).cloned().collect::<Vec<_>>();
    if tokens.is_empty() {
        return Ok(None);
    }

    // Optional name is the last identifier, unless it's a known typedef/struct name.
    let known_names = parsed
        .opaque_typedefs
        .iter()
        .chain(parsed.structs.iter().map(|s| &s.name))
        .collect::<Vec<_>>();

    let mut name = None;
    let mut type_tokens = tokens.clone();

    if let Some(last_ident) = tokens.iter().rev().find(|t| is_ident(t)).cloned() {
        if !known_names.iter().any(|n| **n == last_ident) {
            name = Some(last_ident.clone());
            // Remove the name token from the end.
            if let Some(pos) = type_tokens.iter().rposition(|t| t == &last_ident) {
                type_tokens.remove(pos);
            }
        }
    }

    let ty = parse_type(&type_tokens, &HashMap::new())?;
    Ok(Some(CParam { name, ty }))
}

fn parse_type(tokens: &[String], _aliases: &HashMap<String, CType>) -> Result<CType, String> {
    let mut is_const_ptr = false;
    let mut tokens = tokens.to_vec();
    if tokens.first().is_some_and(|t| t == "const") {
        is_const_ptr = true;
        tokens.remove(0);
    }

    let pointer_count = tokens.iter().filter(|t| *t == "*").count();
    tokens.retain(|t| t != "*");

    // Ignore calling convention / export macro identifiers (best-effort).
    while tokens.first().is_some_and(|t| t.chars().all(|c| c.is_ascii_uppercase() || c == '_')) {
        tokens.remove(0);
    }

    let base = if tokens.is_empty() {
        return Err("missing type".to_string());
    } else if tokens.len() == 1 && tokens[0] == "void" {
        CType::Void
    } else if tokens.len() == 1 && (tokens[0] == "bool" || tokens[0] == "_Bool") {
        CType::Bool
    } else if tokens.len() == 1 && tokens[0] == "int64_t" {
        CType::Int64
    } else if tokens.len() == 1 && tokens[0] == "double" {
        CType::Double
    } else if tokens.len() == 2 && tokens[0] == "struct" && is_ident(&tokens[1]) {
        CType::Named(tokens[1].clone())
    } else if tokens.len() == 1 && is_ident(&tokens[0]) {
        CType::Named(tokens[0].clone())
    } else {
        return Err(format!("unsupported C type tokens: `{}`", tokens.join(" ")));
    };

    if pointer_count == 0 {
        return Ok(base);
    }

    Ok(CType::Pointer {
        base: Box::new(base),
        is_const: is_const_ptr,
    })
}

fn is_ident(token: &str) -> bool {
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typedef_structs_functions_and_opaque_pointers() {
        let header = r#"
            #pragma once
            typedef struct Foo* FooRef;
            typedef struct { double x; double y; } Vec2;

            int64_t add_i64(int64_t a, int64_t b);
            double dot(Vec2 a, Vec2 b);
            FooRef foo_new(void);
        "#;

        let parsed = parse_header(header).expect("header should parse");
        assert_eq!(parsed.opaque_typedefs, vec!["FooRef"]);
        assert_eq!(parsed.structs.len(), 1);
        assert_eq!(parsed.structs[0].name, "Vec2");
        assert_eq!(parsed.structs[0].fields.len(), 2);
        assert_eq!(parsed.structs[0].fields[0].name, "x");
        assert_eq!(parsed.structs[0].fields[1].name, "y");

        let names = parsed
            .functions
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["add_i64", "dot", "foo_new"]);
    }
}
