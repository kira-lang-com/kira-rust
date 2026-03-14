#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")"

# Build the Kira dynamic library and generated header.
cargo run --quiet --manifest-path ../../toolchain/Cargo.toml -- build --lib

lib=""
if [ "$(uname -s)" = "Darwin" ]; then
  lib="out/export_lib.dylib"
else
  lib="out/export_lib.so"
fi

clang c_test.c "$lib" -Iout -o out/c_test_bin -Wl,-rpath,"$PWD/out"
./out/c_test_bin
echo "OK"

