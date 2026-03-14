<picture>
  <source media="(prefers-color-scheme: dark)" srcset="Images/KiraBannerDark.png">
  <source media="(prefers-color-scheme: light)" srcset="Images/KiraBannerLight.png">
  <img alt="Kira" src="Images/KiraBannerDark.png">
</picture>

# Kira

[![Build](https://github.com/kira-lang-com/kira/workflows/Build/badge.svg)](https://github.com/kira-lang-com/kira/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

> **Note:** Release builds run on version tags (e.g., `v1.0.0`). See [build status](.github/STATUS.md) for details.

A dual-mode compiled programming language — written in Rust.

Kira functions can run as **native machine code** (via LLVM) or as **interpreted 
bytecode** (via a built-in VM), in the same binary. You choose per function, 
or let the compiler decide automatically.

## Why Kira?

Most languages make you choose: compiled *or* interpreted. Kira gives you both, 
at the function level:

- `@Native` → compiled to real machine code via LLVM. Maximum performance.
- `@Runtime` → runs on Kira's bytecode VM. Hot-reloadable, dynamic, flexible.
- `Auto` → the compiler decides based on what the function does.

## Quick Look
```kira
struct Vec2 {
    x: int,
    y: int,
}

struct Player {
    name: string,
    position: Vec2,
    health: int,
}

@Runtime
func move_player(player: Player, dx: int, dy: int) -> Player {
    player.position.x = player.position.x + dx;
    player.position.y = player.position.y + dy;
    return player;
}

@Runtime
func distance_squared(a: Vec2, b: Vec2) -> int {
    let dx: int = b.x - a.x;
    let dy: int = b.y - a.y;
    return dx * dx + dy * dy;
}

func main() {
    let start: Vec2 = Vec2 { x: 1, y: 2 };
    let target: Vec2 = Vec2 { x: 4, y: 6 };
    let hero: Player = Player { name: "Kira", position: start, health: 100 };
    let moved: Player = move_player(hero, 3, 4);

    printIn(hero.position.x);                  // 1  — value semantics
    printIn(moved.position.x);                 // 4
    printIn(distance_squared(moved.position, target)); // 0
}
```

## Features

- Dual execution model — Native (LLVM AOT) and Runtime (bytecode VM) in one binary
- First-class platform targeting via `#platforms` and `@Platforms`
- Strong static type system — `int`, `float`, `bool`, `string`
- Strict type separation — no implicit coercion, explicit `float()` casts
- Structs with nested fields and value semantics in `@Runtime` code
- Arrays with `for in` loops and ranges
- Recursive functions
- Foundation standard library (`import Foundation.Math`, etc.)
- Multi-file projects with implicit global scope
- Zed editor syntax highlighting

For local Zed dev-extension installs, Tree-sitter grammar compilation also requires
`emcc`, `docker`, or `podman` to be available on `PATH`.

## Getting Started

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases page](https://github.com/kira-lang-com/kira/releases):

- **Linux**: `kira-Linux-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `kira-Darwin-aarch64.tar.gz`
- **Windows**: `kira-Windows-x86_64.zip`

Extract and add to your PATH, or use the toolchain installer (see below).

### Build from Source

#### Prerequisites

- Rust (latest stable)
- LLVM 17
- `clang` and `libtool` (macOS: included with Xcode Command Line Tools)

#### Build
```bash
git clone https://github.com/kira-lang-com/kira
cd kira/toolchain
cargo build --release
cp target/release/toolchain ../kira
```

Or use the built-in toolchain installer:
```bash
cd kira
./kira toolchain install --dev
./kira toolchain path  # shows how to add to PATH
```

### CLI Commands

```bash
kira new my_app              # scaffold a new Kira project
kira build                   # compile to native binary in out/
kira run                     # build and run immediately
kira check                   # type-check without compiling
kira clean                   # remove out/ and build artifacts
kira package                 # package a library for distribution
kira fetch                   # fetch and cache project dependencies
kira add <name>              # add a dependency to the project
kira version                 # print Kira version
kira toolchain install --dev # build and install development toolchain
kira toolchain list          # list installed toolchains
kira toolchain path          # show PATH configuration
```

### Create a new project
```bash
kira new my_app
cd my_app
kira run
```

This scaffolds:
```
my_app/
├── kira.project     # project manifest
└── src/
    └── main.kira    # entry point with Hello, Kira!
```

`kira.project`:
```
name = "my_app"
version = "0.1.0"
entry = "src/main.kira"
```

## Execution Modes

| Annotation | Backend | Use for |
|---|---|---|
| `@Native` | LLVM AOT | Performance-critical code |
| `@Runtime` | Bytecode VM | Dynamic logic, hot-reload |
| *(none)* | Auto | Let the compiler decide |

## Platform System
```kira
#platforms {
    mobile = [ios, android];
    desktop = [macos, windows, linux];
}

@Platforms(mobile)
func haptics() { ... }  // only compiled for mobile targets
```

## Foundation Library
```kira
import Foundation.Math;
import Foundation.String;

func main() {
    printIn(Math.sqrt(16.0));
    printIn(String.concat("Hello", " Kira!"));
}
```

## FFI (Foreign Function Interface)

Kira can link with C libraries and call native functions:

```kira
@Link(library: "mylib", header: "native/mylib.h")

func main() {
    let result: int = add_numbers(5, 3);
    printIn(result);  // 8
}
```

The `@Link` directive:
- Automatically parses C headers
- Generates type-safe bindings
- Handles native library loading at runtime
- Supports platform-specific builds

See `examples/link_ffi/` for a complete example.

## Library System

Kira supports creating and using libraries for code reuse across projects.

### Creating a Library

Set `kind = "library"` in your `kira.project`:

```
name = "math_utils"
version = "1.0.0"
kind = "library"
entry = "src/lib.kira"

[authors]
author = "Your Name"
```

Package your library:
```bash
kira package
```

This creates a `.kpkg` file in the `out/` directory.

### Using Libraries

Add dependencies to your `kira.project`:

```
name = "my_app"
version = "0.1.0"
entry = "src/main.kira"

[dependencies]
math_utils = { version = "1.0.0", path = "../math_utils" }
```

Fetch dependencies:
```bash
kira fetch
```

Import and use in your code:
```kira
import math_utils;

func main() {
    let result: int = math_utils.square(5);
    printIn(result);  // 25
}
```

### Dependency Sources

- **Registry**: `math_utils = "1.0.0"` (future feature)
- **Local path**: `math_utils = { version = "1.0.0", path = "../math_utils" }`
- **Git**: `math_utils = { version = "1.0.0", git = "https://github.com/user/math_utils" }` (future feature)

## Status

Kira is in early development. The following works today:

- ✅ Parser and AST
- ✅ Type system
- ✅ Bytecode VM
- ✅ LLVM AOT backend
- ✅ Native/VM bridge
- ✅ Arrays and loops
- ✅ Multi-file projects
- ✅ Foundation standard library
- ✅ Zed syntax highlighting
- ✅ Structs in `@Runtime`
- ✅ Library system with dependencies
- ✅ FFI (Foreign Function Interface) for C libraries
- 🚧 Closures
- 🚧 Enums
- 🚧 String interpolation
- 🚧 Package registry

## Codebase Architecture

The Kira toolchain is built with maintainability and clarity in mind:

- **150 source files**, 96.6% under 275 lines each
- **Single responsibility principle** enforced throughout
- **Organized folder structure** with max 9 files per folder
- **Clean separation** between production code (`src/`) and tests (`tests/`)
- **Well-documented** with inline comments and architectural guidelines

### Key Modules

- `toolchain/src/parser/` - Chumsky-based parser with core parsing logic
- `toolchain/src/compiler/` - Type checking, lowering, and compilation
- `toolchain/src/runtime/` - Bytecode VM and execution engine
- `toolchain/src/aot/` - LLVM-based ahead-of-time compilation
- `toolchain/src/project/` - Project management and dependency resolution
- `toolchain/tests/` - Integration tests for all major components

See [AGENTS.md](AGENTS.md) for detailed architectural guidelines.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

### Development Setup

1. Clone the repository
2. Install Rust (latest stable) and LLVM 17
3. Build the toolchain: `cd toolchain && cargo build`
4. Run tests: `cargo test`
5. Check code: `cargo check`

### Code Quality Standards

- Files must be under 275 lines
- Each file has a single, clear responsibility
- Folders contain max 9 files (target: 7)
- Tests go in `tests/`, not `src/`
- Follow the guidelines in [AGENTS.md](AGENTS.md)

## License

Apache 2.0 with Runtime Library Exception — see [LICENSE](LICENSE).

You can use Kira to build and distribute your own applications without 
attribution requirements.
