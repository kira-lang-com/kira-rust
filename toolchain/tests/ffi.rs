use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "kira_toolchain_test_{}_{}_{}",
        prefix,
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn build_c_mylib(native_dir: &Path) -> PathBuf {
    let header = native_dir.join("mylib.h");
    let source = native_dir.join("mylib.c");

    write_file(
        &header,
        r#"
#pragma once
#include <stdint.h>

typedef struct Foo* FooRef;

int64_t add64(int64_t a, int64_t b);
FooRef foo_new(void);
int64_t foo_value(FooRef foo);
void foo_free(FooRef foo);
"#
        .trim(),
    );

    write_file(
        &source,
        r#"
#include "mylib.h"
#include <stdlib.h>

struct Foo {
    int64_t value;
};

int64_t add64(int64_t a, int64_t b) { return a + b; }

FooRef foo_new(void) {
    struct Foo* foo = (struct Foo*)malloc(sizeof(struct Foo));
    foo->value = 123;
    return foo;
}

int64_t foo_value(FooRef foo) { return foo->value; }

void foo_free(FooRef foo) { free(foo); }
"#
        .trim(),
    );

    let lib_name = if cfg!(target_os = "macos") {
        "libmylib.dylib"
    } else {
        "libmylib.so"
    };
    let lib_path = native_dir.join(lib_name);

    let mut clang = Command::new("clang");
    if cfg!(target_os = "macos") {
        clang
            .arg("-dynamiclib")
            .arg("-install_name")
            .arg(format!("@rpath/{}", lib_name));
    } else {
        clang.arg("-shared").arg("-fPIC");
    }
    clang
        .arg("-o")
        .arg(&lib_path)
        .arg(&source)
        .current_dir(native_dir);

    let status = clang.status().expect("clang should run");
    assert!(status.success(), "C dylib should build");

    lib_path
}

#[test]
fn builds_dynamic_library_and_c_header_and_calls_from_c() {
    let root = unique_temp_dir("export_lib");
    let out = root.join("out");

    write_file(
        &root.join("kira.project"),
        r#"
name = "export_lib"
version = "0.1.0"
kind = "library"
entry = "src/lib.kira"
"#
        .trim(),
    );

    write_file(
        &root.join("src/lib.kira"),
        r#"
@Export
struct Vec2 {
    x: float,
    y: float,
}

@Export
func add(a: int, b: int) -> int {
    return a + b;
}

@Export
func dot(a: Vec2, b: Vec2) -> float {
    return a.x * b.x + a.y * b.y;
}
"#
        .trim(),
    );

    let lib_path =
        toolchain::aot::build_library_project(&root, &out).expect("library should build");
    assert!(lib_path.is_file(), "library output should exist");

    let header_path = out.join("export_lib.h");
    let header = fs::read_to_string(&header_path).expect("header should exist");
    assert!(
        header.contains("typedef struct { double x; double y; } Vec2;"),
        "header should contain exported struct"
    );
    assert!(
        header.contains("int64_t add("),
        "header should contain exported function"
    );

    let c_test = root.join("c_test.c");
    write_file(
        &c_test,
        r#"
#include "export_lib.h"

int main(void) {
    if (add(2, 3) != 5) return 1;
    Vec2 a = { 1.0, 2.0 };
    Vec2 b = { 3.0, 4.0 };
    double r = dot(a, b);
    if (r != 11.0) return 2;
    return 0;
}
"#
        .trim(),
    );

    let c_bin = root.join("c_test_bin");
    let mut cmd = Command::new("clang");
    cmd.arg(&c_test)
        .arg(&lib_path)
        .arg(format!("-I{}", out.display()))
        .arg("-o")
        .arg(&c_bin);
    if !cfg!(target_os = "windows") {
        cmd.arg(format!("-Wl,-rpath,{}", out.display()));
    }

    let status = cmd.status().expect("clang should run");
    assert!(status.success(), "C harness should compile");

    let status = Command::new(&c_bin).status().expect("C harness should run");
    assert!(status.success(), "C harness should pass");
}

#[test]
fn link_directive_binds_c_header_and_calls_library() {
    if cfg!(target_os = "windows") {
        // Keeping this test non-Windows for now: the toolchain is primarily exercised on POSIX in CI.
        return;
    }

    let root = unique_temp_dir("link_ffi");
    let out = root.join("out");
    let native_dir = root.join("native");

    let _lib_path = build_c_mylib(&native_dir);

    write_file(
        &root.join("kira.project"),
        r#"
name = "link_ffi"
version = "0.1.0"
entry = "src/main.kira"
"#
        .trim(),
    );

    write_file(
        &root.join("src/main.kira"),
        r#"
@Link("mylib", header: "native/mylib.h")

func main() {
    printIn(add64(2, 3));
    let foo = foo_new();
    printIn(foo_value(foo));
    foo_free(foo);
}
"#
        .trim(),
    );

    let binary =
        toolchain::aot::build_default_project(&root, &out).expect("project should build");
    let output = Command::new(&binary)
        .output()
        .expect("built binary should run");
    assert!(
        output.status.success(),
        "binary should exit successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines, vec!["5", "123"]);
}

#[test]
fn manifest_dependency_header_auto_links_and_binds() {
    if cfg!(target_os = "windows") {
        return;
    }

    let root = unique_temp_dir("manifest_link_ffi");
    let out = root.join("out");
    let native_dir = root.join("native");

    let _lib_path = build_c_mylib(&native_dir);

    write_file(
        &root.join("kira.project"),
        r#"
name = "manifest_link_ffi"
version = "0.1.0"
entry = "src/main.kira"

[dependencies]
mylib = { path = "native/mylib.h" }
"#
        .trim(),
    );

    // No `@Link` directive in source; it should be inferred from manifest dependencies.
    write_file(
        &root.join("src/main.kira"),
        r#"
func main() {
    printIn(add64(2, 3));
    let foo = foo_new();
    printIn(foo_value(foo));
    foo_free(foo);
}
"#
        .trim(),
    );

    let binary =
        toolchain::aot::build_default_project(&root, &out).expect("project should build");
    let output = Command::new(&binary)
        .output()
        .expect("built binary should run");
    assert!(
        output.status.success(),
        "binary should exit successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines, vec!["5", "123"]);
}

