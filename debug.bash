cd toolchain
if [[ "$@" == *"--debug"* ]]; then
  cargo build
  cp target/debug/toolchain ../kira
elif [[ "$@" == *"--release"* ]]; then
  cargo build --release
  cp target/release/toolchain ../kira
else
  cargo build
  cp target/debug/toolchain ../kira
fi
cd ..
