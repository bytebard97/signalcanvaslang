#!/usr/bin/env bash
# Build PatchLang WASM packages (Node.js + bundler targets)
set -euo pipefail

cd "$(dirname "$0")/.."

# Ensure rustup's toolchain is used (not Homebrew's)
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

echo "Building Node.js target..."
wasm-pack build crates/patchlang-wasm --target nodejs --out-dir ../../pkg-node

echo "Building bundler target..."
wasm-pack build crates/patchlang-wasm --target bundler --out-dir ../../pkg-web

echo "Done — packages at pkg-node/ and pkg-web/"
