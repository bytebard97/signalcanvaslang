#!/usr/bin/env bash
# Build PatchLang Python wheel via maturin
set -euo pipefail

cd "$(dirname "$0")/.."

# Create venv if it doesn't exist
if [ ! -d ".venv" ]; then
  python3 -m venv .venv
fi

source .venv/bin/activate

# Forward compat for Python 3.14+
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

echo "Building Python wheel..."
cd crates/patchlang-python
maturin develop

echo "Done — patchlang_python installed in .venv"
