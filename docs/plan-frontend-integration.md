# Plan: Replace Chevrotain with WASM Parser in Frontend

## Goal
Replace the TypeScript Chevrotain parser (lexer.ts, parser.ts, visitor.ts, cstTypes.ts) with the Rust WASM parser from SignalCanvasLang, keeping the existing `PatchProgram` AST types and all downstream consumers unchanged.

## Current State
- **SignalCanvasLang** — Rust parser with WASM build, 105 tests passing, published at ByteBard97/SignalCanvasLang
- **Frontend** — Chevrotain parser in `src/lang/`, 17 files import from it
- **Emitter fix** — pushed on `fix/emitter-empty-bus-output` branch

## Strategy: Option A — Fix Serde Output in Rust

Make the WASM `parse()` function output JSON that exactly matches the existing TypeScript `PatchProgram` type. No adapter layer in TS.

### Phase 1: Rust AST Serde Alignment

Create a `serde_compat` module in the `patchlang` crate with custom serializers that transform the internal Rust AST into the shape TypeScript expects.

**Transformations needed:**

| Rust internal | TS expected | Serializer change |
|---|---|---|
| All struct fields | camelCase | `#[serde(rename_all = "camelCase")]` on all structs |
| `TemplateDecl.meta: Vec<KeyValue>` | `Record<string, string>` | Custom serializer: array → object |
| `InstanceDecl.args: Vec<KeyValue>` | `Record<string, number \| string>` | Custom serializer: array → object |
| `InstanceDecl.properties: Vec<KeyValue>` | `Record<string, string>` | Custom serializer: array → object |
| `Signal/Flag/Stream.properties` | `Record<string, string>` | Same as above |
| `ParamDef.default_value: ParamValue` | `number \| string` | Custom serializer: unwrap enum |
| `PortDef.range: Option<RangeSpec>` | `rangeStart?, rangeEnd?` | Custom serializer: flatten |
| `PortDef.direction: PortDirection` | `"in" \| "out" \| "io"` | `#[serde(rename_all = "lowercase")]` |
| `PortRef.instance: Option<String>` | `instance: string` (empty for local) | Default to `""` |
| `PortRef.index: Option<IndexSpec>` | `indexSpec?: IndexElement[]` | Flatten, rename |
| `ConnectDecl.suppressions: Vec<String>` | `Suppression { layers: [] }` | Wrap in object |
| `ConnectDecl.mapping: Option<String>` | `MappingSpec` (parsed) | Parse in Rust |
| All `span` fields | absent | `#[serde(skip)]` |
| `InstanceDecl.template_name` | `templateName` | camelCase rename handles it |
| `Statement` enum tag | `type: "Template"` etc. | Already using `#[serde(tag = "type")]` |

**Approach:** Create a parallel `compat` module with TS-shaped structs and `From<InternalType>` conversions. The WASM entry point serializes the compat types, not the internal types. Internal types stay clean for CLI/Python use.

### Phase 2: WASM Entry Point

Update `patchlang-wasm/src/lib.rs` to:
1. Parse source → internal AST
2. Convert internal AST → compat AST
3. Serialize compat AST → JSON
4. Return JSON string

### Phase 3: Frontend Integration

1. Add dependency: `"patchlang": "file:../../SignalCanvasLang/pkg-web"` in package.json
2. Create `src/lang/wasmParser.ts` — thin wrapper:
   ```typescript
   import init, { parse as wasmParse } from 'patchlang'
   export function compile(text: string): PatchProgram {
     const result = JSON.parse(wasmParse(text))
     if (result.errors.length > 0) throw new Error(result.errors[0].message)
     return result.program
   }
   ```
3. Update `src/lang/visitor.ts` to re-export from wasmParser instead of Chevrotain
4. Keep Chevrotain code for parity testing (parse both, diff ASTs)
5. Once parity proven, delete lexer.ts, parser.ts, cstTypes.ts, visitor.ts
6. Remove `chevrotain` from package.json

### Phase 4: Parity Testing

Create `src/lang/__tests__/wasmParity.test.ts`:
- For each .patch fixture, parse with both Chevrotain and WASM
- Deep-compare the resulting PatchProgram ASTs
- Any difference is a bug to fix before switching over

### Phase 5: Cleanup

- Delete Chevrotain files (lexer.ts, parser.ts, cstTypes.ts, old visitor.ts code)
- Remove chevrotain from package.json
- Update CLAUDE.md to reference WASM parser
- Update npm scripts

## Files to Create/Modify

### In SignalCanvasLang:
- `crates/patchlang/src/compat.rs` — TS-compatible AST types with From conversions
- `crates/patchlang-wasm/src/lib.rs` — use compat types for serialization
- Tests verifying compat output matches TS expectations

### In SignalCanvasFrontend:
- `package.json` — add patchlang dependency, remove chevrotain (eventually)
- `src/lang/wasmParser.ts` — new WASM wrapper
- `src/lang/visitor.ts` — swap implementation
- `src/lang/__tests__/wasmParity.test.ts` — parity tests

## Agent Decomposition (for parallel execution)

1. **Agent: Rust compat module** — build `compat.rs` with all type transformations and From impls, update WASM entry point
2. **Agent: Frontend integration** — add dependency, create wasmParser.ts, wire up visitor.ts
3. **Agent: Parity tests** — write tests that compare Chevrotain vs WASM output on all fixtures

Agents 1 and 2 can run in parallel. Agent 3 runs after both complete.
