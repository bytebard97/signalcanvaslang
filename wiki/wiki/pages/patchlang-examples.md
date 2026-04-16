---
title: PatchLang Examples
tags: [language, examples, patchlang]
sources: [patchlang-design-guide/examples]
updated: 2026-04-16
---

# PatchLang Examples

**Source:** `docs/patchlang-design-guide/examples.md`
**Type:** Annotated examples

## Summary

Working examples covering every major language construct. See [[language-reference]] for grammar. See [[design-decisions]] for rationale behind syntax choices.

---

## Minimal Single-File Project

```patch
# Simple worship venue — single file
template Rio3224 {
  meta { manufacturer: "Yamaha", model: "Rio3224", category: "Stagebox" }
  ports {
    Dante_Pri_In[1..32]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
    Mic_In[1..32]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri_Out    # manufacturer-hardwired path
}

template CL5 {
  meta { manufacturer: "Yamaha", model: "CL5", category: "Console" }
  ports {
    Dante_Pri_In[1..72]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..24]: out(etherCON) [Dante, primary]
    Mix_Bus[1..24]: out
  }
  # No bridge — all internal routing is operator-configured (use route in instance)
}

instance Stage_Left is Rio3224 { location: "Stage Left Wing" }
instance FOH_Console is CL5 { location: "Front of House" }

connect Stage_Left.Dante_Pri_Out -> FOH_Console.Dante_Pri_In {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}

bridge Stage_Left.Mic_In[1..32] -> FOH_Console.Dante_Pri_In[1..32]   # DRC tracing assertion

signal Lead_Vocal { origin: Stage_Left.Mic_In[1], description: "Worship leader vocal" }

config FOH_Console {
  label Dante_Pri_In[1]: "Lead Vocal" { phantom: "true" }
  label Dante_Pri_In[2]: "Kick Drum"
}
```

---

## Multi-File Project

```
hillsong-mtg/
  project.json        ← thin manifest (name, root, author)
  campus.patch        ← entry point
  buildings/
    foh.patch
    monitors.patch
    stage.patch
  lib/
    yamaha.patch
```

`campus.patch`:
```patch
use buildings.foh { FOH_System }
use buildings.monitors { Monitor_System }
use buildings.stage { Stage_System }

instance FOH is FOH_System { location: "Front of House" }
instance Monitors is Monitor_System { location: "Monitor World" }

connect FOH.Dante_Out -> Monitors.Dante_In { cable: "Cat6a_FOH_MON" }
```

`buildings/foh.patch`:
```patch
use lib.yamaha { CL5, Rio3224 }

template FOH_System {
  meta { category: "Building" }
  ports { Dante_Out: out(etherCON) [Dante], Dante_In: in(etherCON) [Dante] }
}

instance Console is CL5 { location: "FOH Mix Position" }
instance SL_Rack is Rio3224 { location: "Stage Left" }

connect SL_Rack.Dante_Pri_Out -> Console.Dante_Pri_In { cable: "Cat6a_SL" }
bridge SL_Rack.Mic_In[1..32] -> Console.Dante_Pri_In[1..32]
```

**Key insight:** A `.patch` file IS a template at every scale. Hierarchy is just templates importing other templates via `use`.

---

## Bus Display Names (`label:`)

Broadcast console software uses `>` in bus names (`SPOTIFY>FOH`). These characters are invalid in identifiers. `label:` carries the display name while the identifier remains the stable key.

```patch
instance FOH_Engine is CL5 {
  bus SpotifyFOH {
    label: "SPOTIFY>FOH"            # display name with invalid chars
    input: Dante_Pri_In[41]
    output "Main L": Mix_Out[25]    # named output — label required
    output "Main R": Mix_Out[26]
    output "Fold Back"              # unrouted output (no destination yet)
  }
  bus Link_1 {
    input: Fader[1..8]
    output "Link 1-L": MADI_1_Out[1]
    output "Link 1-R": MADI_1_Out[2], Dante[5]   # multi-destination
  }
}
```

---

## Ring Network (OptoCore)

```patch
instance Console is SD12 { location: "FOH" }
instance StageRack_1 is SD_Rack { location: "Stage Left" }
instance StageRack_2 is SD_Rack { location: "Stage Right" }
instance MonitorRack is SD_Rack { location: "Monitor World" }

# Primary ring — implicit members (single OptoCore port per device)
ring OptoCore_Primary {
  protocol: "OptoCore"
  member Console
  member StageRack_1
  member StageRack_2
  member MonitorRack
}

# Redundant ring — explicit port references (B ports)
ring OptoCore_Redundant {
  protocol: "OptoCore"
  label: "Redundant ring via B ports"
  member Console.OptoCore_B
  member StageRack_1.OptoCore_B
  member StageRack_2.OptoCore_B
  member MonitorRack.OptoCore_B
}
```

