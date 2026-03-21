---
layout: default
title: Installation
---

# Installation

PatchLang compiles to five targets from a single Rust codebase. Pick the one that fits your project.

## Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
patchlang = { git = "https://github.com/ByteBard97/SignalCanvasLang" }
```

```rust
let result = patchlang::parse(source);
if result.is_valid() {
    // result.program contains the AST
}
```

## CLI

```bash
git clone https://github.com/ByteBard97/SignalCanvasLang
cd SignalCanvasLang
cargo install --path crates/patchlang-cli
```

```bash
patchlang my-venue.patch          # parse and output JSON AST
echo 'instance FOH is CL5' | patchlang   # pipe from stdin
```

## WebAssembly (Browser / Node.js)

Build the WASM packages:

```bash
./scripts/build-wasm.sh
```

This produces:
- `pkg-node/` -- Node.js target (synchronous, for CLI and tests)
- `pkg-web/` -- Browser bundler target (for Vite, webpack, etc.)

```javascript
import { parse, validate } from './pkg-web/patchlang_wasm.js'

const result = JSON.parse(parse(source))
console.log(result.program)   // PatchProgram AST
console.log(result.errors)    // parse errors (empty if valid)

validate(source)  // returns boolean
```

### Vite Configuration

If you're using Vite, add `vite-plugin-wasm` and configure the alias:

```javascript
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [wasm()],
  resolve: {
    alias: {
      'patchlang-wasm': '/path/to/SignalCanvasLang/pkg-web/patchlang_wasm.js',
    },
  },
})
```

## Python

Build the Python wheel:

```bash
./scripts/build-python.sh
```

```python
import patchlang_python

result = patchlang_python.parse(source)   # returns JSON string
valid = patchlang_python.validate(source) # returns bool
```

### Django Integration

The Python bindings are designed for server-side `.patch` file validation in Django:

```python
from django.core.exceptions import ValidationError
import json
import patchlang_python

def validate_patch_content(content: str) -> dict:
    result = json.loads(patchlang_python.parse(content))
    if result['errors']:
        raise ValidationError(result['errors'][0]['message'])
    return result['program']
```

## Prerequisites

- **Rust** toolchain (rustup recommended)
- **wasm-pack** for WASM builds: `cargo install wasm-pack`
- **maturin** for Python builds: `pipx install maturin`
- **wasm32-unknown-unknown** target: `rustup target add wasm32-unknown-unknown`
