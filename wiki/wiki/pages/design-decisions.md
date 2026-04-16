---
title: Design Decisions
tags: [decisions, rationale, design, patchlang]
sources: [patchlang-design-guide/decisions, patchlang-design-guide/appendix]
updated: 2026-04-16
---

# Design Decisions

**Source:** `docs/patchlang-design-guide/decisions.md`, `appendix.md`
**Type:** Decision log

## Summary

Running log of significant design decisions — what was chosen, what was rejected, and why. Rationale was developed through structured Socratic debates (multiple AI agent perspectives). Exists so settled questions are not re-litigated.

---

## D001 — IT Infrastructure Scope
**2026-03-28 | Decided**

**Out of scope.** PatchLang does not model switches or network topology. Target user is an AV engineer, not an IT engineer. Dante and IP audio protocols are modeled as logical virtual networks — all devices can connect without explicitly routing through switch infrastructure. DRC benefit (same-switch redundancy failures) does not justify scope expansion for this audience.

---

## D002 — Dante Secondary Redundancy Modeling
**2026-03-28 | Decided**

**Annotation metadata on primary connect, not explicit secondary ports.** With IT infrastructure out of scope (D001), there is nothing to validate secondary connects against. Explicit secondary ports impose 2x verbosity (40 connects instead of 20 for a 10-device system) for zero analytical benefit. Secondary Dante carries no independent signal — it is a mirror of primary managed by Dante Controller.

```patch
connect Stage.Dante_Pri_Out[1..32] -> Console.Dante_Pri_In[1..32] {
  cable: "Cat6a_Pri"
  redundant_cable: "Cat6a_Sec"
}
```

---

## D003 — WordClock Port Direction (Superseded by D008)
**2026-03-28 | Pending at time, superseded**

Superseded by D008.

---

## D004 — AVB/Milan Port Direction
**2026-03-28 | Pending — awaiting Reid's input**

