---
layout: default
title: Language Specification
---

# PatchLang Language Specification

**Version:** 0.2.5
**Status:** Draft

PatchLang is a domain-specific language for describing signal flow in broadcast and live production environments. It defines device templates, physical instances, cable connections, logical signal mappings, and channel configuration. PatchLang files (`.patch`) are the source of truth — they are human-readable, git-diffable, and designed for LLM generation.

---

## 1. Lexical Structure

### 1.1 Comments

Lines starting with `#` are comments. Comments extend to the end of the line.

```ebnf
comment = "#" { any-char-except-newline } ;
```

```
# This is a comment
template Amp { }  # inline comments work too
```

### 1.2 Whitespace

Spaces, tabs, carriage returns, and newlines are whitespace. Whitespace separates tokens but is otherwise insignificant. There is no significant indentation.

### 1.3 Identifiers

```ebnf
identifier = ( letter | "_" ) { letter | digit | "_" } ;
letter     = "A".."Z" | "a".."z" ;
digit      = "0".."9" ;
```

Identifiers name templates, instances, ports, signals, and properties. They must start with a letter or underscore. Hyphens are not allowed — use underscores.

```
FOH_Console       (* valid *)
_80_ch_Splitter   (* valid — leading underscore for names starting with digits *)
Stage_Left        (* valid *)
my-device         (* INVALID — no hyphens *)
```

### 1.4 Keywords

The following identifiers are reserved keywords:

```
template  instance  is  connect  bridge  bridge_group  link_group
signal  flag  stream  config  ports  meta  in  out  io
for  over  generate  use  slot  routing  route  bus  label
ring  member
```

Keywords can be used as property keys (see §3.10) but not as names for templates, instances, or ports.

### 1.5 Annotations

```
@suppress    @version
```

### 1.6 Literals

```ebnf
number         = "0" | ( digit-nonzero { digit } ) ;
string-literal = '"' { any-char-except-quote } '"' ;
digit-nonzero  = "1".."9" ;
```

### 1.7 Punctuation

```
->  ..  .  {  }  (  )  [  ]  :  ,  *
```

---

## 2. Grammar

### 2.1 Program

A PatchLang file is a sequence of top-level statements.

```ebnf
program   = { statement } ;
statement = template-decl | instance-decl | connect-decl | bridge-decl
          | bridge-group-decl | link-group-decl | signal-decl | flag-decl
          | stream-decl | config-decl | use-decl | ring-decl ;
```

---

## 3. Statements

### 3.1 Template Declaration

Defines a reusable device type.

```ebnf
template-decl = "template" identifier [ param-list ] [ version-annotation ]
                "{" { template-block } "}" ;

template-block = meta-block | ports-block | template-bridge
               | template-instance | template-connect | slot-def ;

param-list = "(" param-def { "," param-def } ")" ;
param-def  = identifier ":" ( number | string-literal ) ;

version-annotation = "@version" "(" string-literal ")" ;
```

```
template Rio3224(mic_count: 32) @version("2.0") {
  meta {
    manufacturer: "Yamaha"
    model: "Rio3224"
    category: "Stagebox"
  }
  ports {
    Dante_Pri_In[1..32]:  in(etherCON)  [Dante, primary]
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
    Dante_Sec_In[1..32]:  in(etherCON)  [Dante, secondary]
    Dante_Sec_Out[1..32]: out(etherCON) [Dante, secondary]
    Mic_In[1..32]:        in(XLR)
    Line_Out[1..16]:      out(XLR)
  }
  bridge Mic_In -> Dante_Pri_Out   # hardwired mic preamp path
  slot MY_Slot[1..3]: MY_Card
}
```

### 3.2 Meta Block

Key-value metadata inside a template.

```ebnf
meta-block = "meta" "{" { key-value-pair } "}" ;
```

Standard keys: `manufacturer`, `model`, `category`. Custom keys are allowed.

### 3.3 Ports Block

Defines the device's physical interfaces.

