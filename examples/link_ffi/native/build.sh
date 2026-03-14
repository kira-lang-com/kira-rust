#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")"

if [ "$(uname -s)" = "Darwin" ]; then
  out="libmylib.dylib"
  clang -dynamiclib -install_name "@rpath/$out" -o "$out" mylib.c
else
  out="libmylib.so"
  clang -shared -fPIC -o "$out" mylib.c
fi

echo "built $out in $(pwd)"