AVB/Milan devices typically have one Ethernet jack carrying both directions simultaneously (unlike WordClock's separate BNCs). Devices can be simultaneous Talkers and Listeners. Still pending.

---

## D005 — `bridge` vs `route` Semantics
**2026-03-28 | Decided**

**Fixed/configurable axis** (not physical/logical axis). `bridge` in a template = manufacturer-hardwired path. `route` in an instance = operator-configured routing. Top-level `bridge` = DRC signal tracing assertion.

The physical/logical axis fails specifically for Probe v2 push: under that model, all internal paths in a software-defined console become `bridge`, giving Probe no way to distinguish "do not touch — hardwired" from "push this — operator-configured."

---

## D006 — Range Size Mismatch in Connect
**2026-03-29 | Decided**

**Hard compile error (S15)**, not warning or silent truncation. Cost asymmetry: a hard error costs 30 seconds to fix; a silent wrong costs a show (engineer patches channels 9–16, hears nothing, debugs Dante Controller, SFP, console routing — never opens the `.patch` file). Use `@suppress(structural)` for intentional partial connects.

---

## D007 — Import Aliasing
**2026-03-29 | Decided**

**Defer aliasing.** Use manufacturer-prefixed names (`CL5`, `Rio3224`) as a structural collision prevention. Aliasing (`as`) creates a local fiction, severs context for human readers and LLMs, and rewards bad library naming. Future escape hatch: **qualified references** (`yamaha::CL5`), not `as` aliasing.

---

## D008 — WordClock Port Direction
**2026-03-29 | Decided (Reid's input)**

**Split `in`/`out`**, not `io`. "I've never seen a BNC that is bidirectional." WordClock uses separate physical 75Ω BNC connectors. `io` was wrong. Splitting enables DRC to catch two clock outputs fighting or an unsynced device. Devices that can be master or slave declare both `WordClock_In` and `WordClock_Out`. Protocol-embedded clocking (inside Dante, MADI streams) needs no separate port.

---

## D009 — PTPv2 (IEEE 1588) Port Modeling
**2026-03-29 | Decided**

**No new port type.** PTP is a multicast UDP protocol running over Ethernet — no physical "PTP In" connector exists. Grandmaster/slave roles are elected dynamically via BMCA. Model as: (1) protocol attribute tag on Ethernet ports `[AES67, PTP]`, and (2) instance-level metadata (`ptp_role`, `ptp_domain`, `ptp_priority1`). Consistent with NMOS IS-04/IS-09 industry standard.

---

## D010 — Intercom Port Modeling Scope
**2026-03-29 | Decided (Reid's input)**

**Model matrices and physical signal sources only.** Three tiers:
1. Intercom matrices (Eclipse HX, Artist, ADAM-M) — model fully
2. Panel physical audio inputs (sources into the matrix) — model as `in()` ports
3. Headset/monitoring, management LAN, control interfaces — do not model

---

## D011 — Template Classification (`kind` Meta Field)
**2026-04-01 | Decided**

**No new keywords. Rename `device_type` to `kind`.** Device kinds: `device`, `card`, `fixed-converter`, `stage-core`, `mic-di`, `mic-splitter`, `rf-system`. Composition kinds: `system`, `venue`. `template` remains the sole declaration keyword. Consistent with D005 card precedent: metadata over keywords. `kind` chosen over `role` (circular), `type` (reserved), `category` (already in use). Backward compat: `device_type` accepted as deprecated alias.

---

## D012 — Backbone Connection Syntax
**2026-04-02 | Decided (Socratic debate)**

**`backbone: true` as boolean key-value on existing `connect` statements.** No new keyword, no parser changes. GigaACE IS a physical cable — `connect` is the correct physical primitive; `backbone: true` modifies how DRC and Signal Trace interpret it.

```patch
connect S7000.GigaACE_Pri_Out -> DM64.GigaACE_Pri_In {
  backbone: true
  cable: "GigaACE_Pri"
}
```

Covers: A&H dLive (GigaACE), Yamaha RIVAGE, DiGiCo SD-Range (OptoCore), Lawo mc², Calrec, SSL System T, Studer Vista. Integrated consoles (Avantis, SQ) need no backbone.

---

## D013 — AES67 Interop Modeling
**2026-04-03 | Decided**

**No new syntax.** Use existing `stream` keyword with `protocol: "AES67"`, `dante_chipset` meta field, `aes67_mode` and `multicast_prefix` instance properties. New Flow DRC layer (F01-F03) catches the three most common real-world AES67 failures: flow slot exhaustion, multicast prefix mismatch, and redundancy loss at protocol boundaries. All detectable with static analysis.

---

## D015 — S04/S05 Route and Bus Checks Use Effective Ports
**2026-04-05 | Decided**

**Use effective ports** (template + card-provided). Routes and buses inside an instance body may reference card-provided ports. Original spec caused 130 false errors on the Hillsong MTG project (Venue FOH Rack buses referencing `MADI` ports from an installed card). Connect checks (S03/S16) already used effective ports — routes/buses should too.

---

## D016 — Case Sensitivity Policy
**2026-04-05 | Decided (Socratic debate)**

**Case-insensitive for attributes/connectors/meta values; case-sensitive for identifiers.**
- Attributes (`Dante`, `analog`, `primary`) — case-insensitive: `analog` = `Analogue` = `ANALOG`
- Connectors (`XLR`, `etherCON`) — case-insensitive
- Meta values (`manufacturer`, `model`) — case-insensitive for catalog lookups
- Identifiers (template names, instance names, port names) — case-sensitive

Rationale: the emitter generates `.patch` files from frontend UI input. If the language is strict about casing for things the user doesn't directly control, every mismatch between emitter output and catalog becomes a bug the user can't fix. Matches CSS/HTML model. VHDL (closest HDL) is fully case-insensitive.

---

## D017 — Bus Output Syntax: Named Outputs with Optional Destinations
**2026-04-13 | Decided (Socratic debate)**

**Output labels required. Multi-destination supported. Unrouted outputs valid. Old unlabeled `output: Port` removed.**

The old syntax silently dropped `InternalBusOutput.name` from the emitter. No backward compat needed — language not yet deployed to users.

AST change: `BusEntry.outputs` from `Vec<PortRef>` → `Vec<BusOutput>` where `BusOutput { label: String, destinations: Vec<PortRef> }`.

---

## Appendix Summary (Key Structural Decisions)

| Decision | Choice | Confidence |
|----------|--------|-----------|
| Card syntax | Templates with `kind: "card"` meta — no `card` keyword | 88% |
| Ring member syntax | Both forms accepted; emitter always outputs explicit | 82% |
| Slot/card compatibility | Inverted — cards declare `fits` | 87% |
| ID separator | `::` double-colon | 85% |
| Reserved keywords | Only `ring` and `member`; `card` unreserved | 92% |
| Project structure | One DB row per canvas level | High |
| `project.json` | Thin manifest; sub-levels inferred from `use` graph | High |
| Multi-file compilation | File map `HashMap<String, String>` — no concatenation | High |
| Flat namespace | All templates share one namespace after merge | High |

## Relation to Other Wiki Pages

- [[language-reference]] — how decisions manifest as syntax
- [[compiler-architecture]] — how decisions manifest as implementation
- [[drc-rules]] — DRC rules referenced in decisions (S15, S04/S05, Flow layer)
