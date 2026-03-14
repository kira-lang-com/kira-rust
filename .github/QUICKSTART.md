# Quick Start Guide for Contributors

This guide will help you get started with Kira development in minutes.

## 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/kira-lang-com/kira
cd kira

# Build the toolchain
cd toolchain
cargo build --release

# Install locally
cp target/release/toolchain ../kira
chmod +x ../kira
cd ..
```

## 2. Verify Installation

```bash
# Check version
./kira version

# Create a test project
./kira new hello_kira
cd hello_kira

# Run it
../kira run
```

You should see: `Hello, Kira!`

## 3. Development Workflow

### Make Changes

Edit files in `toolchain/src/`

### Test Your Changes

```bash
cd toolchain

# Format code
cargo fmt

# Check for issues
cargo clippy

# Run tests
cargo test

# Build
cargo build --release
```

### Use the Test Script

```bash
# From the repository root
.github/scripts/test-build.sh
```

This runs all checks automatically.

## 4. Project Structure

```
kira/
├── .github/          # CI/CD workflows and templates
├── app/              # Example Kira application
├── toolchain/        # The Kira compiler and toolchain
│   ├── src/
│   │   ├── aot/      # Ahead-of-time compilation (LLVM)
│   │   ├── ast/      # Abstract syntax tree
│   │   ├── cli/      # Command-line interface
│   │   ├── compiler/ # Compilation logic
│   │   ├── library/  # Standard library (Foundation)
│   │   ├── parser/   # Parser implementation
│   │   ├── project/  # Project management
│   │   └── runtime/  # Bytecode VM
│   └── Cargo.toml
└── README.md
```

## 5. Common Tasks

### Add a New CLI Command

Edit `toolchain/src/cli/mod.rs`:

1. Add to `Commands` enum
2. Add handler function
3. Update match statement in `run()`

### Add a Standard Library Function

Edit files in `toolchain/src/library/foundation/`:

1. Add function to appropriate module
2. Register in `mod.rs`
3. Add tests

### Fix a Bug

1. Write a failing test
2. Fix the bug
3. Verify test passes
4. Run full test suite

## 6. Submitting Changes

```bash
# Create a branch
git checkout -b fix/my-bug-fix

# Make changes and commit
git add .
git commit -m "Fix: description of fix"

# Push to your fork
git push origin fix/my-bug-fix
```

Then open a Pull Request on GitHub.

## 7. CI/CD

When you push or open a PR, GitHub Actions will automatically run on every commit to any branch:

- ✓ Check code formatting
- ✓ Run clippy linter
- ✓ Run all tests on Linux, macOS, and Windows
- ✓ Build release binaries

All checks must pass before merging. You can see the status of your builds in the Actions tab or on your PR.

## 8. Getting Help

- Read [CONTRIBUTING.md](.github/CONTRIBUTING.md) for detailed guidelines
- Check [WORKFLOWS.md](.github/WORKFLOWS.md) for CI/CD info
- Open an issue for questions or bugs
- Join discussions in GitHub Discussions

## 9. Useful Commands

```bash
# Format all code
cargo fmt --all

# Check without building
cargo check

# Run specific test
cargo test test_name

# Build with verbose output
cargo build --release --verbose

# Clean build artifacts
cargo clean

# Update dependencies
cargo update
```

## 10. Tips

- Use `cargo watch` for automatic rebuilds during development
- Run `cargo clippy --fix` to auto-fix some issues
- Use `RUST_BACKTRACE=1` for detailed error traces
- Check the `app/` directory for example Kira code

Happy coding! 🚀
