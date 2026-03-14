# Agent Guidelines for Kira Development

This document provides guidelines for AI agents working on the Kira programming language toolchain.

## Core Principles

### 1. File Size Limit: 200 Lines Maximum

**Every source file should be under 200 lines of code.** This is a hard limit that improves:
- Code readability and maintainability
- Easier code review and debugging
- Better separation of concerns
- Reduced cognitive load when understanding code

**When a file approaches 200 lines:**
- Extract logical units into separate files
- Create subdirectories to organize related functionality
- Split large functions into smaller, focused functions in separate files
- Use the module system to maintain clean interfaces

### 2. One File, One Purpose

Each file should have a single, clear responsibility:

**Good examples:**
- `native_build.rs` - Handles native C library compilation
- `ffi_loader.rs` - Manages dynamic FFI library loading
- `run_module.rs` - Executes pre-compiled modules

**Bad examples:**
- `utils.rs` with unrelated helper functions
- `common.rs` with mixed functionality
- Files that do "multiple things"

### 3. Use Folders for Organization

Organize related files into logical folder structures:

```
compiler/
├── lowering/          # Bytecode lowering
│   ├── assignments.rs
│   ├── calls.rs
│   ├── expressions.rs
│   ├── function.rs
│   ├── literals.rs
│   ├── loops.rs
│   ├── operators.rs
│   └── statements.rs
├── eligibility/       # Native compilation eligibility
│   ├── analyzer.rs
│   ├── assignments.rs
│   ├── calls.rs
│   └── ...
└── native_build.rs    # Top-level native build logic
```

**Benefits:**
- Clear mental model of codebase structure
- Easy to locate functionality
- Natural boundaries between components
- Scales well as project grows

### 4. Avoid Nested Code

Minimize nesting depth to improve readability:

**Bad:**
```rust
if condition1 {
    if condition2 {
        if condition3 {
            if condition4 {
                // deeply nested logic
            }
        }
    }
}
```

**Good:**
```rust
if !condition1 {
    return early;
}
if !condition2 {
    return early;
}
if !condition3 {
    return early;
}
if !condition4 {
    return early;
}
// main logic at top level
```

**Techniques:**
- Use early returns (guard clauses)
- Extract nested logic into separate functions
- Use `?` operator for error handling
- Prefer flat control flow over deep nesting

### 5. Code Quality Standards

#### Naming Conventions
- Use descriptive, self-documenting names
- Functions: `verb_noun` (e.g., `build_native_library`, `load_ffi_metadata`)
- Types: `PascalCase` (e.g., `FfiLoader`, `CompiledModule`)
- Constants: `SCREAMING_SNAKE_CASE`
- Avoid abbreviations unless universally understood

#### Function Design
- Keep functions focused and small (ideally under 30 lines)
- Each function should do one thing well
- Minimize parameters (3-4 max, use structs for more)
- Return `Result<T, E>` for operations that can fail

#### Error Handling
- Use descriptive error messages
- Provide context in error messages (file paths, operation being performed)
- Propagate errors with `?` operator
- Don't use `unwrap()` or `expect()` in production code

#### Documentation
- Add doc comments (`///`) for public APIs
- Explain "why" not just "what"
- Include examples for complex functionality
- Document assumptions and invariants

#### Testing
- Write tests for new functionality
- Test edge cases and error conditions
- Keep tests focused and independent
- Use descriptive test names

## Architecture Patterns

### Module Organization

```rust
// mod.rs - Public interface
pub use submodule::PublicType;
pub use another::public_function;

mod submodule;
mod another;
mod internal; // Not re-exported
```

### Separation of Concerns

**Layers:**
1. **CLI Layer** (`src/cli/`) - User interface, argument parsing
2. **Compiler Layer** (`src/compiler/`) - Language compilation logic
3. **Runtime Layer** (`src/runtime/`) - VM and execution
4. **AOT Layer** (`src/aot/`) - Ahead-of-time compilation
5. **Project Layer** (`src/project/`) - Project management

