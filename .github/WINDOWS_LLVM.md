# Windows LLVM Build Cache

## Overview

Windows builds require LLVM to be built from source because pre-built Windows binaries don't include `llvm-config.exe`, which is required by the `llvm-sys` Rust crate.

Building LLVM from source takes approximately 30-40 minutes, so we use GitHub Actions cache to avoid rebuilding it on every commit.

## Initial Setup

Before Windows builds can work, you need to run the LLVM build workflow once to populate the cache:

1. Go to the [Actions tab](../../actions)
2. Select "Build LLVM for Windows" workflow
3. Click "Run workflow" → "Run workflow"
4. Wait ~30-40 minutes for the build to complete

Once complete, the LLVM installation will be cached and reused by all subsequent builds.

## Cache Details

- **Cache Key**: `llvm-17.0.6-windows-msvc-<hash>`
- **Cache Path**: `C:\LLVM-17`
- **LLVM Version**: 17.0.6
- **Build Configuration**: MinSizeRel (optimized for size)
- **Targets**: X86 only (sufficient for x86_64-pc-windows-msvc)

## Cache Maintenance

The cache is automatically maintained:

- **Expiration**: GitHub caches expire after 7 days of no use
- **Refresh**: The workflow runs monthly (1st of each month) to keep the cache fresh
- **Manual Refresh**: You can manually trigger the workflow anytime to rebuild

## Troubleshooting

### Build fails with "LLVM cache not found"

This means the LLVM build workflow hasn't been run yet. Follow the Initial Setup steps above.

### Cache is stale or corrupted

1. Go to repository Settings → Actions → Caches
2. Delete the `llvm-17.0.6-windows-msvc-*` cache
3. Re-run the "Build LLVM for Windows" workflow

### Build takes too long

The first LLVM build takes 30-40 minutes. Subsequent builds using the cache should only take a few minutes for the actual Kira compilation.

## Technical Details

The LLVM build uses:
- **Compiler**: MSVC (Visual Studio 2022)
- **Build System**: Ninja
- **CMake Options**:
  - `CMAKE_BUILD_TYPE=MinSizeRel` - Optimized for size
  - `LLVM_TARGETS_TO_BUILD=X86` - Only X86 target
  - `LLVM_INCLUDE_TESTS=OFF` - Skip tests
  - `LLVM_INCLUDE_EXAMPLES=OFF` - Skip examples
  - `LLVM_BUILD_TOOLS=ON` - Include llvm-config and other tools

This configuration produces a minimal LLVM installation with everything needed for llvm-sys while keeping build times and cache size reasonable.
