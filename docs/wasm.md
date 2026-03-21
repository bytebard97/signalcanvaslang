---
layout: default
title: WebAssembly Integration
---

# WebAssembly Integration

PatchLang compiles to WebAssembly via [wasm-pack](https://rustwasm.github.io/wasm-pack/), producing packages for both browser bundlers (Vite, webpack) and Node.js.

## Building

```bash
./scripts/build-wasm.sh
```

Produces:
- `pkg-web/` — browser bundler target (ES module + `.wasm` binary)
- `pkg-node/` — Node.js target (CommonJS, synchronous loading)

## Browser Usage (Vite)

```javascript
// vite.config.ts
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

```typescript
// src/parser.ts
import { parse, validate } from 'patchlang-wasm'

const result = JSON.parse(parse(source))
// result.program  — PatchProgram AST (matches TypeScript types)
// result.errors   — parse errors with byte-offset spans
```

## Node.js Usage

```javascript
const { parse, validate } = require('./pkg-node/patchlang_wasm.js')

const result = JSON.parse(parse(source))
console.log(result.program.statements.length)
```

## API

### `parse(source: string): string`

Parses PatchLang source and returns a JSON string:

```json
{
  "program": {
    "type": "Program",
    "statements": [...]
  },
  "errors": []
}
```

The `program` field matches the TypeScript `PatchProgram` type used by the SignalCanvas frontend. All field names are camelCase, properties are `Record<string, string>` objects, and port directions are lowercase strings.

If there are parse errors, the parser recovers and produces a partial AST alongside the error list. Each error includes byte-offset spans for precise source location.

### `validate(source: string): boolean`

Returns `true` if the source parses without errors. Faster than `parse()` when you only need a yes/no answer.

## Error Format

```json
{
  "message": "expected ']' to close port range",
  "span": { "start": 47, "end": 48 },
  "hint": "port ranges must be closed with ']'"
}
```

Use the span offsets to compute line/column numbers or highlight ranges in an editor.
