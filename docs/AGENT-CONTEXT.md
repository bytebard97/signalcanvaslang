# Agent Context — SignalCanvasLang

## What Is This Repo

SignalCanvasLang is a PatchLang compiler written in Rust. PatchLang is a domain-specific language for describing signal flow in broadcast/live production (audio, video, intercom systems). The compiler parses `.patch` text files into an AST and serializes to JSON.

It compiles to three targets from one codebase:
- **WASM for browser** (`pkg-web/`) — used by the Vue frontend
- **WASM for Node.js** (`pkg-node/`) — used by CLI tools and tests
- **Python wheel** (`.venv/`) — used by the Django backend

## Repo Layout

```
SignalCanvasLang/
├── crates/patchlang/src/
│   ├── lexer.rs              — Logos-based lexer, keyword tokens
│   ├── parser.rs             — Top-level statement parser
│   ├── body_parser.rs        — Instance body parser (route, bus, slot, config)
│   ├── template_parser.rs    — Template body parser (ports, meta, bridge, slot defs)
│   ├── ast.rs                — Internal Rust AST types
│   ├── compat.rs             — Converts internal AST → TypeScript-compatible types
│   ├── compat_types.rs       — TypeScript-shaped AST structs (camelCase, flat)
│   ├── error.rs              — ParseError type
│   ├── lib.rs                — Public API: parse(), validate(), to_ts_result()
│   ├── parser_tests.rs       — Parser unit tests
│   ├── template_parser_tests.rs
│   └── compat_tests.rs       — Compat layer tests
├── crates/patchlang-wasm/src/
│   └── lib.rs                — WASM bindings (wasm-bindgen exports)
├── crates/patchlang-python/src/
│   └── lib.rs                — Python bindings (PyO3 exports)
├── tests/
│   ├── fixtures/examples/    — Real-world .patch test files
│   ├── fixtures/mtg-features/— Feature-specific test fixtures from Hillsong project
│   └── chevrotain-parity/    — Reference TypeScript compiler for parity testing
├── pkg-node/                 — Built WASM package (Node.js target)
├── pkg-web/                  — Built WASM package (browser/bundler target)
└── docs/
```

## Sibling Repos

- **SignalCanvasFrontend** (`../SignalCanvasFrontend/`) — Vue 3 frontend app
- **SignalCanvasRouter** (`../SignalCanvasRouter/`) — Rust wire routing engine (separate repo)

## Current Keywords (lexer.rs)

`template`, `instance`, `connect`, `bridge`, `bridge_group`, `link_group`, `signal`, `flag`, `stream`, `config`, `ports`, `meta`, `in`, `out`, `io`, `for`, `over`, `generate`, `use`, `slot`, `route`, `bus`, `label`, `suppress`

## Build & Test Commands

```bash
# Run all Rust tests
cargo test -p patchlang

# Run clippy
cargo clippy --all-targets -- -D warnings

# Build WASM (Node.js)
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm

# Build WASM (browser)
wasm-pack build --target bundler --out-dir pkg-web -- --features wasm

# Test via Node.js WASM
node -e "const w = require('./pkg-node/patchlang_wasm.js'); console.log(JSON.parse(w.parse('template A { ports { X: in } }')))"
```

## Architecture Decisions (from Socratic debates)

Read full reasoning in `../SignalCanvasFrontend/docs/REID-TODO-PATCHLANG-MIGRATION.md` (bottom section "Answers to Reid's v0.2.0 Spec Questions").

1. **No `card` keyword.** Cards are just templates with `meta { device_type: "card" }`. No new grammar needed.

2. **`ring` keyword with hybrid member syntax.** `member Console` (implicit port resolution) and `member Console.OptoCore` (explicit port) both accepted. Needs new grammar: `Ring` and `Member` tokens, `RingDecl` AST node, `parse_ring()` function.

3. **Reserve `card` now, not `ring`.** Add `Card` token to lexer immediately (prevents use as identifier). `Ring` gets added when the ring feature is implemented.