```ebnf
ports-block = "ports" "{" { port-def } "}" ;

port-def = identifier [ range-spec ] ":" port-direction
           [ connector-spec ] [ attribute-list ] ;

range-spec     = "[" number ".." number "]" ;
port-direction = "in" | "out" | "io" ;
connector-spec = "(" identifier ")" ;
attribute-list = "[" attribute { "," attribute } "]" ;
attribute      = identifier [ ":" identifier ] ;
```

#### Port direction rules

Channel-based protocols carry discrete numbered audio or video channels and **must** use split `in` + `out` lines — never `io`:

```
Dante_Pri_In[1..32]:  in(etherCON)  [Dante, primary]
Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
Mic_In[1..32]:        in(XLR)
MADI_In[1..64]:       in(SC_Fiber)  [MADI]
MADI_Out[1..64]:      out(SC_Fiber) [MADI]
WordClock_In:         in(BNC_75)    [WordClock]
WordClock_Out:        out(BNC_75)   [WordClock]
```

WordClock uses separate physical 75Ω BNC connectors — never bidirectional. Devices that are always clock masters declare only `WordClock_Out`; always-slaves declare only `WordClock_In`; devices that can be either declare both.

`io` is reserved for ring/bus protocols and management ports:

```
OptoCore_A:      io(etherCON) [OptoCore]     # ring bus
AVB_Port:        io(etherCON) [AVB]           # ring/bus
Ethernet_Mgmt:   io(RJ45)                     # management
```

**Protocol → direction mapping:**

| Direction | Protocols |
|-----------|-----------|
| `in` + `out` (split) | Dante, AES67, MADI, AES3, SDI, Analogue, SoundGrid, NDI, SMPTE2110, WordClock |
| `io` (ring/bus) | OptoCore, TWINLANe, AVB, Milan, GigaACE |
| `io` (management) | Ethernet_Mgmt |

#### Connectors

Common connector identifiers: `XLR`, `BNC_75`, `RJ45`, `etherCON`, `SFP`, `LC_Fiber`, `SC_Fiber`, `MTRJ_Fiber`, `SpeakON`, `TRS_14`, `TRS_3`, `DB25`, `HDMI`, `SDI_BNC`, `USB`, `SMA`.

Custom connector names are allowed.

#### Attributes

Attributes describe transport protocols and properties: `Dante`, `AES67`, `MADI`, `AES3`, `SDI`, `NDI`, `SMPTE2110`, `Analogue`, `WordClock`, `Ethernet`, `primary`, `secondary`, `redundant`, `Gigabit`.

Named attributes use `key: value` syntax: `[protocol: SDI, format: UHD]`.

### 3.4 Instance Declaration

Creates a physical device from a template.

```ebnf
instance-decl = "instance" identifier "is" identifier
                [ arg-list ] [ version-constraint ] [ instance-body ] ;

arg-list           = "(" arg-def { "," arg-def } ")" ;
arg-def            = identifier ":" ( number | string-literal ) ;
version-constraint = "@version" "(" string-literal ")" ;

instance-body = "{" { instance-entry } "}" ;
instance-entry = route-entry | bus-entry | slot-assignment | key-value-pair ;
```

```
instance Stage_Left is Rio3224 {
  location: "Stage Left Wing"
  ip: "192.168.1.31"
}

instance FOH_Console is CL5(mic_count: 48) @version(">=4.0") {
  location: "Front of House"
  route Dante_Pri_In[1] -> Fader[1]
  route Dante_Pri_In[2] -> Fader[2]
  bus Main_LR {
    input: Fader[1..8]
    output: Matrix_Out[1..2]
  }
  slot MY_Slot[1]: Dante_Card   # bare identifier — not quoted
}
```

### 3.5 Connect Declaration

Defines a physical cable between two ports.

```ebnf
connect-decl = "connect" port-ref "->" port-ref [ connect-body ] ;

connect-body = "{" [ suppress-annotation ] { key-value-pair } "}" ;

suppress-annotation = "@suppress" "(" layer { "," layer } ")" ;
layer = "direction" | "mechanical" | "electrical" | "logical"
      | "temporal" | "structural" | "all" ;
```

```
connect Stage_Left.Dante_Pri -> Dante_Switch.Port[1] {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}

connect SL.Analog_Out[1..4] -> FOH.Line_In[1..4] {
  @suppress(protocol_mismatch)
  mapping: "1:1"
}
```

