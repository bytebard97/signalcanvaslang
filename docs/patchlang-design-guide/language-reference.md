# Language Reference

This section is the formal grammar and syntax reference. It is meant for lookup, not for reading cover to cover. Start with the Examples section if you want to understand PatchLang by seeing it in action.

## Lexical Structure

### Comments

Lines starting with `#` are comments. Comments extend to the end of the line.

```ebnf
comment = "#" { any-char-except-newline } ;
```

```
# This is a comment
template Amp { }  # inline comments work too
```

### Whitespace

Spaces, tabs, carriage returns, and newlines are whitespace. Whitespace separates tokens but is otherwise insignificant. There is no significant indentation.

### Identifiers

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

### Keywords

```
template  instance  is  connect  bridge  bridge_group  link_group
signal  flag  stream  config  ports  meta  in  out  io
for  over  generate  use  slot  routing  route  bus  label
ring  member
```

Keywords can be used as property keys but not as names for templates, instances, or ports.

`auto` is a **contextual keyword** — it is only recognized inside index brackets `[]`. Outside brackets, `auto` is a valid identifier (e.g., `instance auto is T`).

`for`, `over`, and `generate` are reserved for future use (parametric template generation). No grammar production exists for them yet.

`card` is **not** a keyword — it is available as an identifier. Card templates use `meta { device_type: "card" }` instead.

### Annotations

```
@suppress    @version
```

### Literals

```ebnf
number         = "0" | ( digit-nonzero { digit } ) ;
string-literal = '"' { any-char-except-quote } '"' ;
digit-nonzero  = "1".."9" ;
```

### Punctuation

```
->  ..  .  {  }  (  )  [  ]  :  ,  *
```

---

## Grammar

A PatchLang file is a sequence of top-level statements.

```ebnf
program   = { statement } ;
statement = template-decl | instance-decl | connect-decl | bridge-decl
          | bridge-group-decl | link-group-decl | signal-decl | flag-decl
          | stream-decl | config-decl | use-decl | ring-decl ;
```

---

## Statements

### Template Declaration

