![Kira Banner](Images/KiraBanner.png)

# Kira

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

### Prerequisites

- Rust (latest stable)
- LLVM 17
- `clang` and `libtool` (macOS: included with Xcode Command Line Tools)

### Build
```bash
git clone https://github.com/kira-lang-com/kira
cd kira
cargo build --release
```

### Run a project
```bash
kira run    # compile and run via VM
kira build  # compile to a native binary in out/
```

### Project structure
```
my_app/
├── kira.project     # project manifest
└── main.kira        # entry point
```

`kira.project`:
```
name = "my_app"
version = "0.1.0"
entry = "main.kira"
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
- 🚧 Closures
- 🚧 Enums
- 🚧 String interpolation
- 🚧 Package manager

## License

Apache 2.0 with Runtime Library Exception — see [LICENSE](LICENSE).

You can use Kira to build and distribute your own applications without 
attribution requirements.
