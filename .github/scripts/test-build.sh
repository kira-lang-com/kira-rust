#!/bin/bash
set -e

echo "Testing Kira build locally..."
echo ""

# Check prerequisites
echo "Checking prerequisites..."
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found. Install Rust first."; exit 1; }
command -v clang >/dev/null 2>&1 || { echo "Error: clang not found. Install LLVM first."; exit 1; }

echo "✓ Prerequisites found"
echo ""

# Build
echo "Building toolchain..."
cd toolchain
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo ""
    echo "Binary location: toolchain/target/release/toolchain"
    echo ""
    echo "To install:"
    echo "  cp target/release/toolchain ../kira"
    echo "  chmod +x ../kira"
else
    echo ""
    echo "✗ Build failed"
    exit 1
fi

# Run tests
echo ""
echo "Running tests..."
cargo test --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ All tests passed!"
else
    echo ""
    echo "✗ Tests failed"
    exit 1
fi

echo ""
echo "✓ All checks passed! Ready to commit."