Defines a reusable device type. Templates are the fundamental building block — whether describing a single device, a room full of equipment, a building, or an entire campus.

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
    Dante_Pri_In[1..32]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
    Dante_Sec_In[1..32]: in(etherCON) [Dante, secondary]
    Dante_Sec_Out[1..32]: out(etherCON) [Dante, secondary]
    Mic_In[1..32]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri_Out
  slot MY_Slot[1..3]: MY_Format
}
```

#### Template Parameters with Defaults

Parameters declare named values with default values. The syntax uses `:` between name and default. Both numeric and string defaults are supported.

```
template StageSplit(mic_count: 32, name: "Default") {
  ports {
    Mic_In[1..32]: in(XLR)
    Dante_Out[1..32]: out(etherCON) [Dante]
  }
}
```

When instantiated, parameter values can be overridden:

```
instance SL_Split is StageSplit(mic_count: 16)
```

#### Template Composition

Templates can contain nested `instance` and `connect` declarations, enabling hierarchical composition. A template representing a room or system can instantiate sub-templates and wire them together internally.

```
template FOH_Rack {
  ports {
    Dante_In[1..64]: in(etherCON) [Dante]
    Dante_Out[1..64]: out(etherCON) [Dante]
  }
  instance Console is CL5
  instance Stagebox is Rio3224
  connect Stagebox.Dante_Pri_Out -> Console.Dante_Pri_In
}
```

### Meta Block

Key-value metadata inside a template.

```ebnf
meta-block = "meta" "{" { key-value-pair } "}" ;
```

Standard keys: `manufacturer`, `model`, `category`. Custom keys are allowed.

### Ports Block

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

```
Mic_In[1..32]: in(XLR)                     # 32 XLR inputs
Dante_Pri_In[1..72]: in(etherCON) [Dante, primary]   # directional with attributes
Dante_Pri_Out[1..24]: out(etherCON) [Dante, primary]  # split in/out for channel protocols
OptoCore_A: io(SFP) [OptoCore]             # io for ring/bus protocols
Mix_Bus[1..24]: out                         # outputs, no connector specified
```

#### Port Direction Model

Channel-based protocols (Dante, MADI, AES67, SDI, Analogue, AES3, SoundGrid, NDI, SMPTE2110) get **two explicit port lines** — one `in`, one `out`. This allows asymmetric channel counts (e.g., CL5 receives 72 Dante channels, sends 24).

`io` is reserved for ring/bus protocols (OptoCore, TWINLANe, AVB/Milan, GigaACE) and management ports (Ethernet_Mgmt).

WordClock uses **split `in`/`out`** despite being a management signal. Every WordClock-capable device has separate physical 75Ω BNC connectors for input and output — `io` was incorrect. See D008.

| Direction | When to use |
|-----------|-------------|
| `in()`  | Signal enters the device — mic preamps, Dante receive, SDI input, WordClock receive |
| `out()` | Signal leaves the device — headphone out, Dante send, SDI output, WordClock master |
| `io()`  | Genuinely bidirectional — ring/bus protocols (OptoCore, AVB), management (Ethernet_Mgmt) |

| Direction | Protocols |
|-----------|-----------|
| **Two lines** (`in` + `out`) | Dante, AES67, MADI, Analogue, AES3, SDI, SoundGrid, NDI, SMPTE2110 |
| **Two lines** (`in` + `out`) | WordClock — always directional, separate physical BNC connectors |
| **`io`** (ring/bus) | OptoCore, TWINLANe, AVB/Milan, GigaACE |
| **`io`** (management) | Ethernet_Mgmt |

**Backward compatibility:** The parser accepts `io` for any protocol (legacy files). The emitter must produce split `in`/`out` for channel protocols.

#### Connectors

Common connector identifiers: `XLR`, `BNC_50`, `BNC_75`, `RJ45`, `etherCON`, `SFP`, `LC_Fiber`, `SC_Fiber`, `MTRJ_Fiber`, `SpeakON`, `TRS_14`, `TRS_3`, `DB25`, `HDMI`, `SDI_BNC`, `USB`, `SMA`.

Custom connector names are allowed.

#### Attributes

Attributes describe transport protocols and properties: `Dante`, `AES67`, `MADI`, `AES3`, `SDI`, `NDI`, `SMPTE2110`, `Analogue`, `WordClock`, `Ethernet`, `primary`, `secondary`, `redundant`, `Gigabit`.

Named attributes use `key: value` syntax: `[protocol: SDI, format: UHD]`.

### Instance Declaration

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
  route Dante_In[1] -> Fader[1]
  bus Main_LR {
    input: Fader[1..8]
    output: Matrix_Out[1..2]
  }
  slot MY_Slot[1]: MY16_AUD
}
```

### Connect Declaration

Defines a physical cable between two ports.

```ebnf
connect-decl = "connect" port-ref "->" port-ref [ connect-body ] ;

connect-body = "{" [ suppress-annotation ] { key-value-pair } "}" ;

suppress-annotation = "@suppress" "(" identifier { "," identifier } ")" ;
```

```
connect Stage_Left.Dante_Pri_Out -> FOH_Console.Dante_Pri_In {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}
connect FOH_Console.Dante_Pri_Out -> Stage_Left.Dante_Pri_In {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}
```

Each physical cable with split in/out ports gets **two connect statements** — one per signal direction. Cable metadata is duplicated on both. Do NOT use `link_group` for bidirectional pairs — link groups are for multi-cable logical units (like quad-link 4K SDI).

#### DRC Suppression

The `@suppress(layer1, layer2)` annotation inside a connect body disables specific DRC checks for that connection. The annotation must appear immediately after the opening `{`.

