#!/usr/bin/env bash
# Build the Windows GNU DLL with the Rustup-managed cross target. Run inside
# `nix-shell`; shell.nix supplies MinGW-w64 and project-local RUSTUP_HOME.
set -euo pipefail

: "${RUSTUP_HOME:?Run this from nix-shell so RUSTUP_HOME is configured}"
toolchain="1.93.1-x86_64-unknown-linux-gnu"
toolchain_bin="$RUSTUP_HOME/toolchains/$toolchain/bin"
if [[ ! -x "$toolchain_bin/cargo" ]]; then
  echo "missing Rustup toolchain: $toolchain" >&2
  exit 1
fi
# Nix's MinGW compiler omits the conventional libpthread.a name expected by
# Rust's GNU target. The bootstrap shim does not use pthread symbols, so an
# empty archive is sufficient until the proper target runtime is packaged.
mkdir -p .nix-mingw
x86_64-w64-mingw32-ar rcs .nix-mingw/libpthread.a
export RUSTFLAGS="-L native=$PWD/.nix-mingw${RUSTFLAGS:+ $RUSTFLAGS}"

PATH="$toolchain_bin:$PATH" "$toolchain_bin/cargo" build --target x86_64-pc-windows-gnu "$@"
PATH="$toolchain_bin:$PATH" "$toolchain_bin/cargo" build \
  --manifest-path injector/Cargo.toml \
  --target-dir "$PWD/target" \
  --target x86_64-pc-windows-gnu "$@"
x86_64-w64-mingw32-gcc -municode -O2 -s injector/launcher.c \
  -o target/x86_64-pc-windows-gnu/debug/ovr-loader-launcher.exe