#### Range size constraint

When both sides of a `connect` use explicit channel ranges (not `[auto]`), the channel counts must match. A mismatch is a compile error (S15). Use `@suppress(structural)` for intentional partial connects:

```
# error — 16 channels into 8
connect A.Out[1..16] -> B.In[1..8]

# correct — same count on both sides
connect A.Out[1..8] -> B.In[1..8]

# also correct — intentional partial, explicitly declared
connect A.Out[1..16] -> B.In[1..16] { @suppress(structural) }
```

#### Mapping Property

The `mapping` property specifies channel-level routing. Three formats:

```
mapping: "1:1"                    # sequential one-to-one (default)
mapping: "offset 16"              # shifted by N channels
mapping: "1->3, 2->4, 3->1"      # explicit per-channel pairs
```

### 3.6 Bridge Declaration

`bridge` has two distinct scopes with different semantics.

```ebnf
bridge-decl = "bridge" port-ref "->" port-ref ;
```

#### Inside a template — manufacturer-hardwired path

A `bridge` inside a template declares a signal path that exists in every unit of that device as manufactured — the path cannot be removed by operator action. The compiler treats it as invariant. SignalCanvasProbe does **not** push template bridges to hardware (they are hardwired and not configurable).

Use `bridge` only when you can say: "every unit that ships from the factory has this path, regardless of software configuration."

```
template Rio3224 {
  ports {
    Mic_In[1..32]:        in(XLR)
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
  }
  bridge Mic_In -> Dante_Pri_Out   # mic preamp is hardwired to Dante — correct
}

template CL5 {
  # no bridge declarations — all internal routing is operator-configured
  # operator routing belongs in the instance as `route`
}
```

#### Top-level between instances — signal trace assertion

A top-level `bridge` between instances is a system designer's assertion that a signal relationship exists for tracing purposes. It is read-only — Signal Trace traverses it, but Probe does not push it.

```ebnf
template-bridge = "bridge" port-ref-or-local "->" port-ref-or-local ;
```

```
# Top-level assertion for signal tracing
bridge Stage_Left.Mic_In[1..32] -> FOH_Console.Dante_Pri_In[1..32]
```

#### `bridge` vs `route`

| Keyword | Scope | Meaning | Probe behavior |
|---------|-------|---------|----------------|
| `bridge` | Inside template | Manufacturer-hardwired path | Do not push |
| `bridge` | Top-level | Signal trace assertion | Read-only |
| `route` | Inside instance | Operator-configured routing state | Push via Probe v2 |

### 3.7 Bridge Group Declaration

Sequential channel mapping — multiple sources auto-fill a destination range.

```ebnf
bridge-group-decl = "bridge_group" port-ref "{" { port-ref } "}" ;
```

```
bridge_group FOH.Dante_Ch {
  SL.Mic_In[1..4]     # maps to Ch[1..4]
  SR.Mic_In[1..4]     # maps to Ch[5..8]
}
```

### 3.8 Link Group Declaration

Groups connections as a logical unit.

```ebnf
link-group-decl = "link_group" identifier "{" { connect-decl | key-value-pair } "}" ;
```

```
link_group Cam1_UHD {
  connect Cam1.SDI_Out[1] -> Router.SDI_In[1]
  connect Cam1.SDI_Out[2] -> Router.SDI_In[2]
  connect Cam1.SDI_Out[3] -> Router.SDI_In[3]
  connect Cam1.SDI_Out[4] -> Router.SDI_In[4]
  mode: "quad_link_4K"
}
```

### 3.9 Signal, Flag, Stream Declarations

```ebnf
signal-decl = "signal" identifier [ "{" { key-value-pair } "}" ] ;
flag-decl   = "flag"   identifier [ "{" { key-value-pair } "}" ] ;
stream-decl = "stream" identifier [ "{" { key-value-pair } "}" ] ;
```

The `origin` property in signals and `source` property in streams accept port references as values.