Valid layer names for suppression:

| Layer | Checks suppressed |
|-------|-------------------|
| `direction` | D01-D03 (direction violations) |
| `mechanical` | M01 (connector mismatch) |
| `electrical` | E01-E02 (level mismatch) |
| `logical` | L01 (protocol mismatch) |
| `temporal` | T01 (clock mismatch) |
| `structural` | S14 (vector port without index) |
| `all` | All suppressible checks |

```
connect Amp.Speaker_Out -> Crossover.Line_In {
  @suppress(electrical)
  cable: "NL4_AMP_XOVER"
}
```

#### Mapping Property

The `mapping` property specifies channel-level routing. Three formats:

```
mapping: "1:1"                    # sequential one-to-one (default)
mapping: "offset 16"              # shifted by N channels
mapping: "1->3, 2->4, 3->1"      # explicit per-channel pairs
```

### Bridge Declaration

`bridge` has two scopes with distinct semantics:

**Inside a template** — a signal path guaranteed by the device's design. This path exists in every physical unit of this template regardless of software configuration or operator action. The compiler treats it as invariant. Probe does not push it. Use `bridge` for paths the manufacturer hardwired: mic preamps feeding Dante outputs, ADC outputs feeding DSP inputs, protocol converters passing signal through.

**Top-level between instances** — a system designer's DRC assertion. Declares that signal flows from one instance's port to another for tracing purposes. Used to connect signal chains across devices (e.g., telling the tracer that mic inputs on a stagebox ultimately reach console inputs via the Dante network). Not pushed by Probe.

**`bridge` is not for operator-configured routing.** Internal routing that an operator can change (console DSP assignments, matrix crosspoints, DSP patching) belongs in the instance body as `route`.

```ebnf
bridge-decl = "bridge" port-ref "->" port-ref ;
```

Inside templates, port references can be local (no instance prefix):

```ebnf
template-bridge = "bridge" port-ref-or-local "->" port-ref-or-local ;
```

```
# Top-level (between instances) — DRC signal tracing assertion
bridge Stage_Left.Mic_In[1..32] -> FOH_Console.Dante_Pri_In[1..32]

# Inside a template — manufacturer-hardwired path (correct use)
template Rio1608 {
  ports {
    Mic_In[1..16]: in(XLR)
    Dante_Pri_Out[1..16]: out(etherCON) [Dante, primary]
  }
  bridge Mic_In -> Dante_Pri_Out   # hardwired by Yamaha — correct
}

# Inside a template — software-defined console (no bridge — correct)
template CL5 {
  ports {
    Dante_Pri_In[1..72]:  in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..24]: out(etherCON) [Dante, primary]
  }
  # No bridge declarations — all internal routing is operator-configured
  # Document routing state as `route` in the instance body
}
```

The bridge target is the port where the signal is *going*. Mic inputs leave the stagebox via `Dante_Pri_Out`, and arrive at the console via `Dante_Pri_In`.

### Bridge Group Declaration

Sequential channel mapping — multiple sources auto-fill a destination range.

```ebnf
bridge-group-decl = "bridge_group" port-ref "{" { port-ref } "}" ;
```

```
bridge_group FOH.Dante_Pri_In {
  SL.Mic_In[1..4]     # maps to In[1..4]
  SR.Mic_In[1..4]     # maps to In[5..8]
}
```

### Link Group Declaration

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

### Signal, Flag, Stream Declarations

```ebnf
signal-decl = "signal" identifier [ "{" { key-value-pair } "}" ] ;
flag-decl   = "flag"   identifier [ "{" { key-value-pair } "}" ] ;
stream-decl = "stream" identifier [ "{" { key-value-pair } "}" ] ;
```

