# PatchLang Design Decisions

A running log of significant design decisions — what we chose, what we rejected, and why. Exists so we don't re-litigate settled questions.

---

## Format

Each entry follows this structure:

**Date** | **Status** (decided / pending / revisit)
**Question:** What were we deciding?
**Decision:** What did we choose?
**Rejected alternative:** What did we not choose, and why?
**Rationale:** The reasoning that drove the decision.

---

## Decisions

---

### D001 — IT Infrastructure Scope
**2026-03-28** | **Decided**

**Question:** Should PatchLang model IT network infrastructure — Ethernet switches, switch ports, VLANs, network topology?

**Decision:** Out of scope. PatchLang does not model switches or network topology.

**Rejected alternative:** Lightweight infrastructure nodes (e.g., `instance Core_Switch is ManagedSwitch`) that could document which switch port each device connects to, enabling DRC checks like "primary and secondary Dante paths traverse separate switches."

**Rationale:** The target user is an AV engineer, not an IT/network engineer. Dante and other IP audio protocols are modeled as logical virtual networks — all Dante devices can connect to each other without explicitly routing through switch infrastructure. Pulling IT infrastructure into scope would significantly expand language complexity, attract a different user profile, and pull engineering effort away from AV signal flow. The DRC benefit (catching same-switch redundancy failures) does not justify the scope expansion for this audience.

---

### D002 — Dante Secondary Redundancy Modeling
**2026-03-28** | **Decided**

**Question:** Should Dante secondary redundancy ports be modeled as explicit `connect` statements (same as primary), or as annotation metadata on the primary connect?

**Decision:** Annotation metadata on the primary connect. Templates should not declare `Dante_Sec_In` / `Dante_Sec_Out` ports.

```patch
connect Stage.Dante_Pri_Out[1..32] -> Console.Dante_Pri_In[1..32] {
  cable: "Cat6a_Pri"
  redundant_cable: "Cat6a_Sec"
}
```

**Rejected alternative:** Explicit secondary ports declared in templates and wired with full `connect` statements — treating secondary identical to primary in the language model.

**Rationale:** The only compelling argument for explicit secondary connects was DRC validation of switch topology (verifying that primary and secondary paths traverse separate physical switches). That argument collapsed when we decided IT infrastructure is out of scope (D001) — if PatchLang doesn't model switches, there is nothing to validate secondary connects against. With that DRC benefit gone, explicit secondary connects impose a 2x verbosity cost (40 connects for a 10-device system instead of 20) for zero analytical benefit. Secondary Dante carries no independent signal — it is a mirror of primary managed automatically by Dante Controller, requiring no engineering decisions. It is a property of the primary connection, not a new signal relationship, and should be modeled as such.

---

### D003 — WordClock Port Direction
**2026-03-28** | **Pending — awaiting Reid's input**

**Question:** Should WordClock ports use `io` (current spec) or split `in`/`out`?

**Context:** WordClock is physically directional (master → slaves, separate BNC connectors for In and Out on most gear). Using `in`/`out` would enable DRC to catch topology errors. The complication is that some devices can be either master or slave depending on configuration.

**See also:** D004 (AVB/Milan — same class of question)

---

### D004 — AVB/Milan Port Direction
**2026-03-28** | **Pending — awaiting Reid's input**

**Question:** Should AVB/Milan ports use `io` (current spec) or split `in`/`out`?

