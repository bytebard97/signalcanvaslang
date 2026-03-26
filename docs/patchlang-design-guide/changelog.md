## Revision History

### v0.2.5 — 2026-03-26

- **Ring keyword fully implemented.** Lexer, parser, AST, and DRC rules R01-R04 all complete. Ring added to language specification (EBNF, keywords, examples). Compatibility layer bridges old and new ring formats. Parser error hints updated to include `ring`.
- **Convention DRC checks added.** Four new rules: C01 orphaned devices, C02 duplicate connections, C03 zero-port templates, C04 empty buses.
- **Auto-channel assignment.** `[auto]` index syntax with error codes A01-A03 for conflicts and exhaustion.
- **Multi-file project compilation.** `compile_project()`, `resolve_uses()`, and `parse_manifest()` APIs finalized.
- **Layout validation.** Layout JSON schema validation and cross-validation against compiled graph.
- **Deterministic ID generation.** Port, route, and slot IDs now produced deterministically for stable diffs.
- **Source formatter.** Canonical pretty-printer for `.patch` source files.
- **Fixture overhaul.** 19 fixture files updated to split `io()` into `in()`/`out()` for directional protocols.
- **Test coverage.** 475 tests passing (up from ~134 in early versions).

### v0.2.4 — 2026-03-26

- **Ring keyword confirmed implemented.** 45 tests passing across parser, DRC, and serialization. DRC validates ring member references (instance existence, port existence, protocol matching). Redundant ring patterns work (primary + backup with explicit port refs). Frontend emitter can safely emit `ring` declarations for roundtrip fidelity.
- **Playground layout engine overhauled.** Switched from MSAGL JS post-processing to ELK ORTHOGONAL routing (single-pass, -265KB bundle). Port geometry now passed explicitly to ELK for accurate edge-to-port alignment. Crossing minimization: worship 4→1, broadcast 3→0.
- **Bench tooling.** `npm run bench:clean` wipes generated artifacts. Per-phase timing (compile/build/layout/render). Crossing detail diagnostics showing exactly which edges cross.

### v0.2.3 — 2026-03-25

- **Reid's questions page added.** Answers to emitter alignment, multi-file loading, project tree sidebar, error display, and WASM integration questions.
- **Frontend integration guide expanded.** Emitter requirements summary table covering cards, slots, config labels, rings, bidirectional cables, and ID format. Error display rules: never block rendering entirely, show partial results.
- **Compiler API clarified.** `compile_project()` receives all files as a map (no filesystem I/O). `resolve_uses()` returns namespace strings, not file paths. `check()` replaces `parse()` for DRC validation.

### v0.2.2 — 2026-03-23

- **Cards and slots.** Templates with `device_type: "card"` and `fits` meta for slot compatibility. Slot assignments use bare identifiers (not quoted strings).
- **Config labels.** Use directional port names: `label Dante_Pri_In[1]: "Lead Vocal"`.
- **Bidirectional cables.** Two `connect` statements per cable (forward + return path).
- **Template naming.** Model only (`CL5` not `Yamaha_CL5`).
- **ID separator.** `::` throughout (`pl::template::port`). Old `pl_` underscore format deprecated.
- **Bridges.** Use directional port names: `bridge Mic_In -> Dante_Pri_Out`.

### v0.2.1 — 2026-03-21

- **Multi-file projects.** `use` statements for cross-file references. `project.json` manifest with root entry point, page list, and library paths.
- **Compiler APIs.** `compile_project(filesMap, rootPath)` for multi-file compilation. `resolve_uses(source)` for dependency discovery. Both available in WASM and Python.
- **Layout sidecar.** `.layout.json` schema for persisting canvas positions, group boxes, and viewport. `validate_layout()` API.

### v0.2.0 — 2026-03-19

- **Initial specification.** Rust parser replacing Chevrotain JS parser. Complete grammar for templates, instances, connect, bridge, signal, config, stream, and ring statements.
- **Port direction model.** `in`, `out`, `io` with protocol and connector attributes. `io` reserved for ring/bus protocols (OptoCore, TWINLANe, AVB/Milan, GigaACE).
- **Index specs.** Single indices (`[1]`), ranges (`[1..32]`), and slices on connect/bridge statements.
- **DRC engine.** Seven rule categories: structural, electrical, logical, mechanical, temporal, direction, and ring topology. Suppressions via `@suppress`.
- **Design decisions.** All ratified through structured Socratic debate and cross-agent consensus (appendix).
