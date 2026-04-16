---
title: Language Reference
tags: [language, grammar, syntax, reference]
sources: [patchlang-design-guide/language-reference]
updated: 2026-04-16
---

# Language Reference

**Source:** `docs/patchlang-design-guide/language-reference.md`
**Type:** Formal grammar reference — for lookup, not for reading cover to cover

## Summary

Complete grammar and syntax reference for PatchLang. Start with [[patchlang-examples]] if you want to understand PatchLang by seeing it in action.

## Lexical Structure

### Comments
Lines starting with `#` are comments (extend to end of line). Inline comments also work.

### Whitespace
Spaces, tabs, CR, LF — insignificant. No significant indentation.

### Identifiers
```ebnf
identifier = ( letter | "_" ) { letter | digit | "_" } ;
```
Must start with a letter or underscore. **No hyphens** — use underscores.

### Keywords
```
template  instance  is  connect  bridge  bridge_group  link_group
signal  flag  stream  config  ports  meta  in  out  io
for  over  generate  use  slot  routing  route  bus  label
ring  member
```
`auto` is a **contextual keyword** (only inside `[]`). `card` is **not** a keyword.

### Annotations
```
@suppress    @version
```

---

## Statements

### Template Declaration
```
template Rio3224(mic_count: 32) @version("2.0") {
  meta { manufacturer: "Yamaha", model: "Rio3224" }
  ports {
    Mic_In[1..32]: in(XLR)
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
  }
  bridge Mic_In -> Dante_Pri_Out
  slot MY_Slot[1..3]: MY_Format
}
```
Parameters have defaults (`name: default_value`). Templates can contain nested instances and connects (composition). See [[patchlang-examples]] for composition patterns.

### Meta Block
Key-value metadata inside a template. Standard keys: `kind`, `manufacturer`, `model`, `category`, `dante_chipset`.

**`kind` values:**

| Kind | Meaning |
|------|---------|
| `device` | Physical hardware (default when absent) |
| `card` | Expansion card — requires `fits` |
| `fixed-converter` | Deterministic routing device |
| `stage-core` | Passive XLR loom/snake |
| `mic-di` | Single microphone or DI box |
| `mic-splitter` | Multi-way analogue signal splitter |
| `rf-system` | Wireless mic receiver, IEM transmitter |
| `system` | Logical grouping of devices |
| `venue` | Top-level facility or building |

`device_type` is accepted as a deprecated alias for `kind` (emits M-I02 warning).

### Ports Block
```
Mic_In[1..32]: in(XLR)
Dante_Pri_In[1..72]: in(etherCON) [Dante, primary]
Dante_Pri_Out[1..24]: out(etherCON) [Dante, primary]
OptoCore_A: io(SFP) [OptoCore]
```

**Port Direction Model:**

| Direction | When to use |
|-----------|-------------|
| `in()` | Signal enters the device (mic preamps, Dante receive, WordClock receive) |
| `out()` | Signal leaves the device (headphone out, Dante send, WordClock master) |
| `io()` | Ring/bus protocols (OptoCore, TWINLANe, AVB/Milan, GigaACE) and management (Ethernet_Mgmt) |

Channel protocols (Dante, MADI, AES67, SDI, Analogue, AES3, SoundGrid, NDI, SMPTE2110) get **two explicit lines** — one `in`, one `out`. WordClock uses split `in`/`out` (separate physical BNC connectors). See D008.

**Case sensitivity:** Attributes and connectors are **case-insensitive** (`analog` = `Analogue`). Identifiers are **case-sensitive** (`FOH_Console` ≠ `foh_console`). See D016.

### Instance Declaration
```
instance Stage_Left is Rio3224 {
  location: "Stage Left Wing"
  ip: "192.168.1.31"
}
```
Instance body can contain: `route`, `bus`, `slot` assignments, and arbitrary key-value properties.

### Connect Declaration
```
connect Stage_Left.Dante_Pri_Out -> FOH_Console.Dante_Pri_In {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}
```
Each physical cable with split in/out ports gets **two connect statements** (one per direction). Cable metadata is duplicated on both. Do NOT use `link_group` for bidirectional pairs.

**`@suppress` annotation** inside connect body disables specific DRC layers:
- `direction`, `mechanical`, `electrical`, `logical`, `temporal`, `structural`, `all`

**`mapping` property:**
```
mapping: "1:1"           # sequential one-to-one (default)
mapping: "offset 16"     # shifted by N channels
mapping: "1->3, 2->4"    # explicit per-channel pairs
```

