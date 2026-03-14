# GitHub Actions Workflows

This document describes the CI/CD workflows configured for the Kira project.

## Workflows Overview

### 1. CI Workflow (`ci.yml`)

**Triggers:**
- Push to any branch (including main, develop, feature branches, etc.)
- Pull requests to any branch

**Jobs:**

#### Check
- Validates that the code compiles
- Runs on Ubuntu with LLVM 17
- Uses cargo caching for faster builds

#### Format
- Ensures code follows Rust formatting standards
- Runs `cargo fmt --all -- --check`
- Fails if code is not properly formatted

#### Clippy
- Runs the Rust linter for code quality
- Runs `cargo clippy --all-features -- -D warnings`
- Treats warnings as errors

#### Test Suite
- Runs all tests on Ubuntu, macOS, and Windows
- Installs LLVM 17 on each platform
- Uses cargo caching for faster test runs

### 2. Build Workflow (`build.yml`)

**Triggers:**
- Push to any branch (including main, develop, feature branches, etc.)
- Pull requests to any branch
- Release creation

**Platforms:**
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- macOS x86_64 (`x86_64-apple-darwin`)
- macOS aarch64 (`aarch64-apple-darwin`)
- Windows x86_64 (`x86_64-pc-windows-msvc`)

**Artifacts:**
- Packaged binaries for each platform
- Uploaded as GitHub Actions artifacts
- Attached to releases when triggered by release events

### 3. Release Workflow (`release.yml`)

**Triggers:**
- Push of version tags (e.g., `v0.1.0`, `v1.2.3`)

**Process:**
1. Creates a GitHub release from the tag
2. Builds optimized binaries for all platforms
3. Strips debug symbols (Unix platforms)
4. Renames binaries to `kira` / `kira.exe`
5. Packages as `.tar.gz` (Unix) or `.zip` (Windows)
6. Generates SHA256 checksums
7. Uploads all assets to the release

**Release Assets:**
- `kira-Linux-x86_64.tar.gz` + `.sha256`
- `kira-Darwin-x86_64.tar.gz` + `.sha256`
- `kira-Darwin-aarch64.tar.gz` + `.sha256`
- `kira-Windows-x86_64.zip` + `.sha256`

## Creating a Release

To create a new release:

1. Update version in `toolchain/Cargo.toml`
2. Commit the version change
3. Create and push a version tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. The release workflow will automatically:
   - Create the GitHub release
   - Build binaries for all platforms
   - Upload release assets

## Local Testing

Before pushing, you can test the build locally:

```bash
.github/scripts/test-build.sh
```

Or manually:

```bash
cd toolchain
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --release
cargo build --release
```

## Caching Strategy

All workflows use GitHub Actions caching for:
- Cargo registry (`~/.cargo/registry`)
- Cargo git index (`~/.cargo/git`)
- Build artifacts (`toolchain/target`)

This significantly speeds up CI runs by reusing dependencies and incremental compilation artifacts.

## LLVM Installation

Each platform installs LLVM 17 differently:

- **Ubuntu**: Uses the official LLVM apt repository script
- **macOS**: Uses Homebrew (`brew install llvm@17`)
- **Windows**: Uses Chocolatey (`choco install llvm`)

The `LLVM_SYS_170_PREFIX` environment variable is set to help the `llvm-sys` crate find the installation.

## Troubleshooting

### Build Fails on Specific Platform

Check the Actions logs for the specific platform. Common issues:
- LLVM not found: Check `LLVM_SYS_170_PREFIX` is set correctly
- Linking errors: Ensure all system dependencies are installed
- Test failures: May be platform-specific bugs

### Release Assets Not Uploaded

Ensure:
- The tag follows the `v*` pattern (e.g., `v0.1.0`)
- The `GITHUB_TOKEN` has sufficient permissions
- The release was created successfully

### Caching Issues

If caching causes problems:
- Clear the cache from the Actions UI
- Update the cache key in the workflow file
- Temporarily disable caching to test