```
signal Lead_Vocal {
  origin: Stage_Left.Mic_In[1]
  channel: "1"
  description: "Worship leader vocal"
}

stream SL_Dante_Primary {
  source: Stage_Left.Dante_Pri
  channels: "32"
  protocol: "Dante"
}

flag Genlock_OK {
  description: "All cameras locked to house sync"
  severity: "warning"
}
```

### 3.10 Config Declaration

Per-instance channel labels and metadata.

```ebnf
config-decl  = "config" identifier "{" { config-label } "}" ;
config-label = "label" port-ref-or-local ":" string-literal
               [ "{" { key-value-pair } "}" ] ;
```

```
config FOH_Console {
  label Dante_In[1]: "Pastor Mic" {
    phantom: "true"
    stand: "tall boom"
  }
  label Dante_In[2]: "Worship Leader" {
    phantom: "true"
  }
  label Fader[1]: "Lead Vocal"
}
```

### 3.11 Use Declaration

Imports templates from a library namespace.

```ebnf
use-decl = "use" namespace [ "." "*" ] [ "{" identifier { "," identifier } "}" ] ;
namespace = identifier { "." identifier } ;
```

```
use audio.yamaha { CL5, Rio3224 }
use video.blackmagic.*
use infrastructure.dante
```

All imported templates share a single flat namespace. If two imports define a template with the same name, the compiler emits an error. Import aliasing (`as`) is not supported.

**Naming convention (required for shared libraries):** Template names must use a manufacturer prefix or model number — not standalone generic names. `CL5`, `Rio3224`, `SD12` are correct. `Patch_Bay` is only acceptable in project-local templates; in a shared library it must be prefixed (e.g., `Neutrik_Patch_Bay`). Model numbers and manufacturer-prefixed names are globally unique by industrial convention and do not collide across vendors.

### 3.12 Ring Declaration

Declares a network ring topology — a loop of devices connected by a shared transport protocol (e.g., OptoCore, TWINLANe, MADI ring). Members are listed in ring order.

```ebnf
ring-decl   = "ring" identifier "{" { ring-entry } "}" ;
ring-entry  = ring-member | key-value-pair ;
ring-member = "member" identifier [ "." identifier ] ;
```

Members have two forms:
- **Implicit port:** `member InstanceName` — the compiler resolves the ring port automatically based on the protocol.
- **Explicit port:** `member InstanceName.PortName` — specifies which port participates in the ring.

```
# Primary ring — implicit port resolution
ring OptoCore_Primary {
  protocol: "OptoCore"
  member Console
  member StageRack_1
  member StageRack_2
  member MonitorRack
}

# Redundant ring — explicit port references for dual-homed devices
ring OptoCore_Redundant {
  protocol: "OptoCore"
  label: "Redundant ring via B ports"
  member Console.OptoCore_B
  member StageRack_1.OptoCore_B
  member StageRack_2.OptoCore_B
  member MonitorRack.OptoCore_B
}
```

### 3.13 Slot Definition (inside templates)

```ebnf
slot-def = "slot" identifier [ range-spec ] ":" identifier ;
```

```
slot MY_Slot[1..3]: MY_Card
slot Expansion[1..8]: Expansion
```

### 3.14 Route Entry (inside instance body)

```ebnf
route-entry = "route" port-ref-or-local "->" port-ref-or-local ;
```

### 3.15 Bus Entry (inside instance body)

```ebnf
bus-entry      = "bus" identifier "{" { bus-port-entry } "}" ;
bus-port-entry = ( "input" | "output" | "in" | "out" ) ":" port-ref-or-local ;
```

### 3.16 Slot Assignment (inside instance body)

```ebnf
slot-assignment = "slot" identifier [ "[" number "]" ] ":" identifier ;
```

Card names are bare identifiers, not quoted strings. `slot MY_Slot[1]: MY16_AUD` is correct; `slot MY_Slot[1]: "MY16_AUD"` is legacy syntax and will be rejected.

---

## 4. Common Productions

### 4.1 Port Reference

```ebnf
port-ref          = identifier "." identifier [ index-spec ] ;
port-ref-or-local = identifier [ "." identifier ] [ index-spec ] ;
```

A `port-ref` is always fully qualified: `Instance.Port`. A `port-ref-or-local` is used inside templates and instance bodies where the instance prefix is optional.

