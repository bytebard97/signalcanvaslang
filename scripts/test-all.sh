#!/usr/bin/env bash
# Run all tests: Rust unit tests, WASM smoke tests, Python smoke tests
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== Rust tests ==="
cargo test -p patchlang

echo ""
echo "=== WASM tests ==="
node tests/test_wasm.mjs

echo ""
echo "=== Python tests ==="
source .venv/bin/activate 2>/dev/null || { echo "SKIP: no .venv (run scripts/build-python.sh first)"; exit 0; }
python tests/test_python.py

echo ""
echo "All tests passed!"