**Context:** Same question as WordClock but with a different physical reality — AVB/Milan devices typically have one Ethernet jack carrying both directions simultaneously (unlike WordClock's separate BNC connectors). Devices can be simultaneous Talkers and Listeners.

**See also:** D003 (WordClock — same class of question)

---

### D005 — `bridge` vs `route` Semantics
**2026-03-28** | **Decided**

**Question:** What should `bridge` mean — the physical/logical axis ("no physical cable") or the fixed/configurable axis ("manufacturer-hardwired guarantee")? And what distinguishes `bridge` from `route`?

**Decision:** Fixed/configurable axis. Two distinct scopes:

| Scope | Keyword | Meaning | Probe v2 behavior |
|---|---|---|---|
| Inside `template` | `bridge Mic_In -> Dante_Pri_Out` | Signal path guaranteed by device design. Exists in every unit of this template regardless of software configuration. DRC treats as invariant. | Do NOT push — it is fixed hardware behavior |
| Inside `instance` | `route Dante_In[1] -> Line_Out[3]` | Operator-configured routing state for this specific device. May change between shows. | Push via SCP / Ember+ / AES70 / Q-SYS in Probe v2 |
| Top-level between instances | `bridge Stage.Mic_In[1..16] -> Console.Dante_Pri_In[1..16]` | System designer's DRC assertion. "For signal tracing, treat this as a guaranteed path." | Read-only — documents logical signal flow, not pushed |

**Rejected alternative:** Physical/logical axis — `bridge` = "no physical cable" regardless of whether the path is hardwired or operator-configured. Signal Trace works correctly under either axis. The physical/logical axis fails specifically when Probe v2 pushes configuration to live hardware: under that model, all internal paths in a software-defined console (CL5, SD12) become `bridge`, giving Probe no way to distinguish "do not touch — hardwired" from "push this — operator-configured." That makes correct push implementation impossible.

**No new keyword needed.** The physical/logical axis concept — "all non-cable paths" — is fully covered by `bridge` + `route` together. Signal Trace traverses both. The only thing the physical/logical axis offered was collapsing them into one keyword, which is exactly what Probe v2 prevents.

**Rationale:** The Configuration Push feature (Probe v2) is decisive. When Probe pushes routing to live hardware, it must have a deterministic manifest of what to touch. `route` = push. `bridge` = do not touch. This round-trip — Probe reads live state → writes `route` in `.patch` → push reads `route` and sends to device — only works if the keyword carries the "is pushable" semantic. Additionally, Signal Trace gains richer annotation under this model: `bridge` hops can be labeled "guaranteed by hardware design" while `route` hops can be labeled "depends on current operator configuration" — information that is genuinely useful when tracing a fault at 11pm.

**Implications for existing spec:**
- The `language-reference.md` definition of `bridge` ("logical signal mapping between ports, no physical cable") must be updated to reflect fixed/configurable semantics.
- `compiler.md` must add a section defining the semantic contract for `bridge` vs `route`.
- The PatchLang skill (`SKILL.md`) must be updated — LLMs generating device templates need to know: use `bridge` only for manufacturer-hardwired paths; operator-configurable internal routing belongs in the instance as `route`.
- Existing template examples (stageboxes using `bridge Mic_In -> Dante_Pri_Out`) are correct — these are manufacturer-hardwired paths.
- Consoles with fully software-defined internal routing (CL5, SD12) should have no `bridge` declarations in their templates; routing state is documented as `route` in instances.

---

### D006 — Range Size Mismatch in connect
**2026-03-29** | **Decided**

**Question:** What should happen when the left and right sides of a `connect` range have different sizes?

```patch
connect Stage.Dante_Pri_Out[1..16] -> Console.Dante_Pri_In[1..8]  # 16 ≠ 8
```

**Decision:** Hard compile error (DRC code `S15`, Structural layer, severity: Error). The file does not compile until the engineer explicitly resolves the mismatch.

**Rejected alternatives:**
- *Warning-only:* Engineers under time pressure (show in 2 hours) dismiss warnings. A warning on a mismatched range produces a file that appears valid while Signal Trace outputs are silently wrong for the unmatched channels.
- *Silent truncation to smaller side:* Introduces two ambiguities — which end truncates, and whether the mismatch was intentional. Produces confidently wrong DRC output ("channels 9–16 are unconnected") with no flag to the engineer. This is the most dangerous option.
- *Defined `partial` keyword:* A valid long-term enhancement but not needed now. YAGNI — hard error is the safe default, and `partial` can be added deliberately when the use case is well-understood.

**Rationale:** The cost asymmetry is decisive. A hard error costs 30 seconds: read the message, fix the typo, recompile. A silent wrong costs a show: engineer patches channels 9–16, hears nothing, debugs Dante Controller, SFP, console routing — never opening the `.patch` file because it compiled successfully. Signal Trace reliability depends on correct range semantics at every hop; a silently-truncated connect poisons the trace.

The `@suppress(structural)` mechanism already handles intentional deviation from structural rules. An engineer documenting a partial system (32 channels into a 64-input console) can write:

```patch
connect Stage.Dante_Pri_Out[1..32] -> Console.Dante_Pri_In[1..32] {
  @suppress(structural)   # intentional: console has 64 inputs, only 32 wired this show
}
```

This forces explicit intent at zero extra cost to engineers who know what they're doing, while protecting engineers who made a typo.

**Analogy:** VHDL and Verilog both hard-error on port-width mismatches by default.

**Affects:** `compiler.md` DRC table, `language-reference.md` DRC table (new S15 entry).

---

### D007 — Import Aliasing
**2026-03-29** | **Decided**

**Question:** Should PatchLang's `use` statement support `as` aliasing to resolve template name collisions between libraries?

```patch
# Proposed aliasing syntax (rejected for now)
use yamaha { CL5 as Yamaha_CL5 }
use corporate.racks { Patch_Bay as Corp_Patch_Bay }
```

**Decision:** Defer aliasing. Do not add `as` aliasing syntax now. Codify the naming convention as a spec requirement instead. Revisit if a confirmed real-world collision is reported.

**Rejected alternative:** Add `as` aliasing to the `use` statement immediately. The for case was structurally sound — without aliasing, a collision between two third-party libraries is an unresolvable hard error — but YAGNI wins: zero collisions exist in the current library, and the naming convention demonstrably prevents them.

**Naming convention (now required, not just advisory):** Template names in shared libraries must use a manufacturer prefix or model number — not generic names. `CL5`, `Rio3224`, `SD12`, `5601MSC` are correct. `Patch_Bay`, `Power_Amp`, `Line_Level_Converter` are not acceptable as standalone names in a shared library; they must be prefixed (`Neutrik_Patch_Bay`, `Yamaha_Power_Amp`). Generic names are acceptable only in project-local templates that are never published as a library.

**Future escape hatch:** If collision support ever becomes necessary, the preferred design is **qualified references** (`yamaha::CL5`) rather than `as` aliasing. Qualified references preserve the original template identity at every use site — which matters for both human readers and LLM code generation — while still disambiguating. They align naturally with the existing library path structure (`lib/audio/yamaha.patch`). This is not in scope now but is the right direction if the language grows there.

**Rationale:** Aliasing creates a local fiction — `Corp_Patch_Bay` exists in one file and nowhere else, severing context for every engineer who reads the project later. It also rewards bad library naming. Model numbers are globally unique by industrial convention. The existing library has no collisions. Adding aliasing now solves a hypothetical at real readability and LLM-generability cost.

**Affects:** `language-reference.md` — strengthen naming convention from advisory to required in the `use`/import section.

---

### D008 — WordClock Port Direction
**2026-03-29** | **Decided** (input from Reid, AV engineer)

**Question:** Should WordClock ports use `io` (current spec) or split `in`/`out`?

**Decision:** Split `in`/`out`. WordClock ports are directional and must be declared as separate ports:

```patch
WordClock_In:  in(BNC_75)  [WordClock]
WordClock_Out: out(BNC_75) [WordClock]
```

Devices that are always clock masters (SPGs, grandmaster appliances) declare only `WordClock_Out`. Devices that are always clock slaves declare only `WordClock_In`. Devices that can be either (e.g., a console that can be master or slave) declare both.

**Rejected alternative:** `io(BNC_75) [WordClock]` — the current spec default.

**Rationale (from Reid, AV engineer):** "I've never seen a BNC that is bidirectional." WordClock uses separate physical 75Ω BNC connectors for input and output on every device. The `io` classification was wrong from the start — it implies a shared bidirectional connector that does not exist in the real world. Splitting to `in`/`out` also enables the DRC to catch real wiring errors: two clock outputs connected together (two masters fighting), or a device with no clock input connected (unsynced).

**Note on embedded clock in protocols:** MADI, Dante, and other audio protocols carry a word clock signal implicitly inside the protocol stream. This does not require a separate WordClock port — it rides along with the existing `Dante_Pri_In`/`Dante_Pri_Out` or `MADI_In`/`MADI_Out` ports. No change needed for protocol-embedded clocking.

**Affects:** `compiler.md` IO direction model table, `SKILL.md` Critical Rule #1, generated fixture files (`focusrite-rednet-a16r.patch`, `evertz-5601msc.patch`).

---

### D009 — PTPv2 (IEEE 1588) Port Modeling
**2026-03-29** | **Decided** (research-based)

**Question:** Does PTPv2 — used by AES67, SMPTE 2110, Ravenna, Q-LAN — need its own dedicated port type in PatchLang?

**Decision:** No new port type. PTP is represented as:
1. A protocol attribute tag on existing Ethernet ports: `[AES67, PTP]` or `[SMPTE_2110]`
2. Instance-level metadata for grandmaster role and domain configuration:

```patch
instance House_GM is Evertz_5700MSC_IP {
  location: "Master Clock Rack"
  ip: "10.0.1.10"
  ptp_role: "grandmaster"
  ptp_domain: 127
  ptp_priority1: 0
}
```

**Rejected alternative:** A dedicated `PTP_Out` port on grandmaster devices or `PTP_In` on slave devices.

**Rationale:** There is no physical "PTP In" or "PTP Out" connector on any AV device. PTP is a multicast UDP protocol (ports 319/320) that runs entirely inside the Ethernet layer over the same jack used for audio/video media. Grandmaster/slave roles are elected dynamically at runtime via the Best Master Clock Algorithm (BMCA) — they are not fixed at patch time. Adding a `PTP_Out` port would fabricate a physical port that does not exist and imply that slaves need a dedicated PTP cable routed from the grandmaster, which is false.

NMOS IS-04/IS-09 (the industry standard for IP broadcast device registration) models PTP as device registration metadata and system configuration parameters — not as port connections. This confirms the correct model: PTP domain membership is a configuration attribute on the device, not a signal path between devices.

**Multi-domain edge case:** In facilities running both Dante (PTPv2 domain 0) and SMPTE 2110 (PTPv2 domain 127) on the same network, domain membership can be documented with protocol attribute tags: `[SMPTE_2110, PTP_domain_127]`. This is an enhancement for future consideration — not required for MVP.

**Note on Dante and PTP:** Dante uses PTPv1 internally. When a Dante domain is placed in AES67 mode, it bridges to PTPv2 domain 0. When in SMPTE mode, it bridges to PTPv2 domain 127. This cross-domain bridging is handled by Dante Domain Manager and does not affect PatchLang port modeling.

---

### D010 — Intercom Port Modeling Scope
**2026-03-29** | **Decided** (input from Reid)

**Question:** Which intercom ports should be modeled as signal flow edges in SignalCanvas? Should headset/partyline XLR ports split `in`/`out` like Dante? Should management/control ports be modeled at all?

**Decision:** Model the matrix and physical signal sources only. Three tiers:

1. **Intercom matrices** (Eclipse HX, Artist, ADAM-M) — model fully. These are routing devices and belong in the signal graph.

2. **Panel physical audio inputs** that source into the matrix — model as `in()` ports. Example: a mic or program input on a panel that feeds audio up to the matrix and out to the rest of the system. These are real signal origins.

3. **Headset/monitoring ports, management LAN, control interfaces** — do not model. Headset connections are local user I/O, not system signal flow. Control interfaces (`LAN: io(RJ45)`) are infrastructure. Neither is something you would draw a cable to in SignalCanvas.

**On partyline loops from a matrix:** A matrix's `Partyline[1..4]: io(XLR)` ports connect to beltpacks via physical XLR cables — these ARE signal flow edges and should be split `in`/`out` on the matrix template.

**Rejected alternative:** Modeling every panel and beltpack port as a first-class signal flow edge. Reid: "I would mostly be keen to only document the matrix." Panels appear as endpoints, not routing nodes.

**On control interfaces:** Reid confirmed: "I wasn't thinking we would document control interfaces." Consistent with D001 (IT infrastructure out of scope).

**Rationale:** SignalCanvas documents signal flow paths that an AV engineer cares about tracing — sources, routes, and destinations. A headset plugged into a beltpack is a local I/O connection for the operator wearing it, not a system signal path. The matrix is the signal routing hub; that is what gets documented.

**Implications for library files:**
- `Eclipse_HX.Partyline[1..4]: io(XLR)` → split to `Partyline_In[1..4]` / `Partyline_Out[1..4]`
- `V12.Headset: io(XLR)` → leave as `io()` or omit
- `LAN: io(RJ45)` (all devices) → leave as `io()`, excluded from signal graph

**See also:** D004 (AVB/Milan — same class of question, still pending)
