# Examples

Each folder under `examples/` is a standalone Kira project (it contains a `kira.project`).

Build an example (from the example directory):

```sh
cargo run --manifest-path ../../toolchain/Cargo.toml -- build --bin
./out/<project_name>/<project_name>
```

Build a dynamic library example:

```sh
cargo run --manifest-path ../../toolchain/Cargo.toml -- build --lib
```

## Index

- `hello_world` - minimal program
- `control_flow` - `if`, `while`, `for`, ranges
- `arrays` - arrays, `.append`, `.length`, indexing
- `structs` - nested structs + mutation
- `foundation` - `import Foundation.*` calls
- `math_utils_lib` + `with_deps` - library dependency example
- `link_ffi` - C header auto bindings via `[dependencies] mylib = { path = "native/mylib.h" }`
- `export_lib` - `--lib` output + generated C header + C harness