```
signal Lead_Vocal {
  origin: Stage_Left.Mic_In[1]
  channel: "1"
  description: "Worship leader vocal"
}

stream SL_Dante_Primary {
  source: Stage_Left.Dante_Pri_Out
  channels: "32"
  protocol: "Dante"
}

flag Genlock_OK {
  description: "All cameras locked to house sync"
  severity: "warning"
}
```

### Config Declaration

Per-instance channel labels and metadata.

```ebnf
config-decl  = "config" identifier "{" { config-label } "}" ;
config-label = "label" port-ref-or-local ":" string-literal
               [ "{" { key-value-pair } "}" ] ;
```

```
config FOH_Console {
  label Dante_Pri_In[1]: "Lead Vocal" { phantom: "true" }
  label Dante_Pri_In[2]: "Kick Drum"
  label Fader[1]: "Lead Vocal"
}
```

Config labels reference whichever port the label is attached to, using the split port name. Each label can carry an optional body of key-value properties.

### Use Declaration (Multi-File Compilation)

Imports templates from another `.patch` file via namespace.

```ebnf
use-decl  = "use" namespace [ "." "*" ] | "use" namespace "{" identifier { "," identifier } "}" ;
namespace = identifier { "." identifier } ;
```

Three import forms:

```
use yamaha { CL5, Rio3224 }              # selective — import specific templates
use shure.*                               # wildcard — all templates in namespace
use infrastructure.dante                  # bare namespace — import the module
```

**Namespace resolution:** Dots map to path separators. `use buildings.foh` resolves to `buildings/foh.patch` relative to the project root.

**Flat namespace:** All imported templates share a single namespace after resolution. If two files define a template with the same name, the compiler emits an error.

**Selective imports** are preferred for clarity. Wildcard imports pull in everything, which can cause name collisions in large projects.

**Naming convention (required for shared libraries):** Template names must use a manufacturer prefix or model number — not generic names. `CL5`, `Rio3224`, `SD12`, `5601MSC` are correct. `Patch_Bay` or `Power_Amp` standing alone are not acceptable in any library intended for reuse; they must be prefixed (`Neutrik_Patch_Bay`, `Yamaha_Power_Amp`). Generic names are only acceptable in project-local templates that are never published. This convention eliminates namespace collisions structurally — model numbers and manufacturer-prefixed names do not collide across vendors.

**Import aliasing (`as`):** Not supported. If two libraries define the same template name, the fix is to correct the naming in the library (use a manufacturer prefix), not to alias at the import site. If this ever proves insufficient, the future escape hatch is qualified references (`yamaha::CL5`) — not `as` aliasing. See design decision D007.

### Ring Declaration

Declares a shared transport bus topology (OptoCore, TWINLANe, AVB).

```ebnf
ring-decl   = "ring" identifier "{" { ring-entry } "}" ;
ring-entry  = "member" ( port-ref | identifier ) | key-value-pair ;
```

```
ring OptoCore_Primary {
  protocol: "OptoCore"
  member Console.OptoCore_A
  member StageRack_1.OptoCore_A
  member StageRack_2.OptoCore_A
  member MonitorRack.OptoCore_A
}
```

Member order reflects the physical ring topology.

**Member syntax:** Both explicit (`member Console.OptoCore_A`) and implicit (`member Console`) forms are accepted by the parser.

- **Explicit form** — `member Instance.Port` — references a specific port. Required when a device has multiple ring ports (e.g., OptoCore_A and OptoCore_B).
- **Implicit form** — `member Instance` — the compiler resolves the port by finding the single port whose attributes match the ring's `protocol`. If zero or multiple ports match, DRC emits an error (R04).

The emitter **must always output the explicit form** — implicit resolution is fragile if a device later gains a second ring port.

**DRC rules:** R01 (unknown instance), R02 (unknown port in explicit form), R03 (protocol attribute mismatch), R04 (ambiguous implicit resolution).

