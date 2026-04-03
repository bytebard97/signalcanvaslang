#!/usr/bin/env bash
# Build PatchLang WASM packages (web + Node.js targets)
# Output goes to pkg-web/ and pkg-node/ at the repo root,
# which is where the frontend's Vite config expects them.
set -euo pipefail

cd "$(dirname "$0")/.."

# Ensure rustup's toolchain is used (not Homebrew's)
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
echo "Building patchlang-wasm v${VERSION}..."

echo "  → web target (pkg-web/)..."
wasm-pack build crates/patchlang-wasm --target web --release --out-dir ../../pkg-web

echo "  → nodejs target (pkg-node/)..."
wasm-pack build crates/patchlang-wasm --target nodejs --release --out-dir ../../pkg-node

echo "  → bundler target (pkg-bundler/)..."
wasm-pack build crates/patchlang-wasm --target bundler --release --out-dir ../../pkg-bundler

echo "Done — v${VERSION} built to pkg-web/, pkg-node/, pkg-bundler/"