4. **`io` direction convention only.** Parser accepts `io` with any protocol — no enforcement. The frontend emitter handles the convention (Dante/MADI → `in`/`out`, OptoCore/TWINLANe → `io`). Update example fixtures to follow convention.

5. **Deterministic IDs via exported utility function.** Add `generate_port_id(instance_name, template_name, port_name, index) -> String` as a pure function exported from the crate. Format: `pl_{templateName}_{portName}[_{index}]`. Export via WASM and Python bindings. Ship conformance tests.

## Code Rules

Follow `/Users/ceres/Desktop/SignalCanvas/ClaudeCodeRules.md`:
- Separate concerns, one responsibility per file/function
- Files under 500 lines (700 max)
- Meaningful tests with edge cases
- DRY, clear names, loose coupling, YAGNI
- No magic numbers, handle errors explicitly
- Read existing code before writing

## DRC (Design Rule Checking) — To Be Built in Rust

The frontend has a 346-line TypeScript DRC at `../SignalCanvasFrontend/src/lang/drc.ts` with 5 validation layers. This needs to be ported to Rust so all platforms get the same checks.

### API Design

```rust
// Parse only — returns AST, may have syntax errors
let result = patchlang::parse(source);

// Parse + DRC — returns AST plus semantic diagnostics
let checked = patchlang::check(source);
```

### Output Shape (JSON via compat layer)

```json
{
  "program": { "statements": [...] },
  "errors": [],
  "diagnostics": [
    {
      "severity": "error",
      "layer": "direction",
      "message": "Invalid connection direction: output -> output",
      "span": { "start": 156, "end": 210 },
      "source": "FOH.Main_Out[1]",
      "target": "Monitor.Main_Out[1]",
      "fix": "One side must be an input port"
    }
  ]
}
```

### Fields

- **`errors`** — syntax errors (already exists, file won't parse correctly)
- **`diagnostics`** — semantic issues (file parses fine but something is wrong or suspicious)
- **`severity`**: `error` (always wrong), `warning` (probably wrong), `info` (suggestion)
- **`layer`**: `direction`, `mechanical`, `electrical`, `logical`, `temporal`
- **`span`**: byte offset range in source text (for editor underlines / Source tab highlighting)
- **`source`** / **`target`**: port references involved (for connect/bridge diagnostics)
- **`fix`**: human-readable suggestion

### DRC Rules to Implement (from TypeScript reference)

**Direction (hard errors):**
- Output → Output connection (always wrong)
- Input → Input connection (always wrong)
- Input → Output connection (backwards)
- Only Output → Input and io connections are valid

**Mechanical (warnings):**
- Connector type mismatch (XLR → BNC, etc.)
- Connector gender issues

**Electrical (warnings):**
- Signal level mismatches (mic level → speaker level without amp)

**Logical (warnings):**
- Protocol mismatches (Dante port connected to MADI port)

**Temporal (info):**
- Clock domain crossings (devices on different word clock sources)

**Structural (errors):**
- Instance referencing non-existent template
- Slot assignment referencing non-existent card template
- Route referencing port that doesn't exist on the device
- Route from input to input or output to output
- Bus output referencing non-existent port
- Channel index beyond port range
- Config block referencing non-existent instance
- Signal origin referencing non-existent port
- Duplicate signal names

### Reference Implementation

Read `../SignalCanvasFrontend/src/lang/drc.ts` for the TypeScript version. Also read `../SignalCanvasFrontend/src/lang/typeCatalog.ts` for connector/protocol compatibility tables used by the DRC.

### WASM/Python Export

The `check()` function should be exported alongside `parse()` and `validate()`:
- WASM: `#[wasm_bindgen] pub fn check(source: &str) -> String` (returns JSON)
- Python: `#[pyfunction] fn check(source: &str) -> String`

## What NOT to Change

- The existing parser grammar for all current keywords — don't break existing `.patch` files
- The compat layer's output shape — frontend depends on the exact JSON structure
- Test fixtures that are passing — don't modify, only add new ones