```
# OptoCore ring connecting FOH and monitor world
template StageRack {
  ports {
    OptoCore_A: io(SFP) [OptoCore]
    OptoCore_B: io(SFP) [OptoCore]
    Mic_In[1..32]: in(XLR)
  }
}

instance FOH_Rack is StageRack
instance MON_Rack is StageRack

ring OptoCore_Ring {
  protocol: "OptoCore"
  member FOH_Rack.OptoCore_A
  member MON_Rack.OptoCore_A
}
```

### Slot Definition (inside templates)

```ebnf
slot-def = "slot" identifier [ range-spec ] ":" identifier [ slot-body ] ;
slot-body = "{" { key-value-pair } "}" ;
```

```
slot MY_Slot[1..3]: MY_Format
slot Expansion[1..8]: Expansion {
  direction: "any"
  channels: 16
}
```

Slots declare the bay shape. Cards declare what bays they fit via `meta { fits: "MY_Format" }`. The optional body supports properties like `direction` and `channels` to document the slot's capabilities.

### Slot Assignment (inside instance body)

```ebnf
slot-assignment = "slot" identifier [ "[" number "]" ] ":" ( identifier | string-literal ) ;
```

```
instance Console is CL5 {
  slot MY_Slot[1]: MY16_AUD
}
```

Slot assignments use **bare identifiers** (the template name of the card). The grammar also accepts `string-literal` for backward compatibility — new code must use bare identifiers.

### Route Entry (inside instance body)

`route` declares the current operator-configured internal routing state of a specific device instance. It maps an input channel to an output channel within one device. This is the complement to `bridge`: where `bridge` in a template captures what the manufacturer hardwired, `route` in an instance captures what the operator configured.

`route` is what Probe generates from Ember+ matrix crosspoints, Q-SYS mixer routing, Yamaha SCP patching, and other device control protocols. In Probe v2, `route` declarations are the target of configuration push — the compiler's route table is applied to live hardware.

```ebnf
route-entry = "route" port-ref-or-local "->" port-ref-or-local ;
```

### Bus Entry (inside instance body)

```ebnf
bus-entry      = "bus" identifier "{" { bus-port-entry } "}" ;
bus-port-entry = ( "input" | "output" | "in" | "out" ) ":" port-ref-or-local ;
```

---

## Common Productions

### Port Reference

```ebnf
port-ref          = identifier "." identifier [ index-spec ] ;
port-ref-or-local = identifier [ "." identifier ] [ index-spec ] ;
```

A `port-ref` is always fully qualified: `Instance.Port`. A `port-ref-or-local` is used inside templates and instance bodies where the instance prefix is optional.

### Index Spec

```ebnf
index-spec    = "[" index-element { "," index-element } "]" ;
index-element = number [ ".." number ] | "auto" ;
```

```
[1]            # single channel
[1..32]        # range from 1 to 32
[1..4,7,9]     # mixed: channels 1,2,3,4,7,9
[auto]         # auto-assign contiguous channels at compile time
```

#### Auto-Assignment (`[auto]`)

When used on one side of a `connect` or `bridge`, the compiler allocates the next N contiguous available channels from the port's declared range, where N is inferred from the other side. Channels are allocated sequentially in declaration order and skip any explicitly claimed indices.

**Constraints:**
- `[auto]` must be the sole element — `[auto, 5]` is a parse error
- Both sides of a connection cannot use `[auto]` (error A02)
- `[auto]` requires a vector port with a declared range (error A03)
- `[auto]` is not valid in `route` or `bus` declarations (error A01)

**Error codes:**

| Code | Meaning |
|------|---------|
| A01 | `[auto]` used in a `route` or `bus` declaration |
| A02 | Both sides of a connection use `[auto]` |
| A03 | `[auto]` on a scalar port (no declared range), or cannot infer count |
| A04 | Auto-assignment overflowed the port range |
| A05 | Enough channels exist but explicit indices fragment the range (no contiguous block) |