### Bridge Declaration
- **Inside template** — path guaranteed by manufacturer's hardware design; exists in every unit. DRC treats as invariant. Probe does NOT push.
- **Top-level between instances** — system designer's DRC assertion for signal tracing. Read-only.

**NOT for operator-configured routing** — use `route` in instance body instead. See D005.

### Bridge Group Declaration
```
bridge_group FOH.Dante_Pri_In {
  SL.Mic_In[1..4]     # maps to In[1..4]
  SR.Mic_In[1..4]     # maps to In[5..8]
}
```
Sequential channel mapping — multiple sources auto-fill a destination range.

### Link Group Declaration
Groups connections as a logical unit (e.g., quad-link 4K SDI):
```
link_group Cam1_UHD {
  connect Cam1.SDI_Out[1] -> Router.SDI_In[1]
  connect Cam1.SDI_Out[2] -> Router.SDI_In[2]
  mode: "quad_link_4K"
}
```

### Signal, Flag, Stream Declarations
```
signal Lead_Vocal { origin: Stage_Left.Mic_In[1], description: "Worship leader" }
stream SL_Dante_Primary { source: Stage_Left.Dante_Pri_Out, channels: "32", protocol: "Dante" }
flag Genlock_OK { description: "All cameras locked to house sync", severity: "warning" }
```

### Config Declaration
```
config FOH_Console {
  label Dante_Pri_In[1]: "Lead Vocal" { phantom: "true" }
  label Dante_Pri_In[2]: "Kick Drum"
}
```

### Use Declaration (Multi-File)
```
use yamaha { CL5, Rio3224 }      # selective
use shure.*                       # wildcard
use infrastructure.dante          # bare namespace
```
Dots map to path separators: `buildings.foh` → `buildings/foh.patch`. All imported templates share a flat namespace — duplicate names are compile errors. Import aliasing (`as`) is not supported; use manufacturer-prefixed names instead. See D007.

### Ring Declaration
```
ring OptoCore_Primary {
  protocol: "OptoCore"
  member Console.OptoCore_A      # explicit form
  member StageRack_1             # implicit form (resolved by protocol)
}
```
Member order reflects physical ring topology. Emitter **must always output explicit form**. See DRC rules R01-R04.

### Slot Definition (inside templates)
```
slot MY_Slot[1..3]: MY_Format
```
Cards declare what they fit: `meta { fits: "MY_Format" }`. Inverted model — adding a new card never requires editing existing templates. See D005.

### Slot Assignment (inside instance body)
```
instance Console is CL5 {
  slot MY_Slot[1]: MY16_AUD      # bare identifier
}
```
Card ports are merged into the instance's effective port namespace (flat merge). Card ports referenced directly: `FOH.MicIn[1]`. Collisions emit S16.

### Route Entry (inside instance body)
```
route MADI_In[41] -> LINE[1]
```
Operator-configured internal routing state. What Probe reads from live hardware and what Probe v2 pushes. Distinct from `bridge` (see D005).

### Bus Entry (inside instance body)
```
bus Main_LR {
  label: "SPOTIFY>FOH"           # display name for identifiers with invalid chars
  input: Fader[1..8]
  output "Main L": Matrix_Out[1]
  output "Main R": Matrix_Out[2], Dante[5]   # multi-destination
  output "Link 1-C"                           # unrouted output
}
```
Output labels are **required** (see D017). `label:` body key carries human-readable display name that may contain `>`, `-`, etc. See [[design-decisions]] D017.

---

## Index Spec

```
[1]           # single channel
[1..32]       # range
[1..4,7,9]    # mixed
[auto]        # auto-assign contiguous channels at compile time
```

`[auto]` allocates the next N contiguous channels in declaration order, skipping explicit indices. Error codes A01-A05. Not valid in `route` or `bus`. Resolved to concrete indices in JSON output; AST retains `Auto` for roundtrip.

---

## Key-Value Pair

Keywords can be used as property keys. Values can be strings, numbers, or port references.

---

## DRC Layers

See [[drc-rules]] for the complete rule reference.

## Relation to Other Wiki Pages

- [[patchlang-overview]] — high-level introduction
- [[patchlang-examples]] — annotated examples of each statement type
- [[drc-rules]] — full DRC rule table
- [[compiler-architecture]] — how the compiler processes this grammar
- [[design-decisions]] — rationale for every syntax choice