### 4.2 Index Spec

```ebnf
index-spec    = "[" index-element { "," index-element } "]"
              | "[" "auto" "]" ;
index-element = number [ ".." number ] ;
```

Supports single indices, ranges, mixed lists, and auto-assignment:

```
[1]            # single channel
[1..32]        # range from 1 to 32
[1..4,7,9]     # mixed: channels 1,2,3,4,7,9
[auto]         # compiler fills in next N available contiguous channels
```

`[auto]` may only appear on one side of a `connect`. It cannot be used on both sides simultaneously. When `[auto]` is present, range size matching (S15) is skipped — the compiler resolves the count.

### 4.3 Key-Value Pair

```ebnf
key-value-pair = property-key ":" property-value ;
property-key   = identifier | "label" | "stream" | "route" | "bus"
               | "routing" | "config" ;
property-value = string-literal | number | port-ref ;
```

Keywords can be used as property keys. Values can be strings, numbers, or port references.

---

## 5. Complete Example

```
# Worship venue — Dante audio network

template Rio3224 {
  meta {
    manufacturer: "Yamaha"
    model: "Rio3224"
    category: "Stagebox"
  }
  ports {
    Dante_Pri_In[1..32]:  in(etherCON)  [Dante, primary]
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
    Dante_Sec_In[1..32]:  in(etherCON)  [Dante, secondary]
    Dante_Sec_Out[1..32]: out(etherCON) [Dante, secondary]
    Mic_In[1..32]:        in(XLR)
    Line_Out[1..16]:      out(XLR)
  }
  bridge Mic_In -> Dante_Pri_Out   # manufacturer-hardwired: mic preamp → Dante
}

template CL5 {
  meta {
    manufacturer: "Yamaha"
    model: "CL5"
    category: "Console"
  }
  ports {
    Dante_Pri_In[1..72]:  in(etherCON)  [Dante, primary]
    Dante_Pri_Out[1..72]: out(etherCON) [Dante, primary]
    Dante_Sec_In[1..72]:  in(etherCON)  [Dante, secondary]
    Dante_Sec_Out[1..72]: out(etherCON) [Dante, secondary]
    Fader[1..72]:         in
    Mix_Bus[1..24]:       out
  }
  # no bridge declarations — all internal routing is operator-configured
}

instance Stage_Left is Rio3224 {
  location: "Stage Left Wing"
  ip: "192.168.1.31"
}

instance FOH_Console is CL5 {
  location: "Front of House"
  ip: "192.168.1.10"
  route Dante_Pri_In[1] -> Fader[1]
  route Dante_Pri_In[2] -> Fader[2]
}

# One connect per direction — bidirectional cables need two statements
connect Stage_Left.Dante_Pri_Out[1..32] -> FOH_Console.Dante_Pri_In[1..32] {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}
connect FOH_Console.Dante_Pri_Out[1..32] -> Stage_Left.Dante_Pri_In[1..32] {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}

signal Lead_Vocal {
  origin: Stage_Left.Mic_In[1]
  description: "Worship leader vocal"
}

config FOH_Console {
  label Dante_Pri_In[1]: "Lead Vocal" { phantom: "true" }
  label Dante_Pri_In[2]: "Kick Drum"
}
```

---

## 6. File Conventions

- **Extension:** `.patch`
- **Encoding:** UTF-8
- **Line endings:** LF or CRLF (both accepted)
- **Layout sidecar:** `.layout.json` stores canvas positions and UI state (not part of PatchLang)
- **Config files:** `.config.patch` can hold `config` blocks imported via `use`

---

## 7. Design Principles

1. **Human-readable first.** A broadcast engineer should be able to read a `.patch` file and understand the signal chain without special tooling.
2. **LLM-friendly.** The syntax is simple enough that language models can generate valid `.patch` files from plain English descriptions.
3. **Git-diffable.** Text diffs show meaningful changes. Adding a mic input is one line, not a JSON blob.
4. **No ambiguity.** Every statement starts with a unique keyword. The grammar is LL(1).
5. **Domain-specific.** The language models broadcast concepts (ports, connectors, protocols, signal chains) directly — not through generic data structures.
