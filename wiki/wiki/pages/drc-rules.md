---
title: DRC Rules Reference
tags: [drc, compiler, validation, errors]
sources: [patchlang-design-guide/language-reference, patchlang-design-guide/compiler]
updated: 2026-04-16
---

# DRC Rules Reference

**Source:** `docs/patchlang-design-guide/language-reference.md`, `compiler.md`
**Type:** Reference table

## Summary

The DRC (Design Rule Check) engine runs after parsing and auto-index resolution. Entry point: `drc::run_all(program)`. Diagnostics have three severities: **error** (invalid, must fix), **warning** (likely problem), **info** (advisory).

Suppressible at connection level via `@suppress(layer_name)`. Supported layers: `direction`, `mechanical`, `electrical`, `logical`, `temporal`, `structural`.

---

## Layer Execution Order

1. Structural → Direction → Mechanical → Electrical → Logical → Temporal → Ring → Flow → Convention

---

## Structural Layer (S01–S16)

| Code | Severity | Rule |
|------|----------|------|
| S01 | Error | Instance references unknown template |
| S02 | Error | Slot assignment references unknown card template |
| S03 | Error | Connect references unknown port on instance |
| S04 | Error | Route references unknown port on instance (checks effective ports: template + card) |
| S05 | Error | Bus input/output references unknown port on instance (checks effective ports: template + card) |
| S06 | Error | Channel index out of declared range |
| S07 | Error | Config block references unknown instance |
| S08 | Error | Signal origin references unknown instance |
| S09 | Error | Signal origin references unknown port on instance |
| S10 | Error | Duplicate instance name |
| S11 | Error | Duplicate signal name |
| S12 | Warning | Card `fits` value does not match slot format |
| S13 | Warning | Card `fits` but no slot in scope uses that format |
| S14 | Warning | Vector port referenced without channel index (suppressible) |
| S15 | Error | Range size mismatch — left and right sides of `connect` have different channel counts |
| S16 | Error | Card port name collision — card port conflicts with template port or another card's port |

**Note on S04/S05:** Route and bus checks use the instance's **effective port namespace** (template ports + card-provided ports from slot assignments). A route like `route MADI[41] -> LINE[1]` is valid when `MADI` comes from an installed card. See D015.

---

## Direction Layer (D01–D03)

| Code | Severity | Rule |
|------|----------|------|
| D01 | Error | Cannot connect output to output |
| D02 | Error | Cannot connect input to input |
| D03 | — | Ports with direction `io` are always valid — skipped |

---

## Mechanical Layer (M01)

| Code | Severity | Rule |
|------|----------|------|
| M01 | Error | Physical connector type mismatch (connectors cannot mate) |

---

## Electrical Layer (E01–E02)

| Code | Severity | Rule |
|------|----------|------|
| E01 | Error | Level mismatch large enough to damage equipment |
| E02 | Warning | Level mismatch that may need a pad |

---

## Logical Layer (L01)

| Code | Severity | Rule |
|------|----------|------|
| L01 | Error | Protocol mismatch — protocols are not interoperable |

---

## Temporal Layer (T01)

| Code | Severity | Rule |
|------|----------|------|
| T01 | Warning | Clock domain mismatch — sample rate conversion may introduce artifacts |

---

## Ring Layer (R01–R04)

| Code | Severity | Rule |
|------|----------|------|
| R01 | Error | Ring member references unknown instance |
| R02 | Error | Ring member explicit port does not exist on template |
| R03 | Warning | Ring member port does not have ring protocol in attributes |
| R04 | Error | Implicit ring member ambiguous — zero or multiple ports match protocol |

---

## Flow Layer (F01–F03)

AES67 interoperability checks. See D013.

| Code | Severity | Rule |
|------|----------|------|
| F01 | Warning | Flow slot exhaustion — stream count exceeds Dante chipset limit |
| F02 | Info | AES67 stream exceeds 8 channels — hardware will auto-split into multiple flows |
| F03 | Error | Multicast prefix mismatch between AES67 devices — audio will silently fail |

---

## Convention Layer (C01–C05)

| Code | Severity | Rule |
|------|----------|------|
| C01 | Info | Orphaned instance — no connections, bridges, ring membership, or config |
| C02 | Warning | Duplicate connection — same source/target port pair connected more than once |
| C03 | Info | Template declared with zero ports |
| C04 | Info | Bus declared with zero outputs |
| C05 | Info | Redundancy terminates at AES67 boundary — AES67 flows use Primary port only |

---

## Meta Info Hints (M-I01 through M-I08)

Run as part of the structural layer:

| Code | Severity | Rule |
|------|----------|------|
| M-I01 | Info | Unknown `kind` value |
| M-I02 | Info | Deprecated `device_type` used — migrate to `kind` |
| M-I03 | Info | Unknown `rf_subtype` value |
| M-I04 | Info | `rf_band` present but `kind` is not `rf-system` |
| M-I05 | Warning | `rf_min_channels` is zero (must be positive) |
| M-I06 | Warning | `rf_max_channels` is less than `rf_min_channels` |
| M-I07 | Info | Unknown `dante_chipset` value — expected: `Ultimo`, `Broadway`, `Brooklyn_II`, `Brooklyn_3`, `HC` |
| M-I08 | Warning | Ultimo chipset does not support AES67 — instance has `aes67_mode: true` but template uses Ultimo |

---

## Auto-Assignment Error Codes (A01–A05)

| Code | Condition |
|------|-----------|
| A01 | `[auto]` used in a `route` or `bus` declaration |
| A02 | Both sides of a connection use `[auto]` |
| A03 | `[auto]` on a scalar port (no declared range), or cannot infer count |
| A04 | Auto-assignment exceeds the port's declared range |
| A05 | Explicit indices fragment the range — cannot find N contiguous channels |

---

## Diagnostic JSON Structure

```json
{
  "severity": "error" | "warning" | "info",
  "layer": "structural" | "direction" | "mechanical" | "electrical" | "logical" | "temporal" | "ring" | "flow" | "convention",
  "message": "human-readable description",
  "span": { "start": 142, "end": 168, "file": 0 },
  "source": "optional port ref label",
  "target": "optional port ref label",
  "fix": "optional suggestion"
}
```

`span.file` is an index into `ProjectResult.files` array. Absent for single-file `check()`.

## Relation to Other Wiki Pages

- [[language-reference]] — `@suppress` usage and layer names
- [[compiler-architecture]] — DRC engine architecture and `run_all()`
- [[design-decisions]] — rationale for specific rules (D006 for S15, D015 for S04/S05, D013 for Flow layer)