---

## Channel Auto-Assignment with `[auto]`

```patch
# Manual (fragile — tedious and error-prone):
connect MON.Dante_Pri_Out[1..2] -> IEM_WL.Dante_In[1..2]
connect MON.Dante_Pri_Out[3..4] -> IEM_MD.Dante_In[1..2]

# With [auto] (compiler assigns sequentially):
connect MON.Dante_Pri_Out[auto] -> IEM_WL.Dante_In[1..2]    # → [1..2]
connect MON.Dante_Pri_Out[auto] -> IEM_MD.Dante_In[1..2]    # → [3..4]
connect MON.Dante_Pri_Out[auto] -> IEM_Keys.Dante_In[1..2]  # → [5..6]

# Mix explicit and auto — explicit channels are pre-scanned and skipped:
connect MON.Dante_Pri_Out[33..34] -> Recorder.Dante_In[1..2]   # locked preset
connect MON.Dante_Pri_Out[auto] -> IEM_WL.Dante_In[1..2]       # → [1..2] (skips 33..34)
```

Use `[auto]` when channel numbers don't matter. Use explicit when the assignment is locked to hardware or the diagram IS the configuration document.

---

## DRC Suppression with `@suppress`

```patch
# Loopback cable — suppress direction violation:
connect Console.Dante_Pri_Out[1..2] -> Console.Dante_Pri_In[65..66] {
  @suppress(direction)
  cable: "Cat6a_Loopback"
}

# Test rig — suppress multiple layers:
connect Recorder.SDI_Out -> Embedder.SDI_In {
  @suppress(mechanical, logical)
  cable: "BNC_Test"
}
```

Always prefer naming the specific layer so other checks still run. `@suppress(all)` as a last resort.

---

## Slot / Card Installation

```patch
template Venue_FOH_Rack {
  meta { manufacturer: "AVID", model: "Venue FOH Rack" }
  ports { AES_In[1..2]: in(XLR) [AES3] }
  slot Expansion[1..3]: HDX_Card
  slot Snake[1..2]: MADI_Card
}

instance FOH_Engine is Venue_FOH_Rack {
  slot Expansion[1]: "HDX_Card"
  slot Expansion[2]: "HDX_Card"
  slot Snake[1]: "MADI_Card"
  slot Snake[2]: "MADI_Card"
  route MADI_In[41] -> LINE[1]   # MADI_In comes from the installed MADI_Card
}
```

---

## Split Directional Ports (Dante with `bridge`)

```patch
template Rio1608_D2 {
  ports {
    Dante_Pri_In[1..16]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..16]: out(etherCON) [Dante, primary]
    Mic_In[1..16]: in(XLR)
    Line_Out[1..8]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri_Out    # mics → Dante transmit
  bridge Dante_Pri_In -> Line_Out   # Dante receive → line outputs
}
```

---

## Config Block (Channel Labels with Metadata)

```patch
config Drums {
  label Stage_Box_Inputs[1]: "Kick In"  { phantom: "true", stand: "Short Boom" }
  label Stage_Box_Inputs[2]: "Kick Out" { stand: "Short Boom" }
  label Stage_Box_Inputs[5]: "Hats"     { phantom: "true" }
  label Stage_Box_Inputs[9]             # intentionally unlabeled (omit label string)
}
```

---

## Template Composition (Subsystem)

```patch
template Monitor_Rig {
  meta { category: "Subsystem" }
  ports { Dante_In[1..48]: in(etherCON) [Dante, primary] }

  instance Console is PM5D
  instance IEM_WL is PSM1000
  instance IEM_MD is PSM1000

  connect Console.Dante_Pri_Out[1..2] -> IEM_WL.Dante_In[1..2]
  connect Console.Dante_Pri_Out[3..4] -> IEM_MD.Dante_In[1..2]
}
```

Double-clicking `Monitor_Rig` on the canvas opens the internal view. Same mechanism as multi-file projects.

## Relation to Other Wiki Pages

- [[language-reference]] — grammar for every construct shown above
- [[drc-rules]] — error codes referenced in suppression examples
- [[design-decisions]] — rationale for syntax choices (D005 bridge/route, D017 bus outputs)
- [[project-structure]] — multi-file project layout details
