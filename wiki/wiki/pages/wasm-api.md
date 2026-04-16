---
title: WASM API
tags: [wasm, javascript, typescript, api, browser]
sources: [patchlang-design-guide/compiler, docs/wasm]
updated: 2026-04-16
---

# WASM API

**Source:** `docs/wasm.md`, `docs/patchlang-design-guide/compiler.md`
**Type:** API reference

## Summary

PatchLang compiles to WebAssembly via `wasm-pack`, producing browser (`pkg-web/`) and Node.js (`pkg-node/`) packages. All functions return **JSON strings** — call `JSON.parse()` on results. See [[compiler-architecture]] for the Rust-side implementation.

---

## Building

```bash
./scripts/build-wasm.sh
```

Produces:
- `pkg-web/` — browser bundler target (ES module + `.wasm` binary)
- `pkg-node/` — Node.js target (CommonJS, synchronous loading)

## Browser Setup (Vite)

```typescript
// vite.config.ts
import wasm from 'vite-plugin-wasm'
export default defineConfig({ plugins: [wasm()] })

// Import
import { parse, check, validate } from 'patchlang-wasm'
```

---

## Single-File Functions

```javascript
const parseResult = JSON.parse(parse(source));
// { program: {...}, errors: [...] }

const checkResult = JSON.parse(check(source));
// { program: {...}, errors: [...], diagnostics: [...] }

const isValid = validate(source);   // boolean
const formatted = format_source(source);  // string or JSON error
```

`check()` is the primary API for the editor — runs parse + auto-resolution + DRC.

---

## Multi-File Functions

```javascript
const deps = JSON.parse(resolve_uses(source));
// ["buildings.foh", "yamaha"]

const result = JSON.parse(compile_project(
  JSON.stringify(filesMap),   // { "campus.patch": "...", "buildings/foh.patch": "..." }
  "campus.patch"              // entry file path
));
// { program, errors, diagnostics, files, templateFiles, useGraph }
```

`templateFiles` — template name → source file path (for drill-down navigation).
`useGraph` — file path → dependency list (for sidebar tree).

---

## Manifest and Layout Validation

```javascript
const manifest = JSON.parse(parse_manifest(projectJsonString));
// { manifest: {...} | null, errors: [...] }

const layoutResult = JSON.parse(validate_layout(layoutJson));
// { valid: bool, errors: [...] }

const consistency = JSON.parse(validate_project_consistency(patchSource, layoutJson));
// { valid: bool, errors: [...], warnings: [...] }
```

---

## ID Generation

```javascript
// Pass -1 for scalar ports (NO_INDEX)
const portId = generate_port_id("Console", "CL5", "Dante_In", 1);
// "pl::CL5::Dante_In_1"

const portIdScalar = generate_port_id("Console", "CL5", "Dante_In", -1);
// "pl::CL5::Dante_In"

const routeId = generate_route_id("CL5", "Mic_In", "Dante_Out");
// "rule::CL5::Mic_In::Dante_Out"

const slotId = generate_slot_id("CL5", "MY_Slot");
// "slot::CL5::MY_Slot"
```

`generate_port_id` uses `i32` for optional indices (WASM limitation — `Option<u32>` not supported). Pass `-1` for "no index".

---

## Builder API (Handle-Based)

The builder API avoids emitting PatchLang text in TypeScript. Instead, call Rust mutations via WASM handles — validation happens in Rust at build time.

```javascript
const handle = create_program();                          // new empty builder
const h2 = JSON.parse(create_program_from_source(src));  // from existing .patch
const source = format_program(handle);                    // → .patch text
const json = get_program_json(handle);                    // → AST JSON
const diags = check_program(handle);                      // → diagnostics JSON
free_program(handle);                                     // release memory
```

**Mutations** (all return `{"ok": true}` or `{"error": "..."}` JSON):

```javascript
add_template(handle, templateJson);
remove_template(handle, name);
add_instance(handle, instanceJson);
remove_instance(handle, name);          // → CascadeResult JSON
add_connect(handle, sourceJson, targetJson, propsJson);  // → {"ok":true,"id":"..."}
remove_connect(handle, id);
set_slot(handle, instance, slotName, slotIndex, cardTemplate);  // slotIndex: -1 = None
add_route(handle, instance, fromPort, fromCh, toPort, toCh);
set_routes(handle, instance, routesJson);
add_bus(handle, instance, busJson);
set_label(handle, instance, port, index, label, propsJson);
add_signal(handle, signalJson);
add_stream(handle, streamJson);
add_flag(handle, flagJson);
add_ring(handle, ringJson);
add_ring_member(handle, ringName, instance, port);  // empty port = None
add_bridge(handle, sourceJson, targetJson);
```

**Handle lifecycle:** handles are indices into `Vec<Option<PatchProgramBuilder>>`. Freed slots are reused.

---

## Error Format

```json
{
  "message": "expected ']' to close port range",
  "span": { "start": 47, "end": 48 },
  "hint": "port ranges must be closed with ']'"
}
```

Use span byte offsets to compute line/column numbers or highlight ranges in an editor.

---

## Relation to Other Wiki Pages

- [[compiler-architecture]] — Rust implementation behind these functions
- [[builder-api]] — full builder API reference
- [[python-api]] — Python equivalent
- [[frontend-integration]] — how the frontend uses these functions during project loading