**Each layer should:**
- Have clear boundaries
- Depend only on lower layers
- Expose minimal public API
- Be testable in isolation

### Dependency Management

- Minimize dependencies between modules
- Use traits for abstraction
- Prefer composition over inheritance
- Keep coupling loose

## Refactoring Guidelines

### When to Split a File

Split when:
- File exceeds 150 lines (before hitting 200 limit)
- Multiple distinct responsibilities exist
- Code has natural boundaries (e.g., different phases)
- Testing becomes difficult due to size

### How to Split

1. **Identify logical units** - Group related functions/types
2. **Create new files** - One per logical unit
3. **Update mod.rs** - Re-export public items
4. **Move code** - Use your editor's refactoring tools
5. **Test** - Ensure everything still works

### Refactoring Checklist

- [ ] Each file under 200 lines
- [ ] Clear single purpose per file
- [ ] Minimal nesting (max 3 levels)
- [ ] Descriptive names throughout
- [ ] Error handling with context
- [ ] Documentation for public APIs
- [ ] Tests for new functionality

## Common Patterns in Kira

### Compilation Pipeline

```
Source → Parse → Resolve → Lower → Compile → Execute/Build
```

Each phase is in a separate module with clear inputs/outputs.

### Error Handling

```rust
pub struct CompileError(pub String);

pub fn compile_thing() -> Result<Output, CompileError> {
    let data = load_data()
        .map_err(|e| CompileError(format!("Failed to load: {}", e)))?;
    // ... more operations
    Ok(output)
}
```

### Builder Pattern

```rust
pub struct ConfigBuilder {
    field1: Option<Type1>,
    field2: Option<Type2>,
}

impl ConfigBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn field1(mut self, value: Type1) -> Self { /* ... */ }
    pub fn build(self) -> Result<Config, Error> { /* ... */ }
}
```

## Working with Existing Code

### Before Making Changes

1. **Understand the context** - Read related files
2. **Check file size** - If near 200 lines, plan to split
3. **Identify dependencies** - What depends on this code?
4. **Run tests** - Ensure baseline functionality

### Making Changes

1. **Keep changes focused** - One logical change per commit
2. **Maintain file size limits** - Split if needed
3. **Update tests** - Add/modify tests for changes
4. **Document** - Update comments and docs
5. **Check quality** - Run linter, formatter

### After Changes

1. **Run full test suite**
2. **Check for warnings** - Fix unused imports, variables
3. **Review file sizes** - Ensure all under 200 lines
4. **Update documentation** - If public API changed
5. **Commit with clear message**

## Examples of Good Structure

### Good: Focused, Small Files

```
compiler/native_build.rs (95 lines)
- build_native_library()
- build_all_native_dependencies()
- Platform-specific compilation logic
```

### Good: Organized Subdirectory

```
runtime/vm/machine/
├── mod.rs (30 lines) - Public interface
├── vm.rs (150 lines) - VM implementation
├── stack.rs (120 lines) - Stack management
└── execution.rs (180 lines) - Instruction execution
```

### Bad: Monolithic File

```
utils.rs (500 lines)
- String helpers
- File I/O helpers
- Math helpers
- Random utilities
```

**Fix:** Split into focused files:
```
utils/
├── string.rs
├── file.rs
├── math.rs
└── mod.rs
```

## Questions to Ask

When writing or reviewing code:

1. **Is this file under 200 lines?**
2. **Does this file have one clear purpose?**
3. **Is the nesting depth reasonable (≤3 levels)?**
4. **Are names descriptive and self-documenting?**
5. **Is error handling comprehensive with context?**
6. **Would a new developer understand this quickly?**
7. **Are there tests for this functionality?**
8. **Is this the right place for this code?**

## Summary

- **200 lines max per file** - Hard limit, split before reaching it
- **One purpose per file** - Clear, focused responsibility
- **Use folders** - Organize related functionality
- **Avoid nesting** - Use early returns and flat control flow
- **Quality matters** - Readable, maintainable, well-tested code

Following these guidelines ensures the Kira codebase remains clean, maintainable, and easy to work with as it grows.
