# Contributing to Kira

Thank you for your interest in contributing to Kira! This document provides guidelines and information about our development process.

## Development Setup

### Prerequisites

- Rust (latest stable)
- LLVM 17
- `clang` and `libtool` (macOS: included with Xcode Command Line Tools)

### Building from Source

```bash
git clone https://github.com/kira-lang-com/kira
cd kira/toolchain
cargo build --release
cp target/release/toolchain ../kira
```

Or use the toolchain installer:

```bash
cd kira
./kira toolchain install --dev
```

## CI/CD Pipeline

Kira uses GitHub Actions for continuous integration and deployment:

### CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and pull request to any branch:

- **Check**: Validates the code compiles
- **Format**: Ensures code follows Rust formatting standards (`cargo fmt`)
- **Clippy**: Runs the Rust linter for code quality
- **Test Suite**: Runs all tests on Ubuntu, macOS, and Windows

### Build Workflow (`.github/workflows/build.yml`)

Builds release binaries for all platforms:

- Linux x86_64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

Artifacts are uploaded for each build and can be downloaded from the Actions tab.

### Release Workflow (`.github/workflows/release.yml`)

Triggered when a version tag is pushed (e.g., `v0.1.0`):

1. Creates a GitHub release
2. Builds optimized binaries for all platforms
3. Strips debug symbols (Unix)
4. Packages binaries as archives
5. Generates SHA256 checksums
6. Uploads all assets to the release

## Making Changes

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix any warnings
- Ensure all tests pass with `cargo test`

### Commit Messages

Use clear, descriptive commit messages:

```
Add feature: brief description

Longer explanation of what changed and why, if needed.
```

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit your changes
6. Push to your fork
7. Open a Pull Request

All PRs must pass CI checks before merging.

## Testing

Run the test suite:

```bash
cd toolchain
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

## Creating a Release

Maintainers can create a release by pushing a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers the release workflow which builds and publishes binaries for all platforms.

## Questions?

Feel free to open an issue for questions, bug reports, or feature requests.