**Resolution:** `[auto]` is resolved eagerly at compile time after parsing. The AST retains the `auto` keyword for roundtrip fidelity; resolved indices are stored in a side table and merged into the JSON output.

```
# Auto-assign 16 channels on the console's Dante input
connect Stage_Left.Dante_Pri_Out[1..16] -> FOH_Console.Dante_Pri_In[auto]

# Second stagebox auto-fills the next 16 channels (17..32)
connect Stage_Right.Dante_Pri_Out[1..16] -> FOH_Console.Dante_Pri_In[auto]
```

**Warning S14:** Referencing a vector port without any index spec (e.g., `connect A.Out -> B.In` where `Out` is `[1..72]`) emits a structural warning. Use explicit indices or `[auto]` to silence it.

### Key-Value Pair

```ebnf
key-value-pair = property-key ":" property-value ;
property-key   = identifier | "label" | "stream" | "route" | "bus"
               | "routing" | "config" ;
property-value = string-literal | number | port-ref ;
```

Keywords can be used as property keys. Values can be strings, numbers, or port references.

---

## Design Rule Checks (DRC)

The compiler runs DRC checks after parsing and auto-resolution. Diagnostics have three severities: **error** (invalid, must fix), **warning** (likely problem), and **info** (advisory). Checks are organized into layers.

### DRC Layers

| Code | Layer | Severity | Rule |
|------|-------|----------|------|
| D01 | Direction | Error | Cannot connect `out` to `out` |
| D02 | Direction | Error | Cannot connect `in` to `in` |
| D03 | Direction | Error | Direction violation (general) |
| S01 | Structural | Error | Instance references unknown template |
| S02 | Structural | Error | Slot assignment references unknown card template |
| S03 | Structural | Error | Connect references unknown port on instance |
| S04 | Structural | Error | Route references unknown port on template |
| S05 | Structural | Error | Bus references unknown port on template |
| S06 | Structural | Error | Channel index out of declared range |
| S07 | Structural | Error | Config block references unknown instance |
| S08 | Structural | Error | Signal origin references unknown instance |
| S09 | Structural | Error | Signal origin references unknown port |
| S10 | Structural | Error | Duplicate instance name |
| S11 | Structural | Error | Duplicate signal name |
| S12 | Structural | Warning | Card `fits` value does not match slot format |
| S13 | Structural | Warning | Card `fits` but no slot in scope uses that format |
| S14 | Structural | Warning | Vector port referenced without channel index |
| S15 | Structural | Error | Range size mismatch — left and right sides of `connect` have different channel counts |
| M01 | Mechanical | Error | Connector type mismatch (e.g., XLR to BNC) |
| E01 | Electrical | Error | Level mismatch large enough to damage equipment |
| E02 | Electrical | Warning | Level mismatch that may need a pad |
| L01 | Logical | Error | Protocol mismatch (e.g., Dante to MADI) |
| T01 | Temporal | Warning | Clock domain mismatch between connected ports |
| R01 | Ring | Error | Ring member references unknown instance |
| R02 | Ring | Error | Ring member references unknown port (explicit form) |
| R03 | Ring | Warning | Ring member port does not have ring protocol attribute |
| R04 | Ring | Error | Implicit member ambiguous (zero or multiple matching ports) |
| C01 | Convention | Info | Orphaned device (no connections, bridges, rings, or config) |
| C02 | Convention | Warning | Duplicate connection (same source-target pair) |
| C03 | Convention | Info | Template declared with zero ports |
| C04 | Convention | Info | Bus declared with zero outputs |
| M-I01 | Meta | Info | Unknown `device_type` value |
| M-I03 | Meta | Info | Unknown `rf_subtype` value |
| M-I04 | Meta | Info | `rf_band` set but `device_type` is not `rf-system` |
| M-I05 | Meta | Warning | `rf_min_channels` must be positive |
| M-I06 | Meta | Warning | `rf_max_channels` less than `rf_min_channels` |
