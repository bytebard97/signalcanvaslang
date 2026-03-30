// packages/demo/src/lib/examples.ts
// Bundled PatchLang example projects for the demo app.

export interface SingleFileExample {
  kind: 'single'
  name: string
  description: string
  source: string
}

export interface MultiFileExample {
  kind: 'multi'
  name: string
  description: string
  files: Record<string, string>
  entry: string
}

export type Example = SingleFileExample | MultiFileExample

export const EXAMPLES: Example[] = [
  {
    kind: 'single',
    name: 'Blank Starter',
    description: 'Minimal template to start from scratch',
    source: `# Blank Starter
# Replace this with your own devices and connections.

template MyDevice {
  ports {
    Input[1..2]: in(XLR)
    Output[1..2]: out(XLR)
  }
  bridge Input -> Output
}

instance Device_A is MyDevice {
  location: "Location A"
}

instance Device_B is MyDevice {
  location: "Location B"
}

connect Device_A.Output[1] -> Device_B.Input[1]
`,
  },
  {
    kind: 'single',
    name: 'Broadcast Truck',
    description: 'Mobile broadcast production vehicle',
    source: `# ============================================================
# Broadcast Truck — Live Production OB Van
# A mobile production unit with cameras, video router,
# encoder, and sync generator for live broadcast.
# ============================================================

# --- Templates ------------------------------------------------

template Camera {
  meta {
    manufacturer: "Sony"
    model: "HDC_3500"
    category: "Camera"
  }
  ports {
    SDI_Out[1..4]: out(BNC_75) [SDI, UHD]
    Return_In[1..2]: in(BNC_75) [SDI, HD]
    Genlock_In: in(BNC_75) [analog, sync]
  }
}

template VideoRouter {
  meta {
    manufacturer: "Ross"
    model: "Ultrix"
    category: "Router"
  }
  ports {
    SDI_In[1..72]: in(BNC_75) [SDI]
    SDI_Out[1..72]: out(BNC_75) [SDI]
    Ref_In: in(BNC_75) [BlackBurst, TriLevel]
  }
  bridge SDI_In -> SDI_Out
}

template Encoder {
  meta {
    manufacturer: "Harmonic"
    model: "VOS360"
    category: "Encoder"
  }
  ports {
    SDI_In[1..4]: in(BNC_75) [SDI, UHD]
    IP_Out: out(SFP_Plus) [SMPTE_2110]
  }
  bridge SDI_In -> IP_Out
}

template SyncGenerator {
  meta {
    manufacturer: "Evertz"
    model: "5601MSC"
    category: "Sync"
  }
  ports {
    Ref_Out[1..16]: out(BNC_75) [BlackBurst, TriLevel]
    GPS_In: in(BNC_75) [GPS]
  }
}

template Multiviewer {
  meta {
    manufacturer: "Evertz"
    model: "EQX_MV"
    category: "Monitoring"
  }
  ports {
    SDI_In[1..16]: in(BNC_75) [SDI]
    HDMI_Out[1..4]: out(HDMI) [UHD]
  }
}

# --- Instances ------------------------------------------------

instance Cam1 is Camera {
  location: "Camera Position 1"
  operator: "Camera 1"
  ip: "10.0.1.11"
}

instance Cam2 is Camera {
  location: "Camera Position 2"
  operator: "Camera 2"
  ip: "10.0.1.12"
}

instance Cam3 is Camera {
  location: "Camera Position 3"
  operator: "Camera 3"
  ip: "10.0.1.13"
}

instance Router is VideoRouter {
  location: "Truck Rack A"
  ip: "10.0.1.100"
}

instance Enc1 is Encoder {
  location: "Truck Rack B"
  ip: "10.0.1.200"
}

instance SyncGen is SyncGenerator {
  location: "Truck Rack A"
  ip: "10.0.1.50"
}

instance MV1 is Multiviewer {
  location: "Truck Monitoring Wall"
  ip: "10.0.1.60"
}

# --- Connections: Cameras to Router ---------------------------

connect Cam1.SDI_Out[1] -> Router.SDI_In[1] {
  cable: "SDI_C1_PGM"
  length: "50m"
  signal_type: "UHD"
}

connect Cam1.SDI_Out[2] -> Router.SDI_In[2] {
  cable: "SDI_C1_ISO"
  length: "50m"
}

connect Cam2.SDI_Out[1] -> Router.SDI_In[3] {
  cable: "SDI_C2_PGM"
  length: "40m"
}

connect Cam2.SDI_Out[2] -> Router.SDI_In[4] {
  cable: "SDI_C2_ISO"
  length: "40m"
}

connect Cam3.SDI_Out[1] -> Router.SDI_In[5] {
  cable: "SDI_C3_PGM"
  length: "60m"
}

connect Cam3.SDI_Out[2] -> Router.SDI_In[6] {
  cable: "SDI_C3_ISO"
  length: "60m"
}

# --- Connections: Router to Return Video ----------------------

connect Router.SDI_Out[61] -> Cam1.Return_In[1] {
  cable: "RET_C1"
  signal_type: "PGM_Return"
}

connect Router.SDI_Out[62] -> Cam2.Return_In[1] {
  cable: "RET_C2"
  signal_type: "PGM_Return"
}

connect Router.SDI_Out[63] -> Cam3.Return_In[1] {
  cable: "RET_C3"
  signal_type: "PGM_Return"
}

# --- Connections: Router to Encoder ---------------------------

connect Router.SDI_Out[1] -> Enc1.SDI_In[1] {
  cable: "ENC_PGM"
  signal_type: "Program"
}

connect Router.SDI_Out[2] -> Enc1.SDI_In[2] {
  cable: "ENC_CLN"
  signal_type: "Clean_Feed"
}

# --- Connections: Router to Multiviewer -----------------------

connect Router.SDI_Out[41] -> MV1.SDI_In[1] {
  cable: "MV_1"
}

connect Router.SDI_Out[42] -> MV1.SDI_In[2] {
  cable: "MV_2"
}

connect Router.SDI_Out[43] -> MV1.SDI_In[3] {
  cable: "MV_3"
}

connect Router.SDI_Out[44] -> MV1.SDI_In[4] {
  cable: "MV_4"
}

connect Router.SDI_Out[45] -> MV1.SDI_In[5] {
  cable: "MV_5"
}

connect Router.SDI_Out[46] -> MV1.SDI_In[6] {
  cable: "MV_6"
}

# --- Genlock Chain --------------------------------------------

connect SyncGen.Ref_Out[1] -> Cam1.Genlock_In {
  cable: "GL_C1"
}

connect SyncGen.Ref_Out[2] -> Cam2.Genlock_In {
  cable: "GL_C2"
}

connect SyncGen.Ref_Out[3] -> Cam3.Genlock_In {
  cable: "GL_C3"
}

connect SyncGen.Ref_Out[4] -> Router.Ref_In {
  cable: "GL_Router"
}

# --- Bridges (logical signal paths) --------------------------

bridge Cam1.SDI_Out[1] -> Router.SDI_In[1]
bridge Cam2.SDI_Out[1] -> Router.SDI_In[3]
bridge Cam3.SDI_Out[1] -> Router.SDI_In[5]
bridge Router.SDI_Out[1] -> Enc1.SDI_In[1]

# --- Signals --------------------------------------------------

signal PGM_Output {
  description: "Main program output"
}

signal ISO_Cam1 {
  origin: Cam1.SDI_Out[2]
  description: "Camera 1 isolated recording feed"
}

signal ISO_Cam2 {
  origin: Cam2.SDI_Out[2]
  description: "Camera 2 isolated recording feed"
}

signal ISO_Cam3 {
  origin: Cam3.SDI_Out[2]
  description: "Camera 3 isolated recording feed"
}

# --- Flags ----------------------------------------------------

flag Genlock_OK {
  description: "All cameras locked to house sync"
  severity: "info"
}
`,
  },
  {
    kind: 'single',
    name: 'Worship Venue',
    description: 'Church audio/video system',
    source: `# ============================================================
# Worship Venue — Dante Audio Network
# A typical house-of-worship audio system with Yamaha stageboxes,
# a CL5 mixing console, and a Dante network switch.
# ============================================================

# --- Templates ------------------------------------------------

template Rio3224 {
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
}

template CL5 {
  meta {
    manufacturer: "Yamaha"
    model: "CL5"
    category: "Console"
  }
  ports {
    Dante_Pri_In[1..72]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..24]: out(etherCON) [Dante, primary]
    Dante_Sec_In[1..72]: in(etherCON) [Dante, secondary]
    Dante_Sec_Out[1..24]: out(etherCON) [Dante, secondary]
    Mix_Bus[1..24]: out
  }
}

template GigabitSwitch {
  meta {
    manufacturer: "Cisco"
    model: "SG350"
    category: "Network"
  }
  ports {
    Port[1..24]: io(RJ45) [Ethernet, Gigabit]
  }
}

# --- Instances ------------------------------------------------

instance Stage_Left is Rio3224 {
  location: "Stage Left Wing"
  ip: "192.168.1.31"
}

instance Stage_Right is Rio3224 {
  location: "Stage Right Wing"
  ip: "192.168.1.32"
}

instance FOH_Console is CL5 {
  location: "Front of House"
  ip: "192.168.1.10"
}

instance Dante_Switch is GigabitSwitch {
  location: "FOH Rack"
  ip: "192.168.1.1"
}

# --- Connections (physical cables) ----------------------------

connect Stage_Left.Dante_Pri_Out -> Dante_Switch.Port[1] {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}
connect Dante_Switch.Port[1] -> Stage_Left.Dante_Pri_In

connect Stage_Left.Dante_Sec_Out -> Dante_Switch.Port[2] {
  cable: "Cat6a_SL_Sec"
  length: "30m"
}
connect Dante_Switch.Port[2] -> Stage_Left.Dante_Sec_In

connect Stage_Right.Dante_Pri_Out -> Dante_Switch.Port[3] {
  cable: "Cat6a_SR_Pri"
  length: "25m"
}
connect Dante_Switch.Port[3] -> Stage_Right.Dante_Pri_In

connect Stage_Right.Dante_Sec_Out -> Dante_Switch.Port[4] {
  cable: "Cat6a_SR_Sec"
  length: "25m"
}
connect Dante_Switch.Port[4] -> Stage_Right.Dante_Sec_In

connect Dante_Switch.Port[5] -> FOH_Console.Dante_Pri_In {
  cable: "Cat6a_FOH_Pri"
  length: "3m"
}
connect FOH_Console.Dante_Pri_Out -> Dante_Switch.Port[5]

connect Dante_Switch.Port[6] -> FOH_Console.Dante_Sec_In {
  cable: "Cat6a_FOH_Sec"
  length: "3m"
}
connect FOH_Console.Dante_Sec_Out -> Dante_Switch.Port[6]

# --- Bridges (logical signal mapping) -------------------------

bridge Stage_Left.Mic_In[1..32] -> FOH_Console.Dante_Pri_In[1..32]
bridge Stage_Right.Mic_In[1..16] -> FOH_Console.Dante_Pri_In[33..48]

# --- Signals --------------------------------------------------

signal Lead_Vocal {
  origin: Stage_Left.Mic_In[1]
  channel: "1"
  description: "Worship leader vocal"
}

signal Kick_Drum {
  origin: Stage_Left.Mic_In[5]
  channel: "5"
  description: "Kick drum mic"
}

signal Snare_Top {
  origin: Stage_Left.Mic_In[6]
  channel: "6"
  description: "Snare top mic"
}

signal Pastor_Lav {
  origin: Stage_Right.Mic_In[1]
  channel: "33"
  description: "Pastor lavalier"
}
`,
  },
  {
    kind: 'single',
    name: 'Concert Venue — Hierarchical',
    description: '5,000-seat concert hall with nested subsystems (drill-down demo)',
    source: `# ============================================================
# Concert Venue — Hierarchical Layout
# A 5,000-seat concert hall organized as nested subsystems.
#
# Hierarchy:
#   Root
#   ├── Audio_FOH (FOH Console, Recording, Amplification)
#   │   └── Amplification (4 amp racks — 3rd nesting level)
#   ├── Audio_Stage (Monitor Console, Stageboxes, Wireless, Optocore)
#   ├── Audio_Net (Dante network switches)
#   ├── Video (Cameras, Switcher, LED Processor, Media Server)
#   ├── Comms (Intercom Frame + Keypanels)
#   └── House_Sync (Sync Generator)
# ============================================================


# ==== DEVICE TEMPLATES ======================================

template DiGiCo_SD7 {
  meta {
    manufacturer: "DiGiCo"
    model: "SD7"
    category: "Console"
  }
  ports {
    Optocore_Tx: out(OpticalCon) [Optocore, primary]
    Optocore_Rx: in(OpticalCon) [Optocore, secondary]
    Local_In[1..12]: in(XLR)
    Local_Out[1..12]: out(XLR)
    AES_Out[1..8]: out(BNC_75) [AES3]
    MADI_Out: out(BNC_75) [MADI, ch64]
  }
}

template SD_Rack {
  meta {
    manufacturer: "DiGiCo"
    model: "SD-Rack"
    category: "Stagebox"
  }
  ports {
    Optocore_Rx: in(OpticalCon) [Optocore, primary]
    Optocore_Tx: out(OpticalCon) [Optocore, secondary]
    Mic_In[1..56]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Optocore_Tx
}

template Lab_gruppen_PLM {
  meta {
    manufacturer: "Lab.gruppen"
    model: "PLM_20K44"
    category: "Amplifier"
  }
  ports {
    AES_In[1..4]: in(XLR) [AES3]
    Speaker_Out[1..4]: out(SpeakON)
  }
  bridge AES_In -> Speaker_Out
}

template Shure_Axient {
  meta {
    manufacturer: "Shure"
    model: "Axient_Digital_AD4Q"
    category: "Wireless"
  }
  ports {
    Dante_Pri: out(etherCON) [Dante, primary]
    Dante_Sec: out(etherCON) [Dante, secondary]
    Analog_Out[1..4]: out(XLR)
    Antenna_In[1..2]: in(BNC_50) [RF]
  }
  bridge Antenna_In -> Analog_Out
}

template OptoSplitter {
  meta {
    manufacturer: "DiGiCo"
    model: "Optocore_DD32R"
    category: "Network"
  }
  ports {
    Loop_In: in(OpticalCon) [Optocore]
    Loop_Out: out(OpticalCon) [Optocore]
    MADI_Out: out(BNC_75) [MADI, ch64]
  }
}

template Barco_E2 {
  meta {
    manufacturer: "Barco"
    model: "E2"
    category: "Video Processor"
  }
  ports {
    SDI_In[1..8]: in(BNC_75) [SDI, G3]
    HDMI_In[1..4]: in(HDMI)
    DVI_Out[1..8]: out(DVI) [LED_Wall]
    SDI_Out[1..4]: out(BNC_75) [SDI, G3]
  }
}

template PTZ_Camera {
  meta {
    manufacturer: "Panasonic"
    model: "AW_UE150"
    category: "Camera"
  }
  ports {
    SDI_Out: out(BNC_75) [SDI, UHD]
    HDMI_Out: out(HDMI) [UHD]
    NDI_Out: out(etherCON) [NDI]
    Genlock_In: in(BNC_75) [TriLevel]
  }
}

template ATEM_Constellation {
  meta {
    manufacturer: "Blackmagic"
    model: "ATEM_Constellation_8K"
    category: "Switcher"
  }
  ports {
    SDI_In[1..20]: in(BNC_75) [SDI, G3]
    SDI_Out[1..12]: out(BNC_75) [SDI, G3]
    SDI_Aux[1..6]: out(BNC_75) [SDI, G3]
  }
  bridge SDI_In -> SDI_Out
}

template Disguise_GX2C {
  meta {
    manufacturer: "Disguise"
    model: "gx_2c"
    category: "Media Server"
  }
  ports {
    SDI_In[1..2]: in(BNC_75) [SDI, G3]
    SDI_Out[1..4]: out(BNC_75) [SDI, UHD]
    DisplayPort_Out[1..4]: out(DP) [UHD]
    Dante_Out: out(etherCON) [Dante]
    NDI_Out: out(etherCON) [NDI]
  }
}

template RTS_ADAM {
  meta {
    manufacturer: "RTS"
    model: "ADAM_M"
    category: "Intercom"
  }
  ports {
    Trunk[1..8]: out(etherCON) [OMNEO]
    AES_In[1..4]: in(XLR) [AES3]
    AES_Out[1..4]: out(XLR) [AES3]
    GPIO_In[1..4]: in(DB25)
    GPIO_Out[1..4]: out(DB25)
  }
}

template RTS_KP {
  meta {
    manufacturer: "RTS"
    model: "KP_5032"
    category: "Intercom Panel"
  }
  ports {
    Trunk: in(etherCON) [OMNEO]
    Headset: out(XLR5)
  }
}

template Luminex_GigaCore {
  meta {
    manufacturer: "Luminex"
    model: "GigaCore_26i"
    category: "Network Switch"
  }
  ports {
    Uplink[1..12]: in(etherCON) [Ethernet, Gigabit]
    Downlink[1..12]: out(etherCON) [Ethernet, Gigabit]
    SFP_In: in(SFP) [Ethernet, G10]
    SFP_Out: out(SFP) [Ethernet, G10]
  }
}

template Sync_Generator {
  meta {
    manufacturer: "Evertz"
    model: "5601MSC"
    category: "Sync"
  }
  ports {
    BB_Out[1..8]: out(BNC_75) [BlackBurst]
    TriLevel_Out[1..8]: out(BNC_75) [TriLevel]
    LTC_Out[1..4]: out(XLR) [Timecode]
    GPS_In: in(BNC_75) [GPS]
    WordClock_Out[1..4]: out(BNC_75) [WordClock]
  }
}


# ==== SUBSYSTEM TEMPLATES (Level 2) =========================

# --- Amplification Sub-Subsystem (Level 3) ------------------
# Groups all 4 amp racks — drills down from AudioFOH

template Amplification {
  ports {
    AES_In[1..8]: in(XLR) [AES3]
    Speaker_Main_L[1..4]: out(SpeakON)
    Speaker_Main_R[1..4]: out(SpeakON)
    Speaker_Subs[1..4]: out(SpeakON)
    Speaker_Fill[1..4]: out(SpeakON)
  }

  instance Main_L is Lab_gruppen_PLM {
    location: "Main PA Left Amp Room"
  }
  instance Main_R is Lab_gruppen_PLM {
    location: "Main PA Right Amp Room"
  }
  instance Subs is Lab_gruppen_PLM {
    location: "Sub Array Amp Room"
  }
  instance Front_Fill is Lab_gruppen_PLM {
    location: "Front Fill Amp Room"
  }

  # AES feeds from FOH console (routed through subsystem port)
  connect AES_In[1] -> Main_L.AES_In[1]
  connect AES_In[2] -> Main_L.AES_In[2]
  connect AES_In[3] -> Main_R.AES_In[1]
  connect AES_In[4] -> Main_R.AES_In[2]
  connect AES_In[5] -> Subs.AES_In[1]
  connect AES_In[6] -> Subs.AES_In[2]
  connect AES_In[7] -> Front_Fill.AES_In[1]
  connect AES_In[8] -> Front_Fill.AES_In[2]

  # Speaker outputs exposed at subsystem boundary
  connect Main_L.Speaker_Out[1..4] -> Speaker_Main_L[1..4]
  connect Main_R.Speaker_Out[1..4] -> Speaker_Main_R[1..4]
  connect Subs.Speaker_Out[1..4] -> Speaker_Subs[1..4]
  connect Front_Fill.Speaker_Out[1..4] -> Speaker_Fill[1..4]
}

# --- Audio FOH Subsystem ------------------------------------
# FOH console, recording rack, and amplification

template AudioFOH {
  ports {
    Optocore_In: in(OpticalCon) [Optocore]
    Optocore_Out: out(OpticalCon) [Optocore]
    WordClock_In: in(BNC_75) [WordClock]
    AES_Out[1..2]: out(BNC_75) [AES3]
    Speaker_Main_L[1..4]: out(SpeakON)
    Speaker_Main_R[1..4]: out(SpeakON)
    Speaker_Subs[1..4]: out(SpeakON)
    Speaker_Fill[1..4]: out(SpeakON)
  }

  instance Console is DiGiCo_SD7 {
    location: "FOH Mix Position"
    ip: "10.10.1.10"
  }
  instance Rec_Rack is SD_Rack {
    location: "FOH Rack Room"
    ip: "10.10.1.11"
  }
  instance Amps is Amplification

  # Optocore ring endpoints
  connect Optocore_In -> Console.Optocore_Rx
  connect Console.Optocore_Out -> Optocore_Out

  # MADI recording feed
  connect Console.MADI_Out -> Rec_Rack.Optocore_Rx {
    cable: "MADI_FOH_Rec"
    length: "3m"
  }

  # WordClock from house sync
  connect WordClock_In -> Console.Local_In[12]

  # AES to amplification sub-subsystem
  connect Console.AES_Out[1..8] -> Amps.AES_In[1..8]

  # AES program feed outputs (for comms/monitoring)
  connect Console.AES_Out[7] -> AES_Out[1]
  connect Console.AES_Out[8] -> AES_Out[2]

  # Pass through speaker outputs
  connect Amps.Speaker_Main_L[1..4] -> Speaker_Main_L[1..4]
  connect Amps.Speaker_Main_R[1..4] -> Speaker_Main_R[1..4]
  connect Amps.Speaker_Subs[1..4] -> Speaker_Subs[1..4]
  connect Amps.Speaker_Fill[1..4] -> Speaker_Fill[1..4]
}

# --- Audio Stage Subsystem ----------------------------------
# Monitor console, stageboxes, wireless, optocore splitter

template AudioStage {
  ports {
    Optocore_In: in(OpticalCon) [Optocore]
    Optocore_Out: out(OpticalCon) [Optocore]
    Dante_Out[1..4]: out(etherCON) [Dante]
  }

  instance Mon_Console is DiGiCo_SD7 {
    location: "Stage Left — Monitor World"
    ip: "10.10.1.20"
  }
  instance Rack_A is SD_Rack {
    location: "Stage Left Pit"
    ip: "10.10.1.21"
  }
  instance Rack_B is SD_Rack {
    location: "Stage Right Pit"
    ip: "10.10.1.22"
  }
  instance Wireless_1 is Shure_Axient {
    location: "Stage Left — RF Rack"
    ip: "10.10.2.10"
  }
  instance Wireless_2 is Shure_Axient {
    location: "Stage Right — RF Rack"
    ip: "10.10.2.11"
  }
  instance Opto_Split is OptoSplitter {
    location: "Stage Left — Optocore Ring"
    ip: "10.10.3.10"
  }

  # Optocore ring (internal portion)
  connect Optocore_In -> Opto_Split.Loop_In
  connect Opto_Split.Loop_Out -> Mon_Console.Optocore_Rx
  connect Mon_Console.Optocore_Tx -> Rack_A.Optocore_Rx
  connect Rack_A.Optocore_Tx -> Rack_B.Optocore_Rx
  connect Rack_B.Optocore_Tx -> Optocore_Out

  # Wireless Dante feeds exposed for network routing
  connect Wireless_1.Dante_Pri -> Dante_Out[1]
  connect Wireless_1.Dante_Sec -> Dante_Out[2]
  connect Wireless_2.Dante_Pri -> Dante_Out[3]
  connect Wireless_2.Dante_Sec -> Dante_Out[4]

  # Monitor console sends mixes back to stage rack line outputs
  # (IEM feeds, sidefills, drum sub). Audio travels via Optocore ring.
  connect Mon_Console.Local_Out[1] -> Rack_A.Line_Out[1] {
    cable: "IEM_Mix_1"
  }
  connect Mon_Console.Local_Out[2] -> Rack_A.Line_Out[2] {
    cable: "IEM_Mix_2"
  }
  connect Mon_Console.Local_Out[3] -> Rack_A.Line_Out[3] {
    cable: "IEM_Mix_3"
  }
  connect Mon_Console.Local_Out[4] -> Rack_A.Line_Out[4] {
    cable: "IEM_Mix_4"
  }
  connect Mon_Console.Local_Out[5] -> Rack_B.Line_Out[1] {
    cable: "Sidefill_L"
  }
  connect Mon_Console.Local_Out[6] -> Rack_B.Line_Out[2] {
    cable: "Sidefill_R"
  }
  connect Mon_Console.Local_Out[7] -> Rack_B.Line_Out[3] {
    cable: "Drum_Sub"
  }
  connect Mon_Console.Local_Out[8] -> Rack_B.Line_Out[4] {
    cable: "Drum_Fill"
  }
}

# --- Audio Network Subsystem --------------------------------
# Dante network switches connecting FOH and stage

template AudioNetwork {
  ports {
    Dante_In[1..4]: in(etherCON) [Dante]
    FOH_SFP_In: in(SFP) [Ethernet, G10]
    FOH_SFP_Out: out(SFP) [Ethernet, G10]
  }

  instance SW_FOH is Luminex_GigaCore {
    location: "FOH Rack Room"
    ip: "10.10.3.1"
  }
  instance SW_Stage is Luminex_GigaCore {
    location: "Stage Left — Network Rack"
    ip: "10.10.3.2"
  }

  # Fiber trunk between switches
  connect SW_FOH.SFP_Out -> SW_Stage.SFP_In {
    cable: "SM_Fiber_NetTrunk"
    length: "60m"
  }

  # Dante device uplinks into stage switch
  connect Dante_In[1] -> SW_Stage.Uplink[1]
  connect Dante_In[2] -> SW_Stage.Uplink[2]
  connect Dante_In[3] -> SW_Stage.Uplink[3]
  connect Dante_In[4] -> SW_Stage.Uplink[4]

  # FOH-side SFP passthrough (for future expansion)
  connect FOH_SFP_In -> SW_FOH.SFP_In
  connect SW_FOH.SFP_Out -> FOH_SFP_Out
}

# --- Video Subsystem ----------------------------------------
# Cameras, switcher, LED processor, and media server

template VideoSystem {
  ports {
    Genlock_In[1..3]: in(BNC_75) [TriLevel]
    BB_In: in(BNC_75) [BlackBurst]
    PGM_Out[1..4]: out(BNC_75) [SDI]
  }

  instance Cam_1 is PTZ_Camera {
    location: "FOH Truss — Center"
    ip: "10.10.4.11"
  }
  instance Cam_2 is PTZ_Camera {
    location: "Stage Left Wing"
    ip: "10.10.4.12"
  }
  instance Cam_3 is PTZ_Camera {
    location: "Stage Right Wing"
    ip: "10.10.4.13"
  }
  instance Switcher is ATEM_Constellation {
    location: "Video Control Room"
    ip: "10.10.4.100"
  }
  instance LED is Barco_E2 {
    location: "Video Control Room"
    ip: "10.10.4.101"
  }
  instance Media is Disguise_GX2C {
    location: "Video Control Room"
    ip: "10.10.4.110"
  }

  # Genlock distribution to cameras
  connect Genlock_In[1] -> Cam_1.Genlock_In
  connect Genlock_In[2] -> Cam_2.Genlock_In
  connect Genlock_In[3] -> Cam_3.Genlock_In

  # BlackBurst to switcher (uses SDI_In[20] as sync input)
  connect BB_In -> Switcher.SDI_In[20]

  # Camera feeds to switcher
  connect Cam_1.SDI_Out -> Switcher.SDI_In[1] {
    cable: "SDI_IMAG1"
    length: "50m"
  }
  connect Cam_2.SDI_Out -> Switcher.SDI_In[2] {
    cable: "SDI_IMAG2"
    length: "30m"
  }
  connect Cam_3.SDI_Out -> Switcher.SDI_In[3] {
    cable: "SDI_IMAG3"
    length: "35m"
  }

  # Media server feeds to switcher
  connect Media.SDI_Out[1] -> Switcher.SDI_In[5] {
    cable: "SDI_MS_Lyrics"
    length: "3m"
  }
  connect Media.SDI_Out[2] -> Switcher.SDI_In[6] {
    cable: "SDI_MS_BG"
    length: "3m"
  }

  # Switcher to LED processor
  connect Switcher.SDI_Out[1] -> LED.SDI_In[1] {
    cable: "SDI_PGM_LED"
    length: "2m"
  }
  connect Switcher.SDI_Out[2] -> LED.SDI_In[2] {
    cable: "SDI_IMAG_LED"
    length: "2m"
  }
  connect Switcher.SDI_Aux[1] -> LED.SDI_In[3] {
    cable: "SDI_AUX_LED"
    length: "2m"
  }

  # Program outputs
  connect Switcher.SDI_Out[3..6] -> PGM_Out[1..4]
}

# --- Comms Subsystem ----------------------------------------
# Intercom matrix and all keypanels

template CommsSystem {
  ports {
    AES_In[1..4]: in(BNC_75) [AES3]
    AES_Out[1..4]: out(BNC_75) [AES3]
  }

  instance Frame is RTS_ADAM {
    location: "Production Office"
    ip: "10.10.5.1"
  }
  instance KP_SM is RTS_KP {
    location: "Stage Manager Desk"
  }
  instance KP_LX is RTS_KP {
    location: "Lighting Control"
  }
  instance KP_FOH is RTS_KP {
    location: "FOH Mix Position"
  }
  instance KP_VID is RTS_KP {
    location: "Video Control Room"
  }

  # Trunk connections to keypanels
  connect Frame.Trunk[1] -> KP_SM.Trunk {
    cable: "Cat6a_KP_SM"
    length: "15m"
  }
  connect Frame.Trunk[2] -> KP_LX.Trunk {
    cable: "Cat6a_KP_LX"
    length: "20m"
  }
  connect Frame.Trunk[3] -> KP_FOH.Trunk {
    cable: "Cat6a_KP_FOH"
    length: "60m"
  }
  connect Frame.Trunk[4] -> KP_VID.Trunk {
    cable: "Cat6a_KP_VID"
    length: "10m"
  }

  # AES passthrough for talkback integration
  connect AES_In[1..4] -> Frame.AES_In[1..4]
  connect Frame.AES_Out[1..4] -> AES_Out[1..4]
}


# ==== ROOT-LEVEL INSTANCES ==================================

instance Audio_FOH is AudioFOH
instance Audio_Stage is AudioStage
instance Audio_Net is AudioNetwork
instance Video is VideoSystem
instance Comms is CommsSystem

instance House_Sync is Sync_Generator {
  location: "Master Control Closet"
  ip: "10.10.6.1"
}


# ==== ROOT-LEVEL CONNECTIONS (cross-subsystem) ===============

# Optocore ring: FOH → Stage → FOH (full ring)
connect Audio_FOH.Optocore_Out -> Audio_Stage.Optocore_In {
  cable: "SM_Fiber_FOH_A"
  length: "60m"
}
connect Audio_Stage.Optocore_Out -> Audio_FOH.Optocore_In {
  cable: "SM_Fiber_Return"
  length: "65m"
}

# Wireless Dante feeds into network switches
connect Audio_Stage.Dante_Out[1] -> Audio_Net.Dante_In[1] {
  cable: "Cat6a_WR1_Pri"
  length: "8m"
}
connect Audio_Stage.Dante_Out[2] -> Audio_Net.Dante_In[2] {
  cable: "Cat6a_WR1_Sec"
  length: "8m"
}
connect Audio_Stage.Dante_Out[3] -> Audio_Net.Dante_In[3] {
  cable: "Cat6a_WR2_Pri"
  length: "12m"
}
connect Audio_Stage.Dante_Out[4] -> Audio_Net.Dante_In[4] {
  cable: "Cat6a_WR2_Sec"
  length: "12m"
}

# Program feed to intercom (operators hear the show mix)
connect Audio_FOH.AES_Out[1] -> Comms.AES_In[1] {
  cable: "AES_PGM_Comms"
  length: "15m"
}
connect Audio_FOH.AES_Out[2] -> Comms.AES_In[2] {
  cable: "AES_PGM_Comms_2"
  length: "15m"
}

# Sync distribution
connect House_Sync.WordClock_Out[1] -> Audio_FOH.WordClock_In {
  cable: "WC_FOH"
  length: "5m"
}
connect House_Sync.TriLevel_Out[1] -> Video.Genlock_In[1] {
  cable: "TL_Cam1"
  length: "55m"
}
connect House_Sync.TriLevel_Out[2] -> Video.Genlock_In[2] {
  cable: "TL_Cam2"
  length: "35m"
}
connect House_Sync.TriLevel_Out[3] -> Video.Genlock_In[3] {
  cable: "TL_Cam3"
  length: "40m"
}
connect House_Sync.BB_Out[1] -> Video.BB_In {
  cable: "BB_Switcher"
  length: "5m"
}


# ==== SIGNALS ================================================

signal Lead_Vocal {
  description: "Lead vocalist — wireless handheld via Axient"
}

signal Guitar_DI {
  description: "Electric guitar direct box"
}

signal Bass_DI {
  description: "Bass guitar direct box"
}

signal IMAG_Program {
  description: "IMAG program feed to LED walls"
}

signal Lyrics_Feed {
  description: "Lyrics/lower-thirds from media server"
}


# ==== FLAGS ==================================================

flag All_Sync_Locked {
  description: "House sync distributed to all video sources"
  severity: "info"
}

flag Optocore_Ring_OK {
  description: "Optocore fiber ring closed — redundant path active"
  severity: "info"
}
`,
  },
  {
    kind: 'multi',
    name: 'Concert Venue — Multi-file',
    description: 'Same venue split across 5 files with use imports',
    entry: '04-venue-instances.patch',
    files: {
      '01-audio-devices.patch': `# ============================================================
# Concert Venue — Audio Device Templates
# Reusable templates for mixing consoles, stageboxes,
# amplifiers, wireless receivers, and audio networking.
# ============================================================

template DiGiCo_SD7 {
  meta {
    manufacturer: "DiGiCo"
    model: "SD7"
    category: "Console"
  }
  ports {
    Optocore_A_In: in(OpticalCon) [Optocore, primary]
    Optocore_A_Out: out(OpticalCon) [Optocore, primary]
    Optocore_B_In: in(OpticalCon) [Optocore, secondary]
    Optocore_B_Out: out(OpticalCon) [Optocore, secondary]
    Local_In[1..12]: in(XLR)
    Local_Out[1..12]: out(XLR)
    AES_Out[1..8]: out(BNC_75) [AES3]
    MADI_A_In[1..64]: in(BNC_75) [MADI, ch64]
    MADI_A_Out[1..64]: out(BNC_75) [MADI, ch64]
    MADI_B_In[1..64]: in(BNC_75) [MADI, ch64]
    MADI_B_Out[1..64]: out(BNC_75) [MADI, ch64]
  }
}

template SD_Rack {
  meta {
    manufacturer: "DiGiCo"
    model: "SD-Rack"
    category: "Stagebox"
  }
  ports {
    Optocore_A_In: in(OpticalCon) [Optocore, primary]
    Optocore_A_Out: out(OpticalCon) [Optocore, primary]
    Optocore_B_In: in(OpticalCon) [Optocore, secondary]
    Optocore_B_Out: out(OpticalCon) [Optocore, secondary]
    Mic_In[1..56]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Optocore_A_Out
}

template Lab_gruppen_PLM {
  meta {
    manufacturer: "Lab.gruppen"
    model: "PLM_20K44"
    category: "Amplifier"
  }
  ports {
    AES_In[1..4]: in(XLR) [AES3]
    Speaker_Out[1..4]: out(SpeakON)
  }
  bridge AES_In -> Speaker_Out
}

template Shure_Axient {
  meta {
    manufacturer: "Shure"
    model: "Axient_Digital_AD4Q"
    category: "Wireless"
  }
  ports {
    Dante_Pri_In[1..4]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..4]: out(etherCON) [Dante, primary]
    Dante_Sec_In[1..4]: in(etherCON) [Dante, secondary]
    Dante_Sec_Out[1..4]: out(etherCON) [Dante, secondary]
    Analog_Out[1..4]: out(XLR)
    Antenna_In[1..2]: in(BNC_50) [RF]
  }
  bridge Antenna_In -> Analog_Out
}

template OptoSplitter {
  meta {
    manufacturer: "DiGiCo"
    model: "Optocore_DD32R"
    category: "Network"
  }
  ports {
    Loop_In: in(OpticalCon) [Optocore]
    Loop_Out: out(OpticalCon) [Optocore]
    MADI_Out: out(BNC_75) [MADI, ch64]
  }
}
`,
      '02-video-devices.patch': `# ============================================================
# Concert Venue — Video Device Templates
# LED wall processors, IMAG cameras, video switcher,
# and media server for concert production.
# ============================================================

template Barco_E2 {
  meta {
    manufacturer: "Barco"
    model: "E2"
    category: "Video Processor"
  }
  ports {
    SDI_In[1..8]: in(BNC_75) [SDI, G3]
    HDMI_In[1..4]: in(HDMI)
    DVI_Out[1..8]: out(DVI) [LED_Wall]
    SDI_Out[1..4]: out(BNC_75) [SDI, G3]
  }
}

template PTZ_Camera {
  meta {
    manufacturer: "Panasonic"
    model: "AW_UE150"
    category: "Camera"
  }
  ports {
    SDI_Out: out(BNC_75) [SDI, UHD]
    HDMI_Out: out(HDMI) [UHD]
    NDI_Out: out(etherCON) [NDI]
    Genlock_In: in(BNC_75) [TriLevel]
  }
}

template ATEM_Constellation {
  meta {
    manufacturer: "Blackmagic"
    model: "ATEM_Constellation_8K"
    category: "Switcher"
  }
  ports {
    SDI_In[1..20]: in(BNC_75) [SDI, G3]
    SDI_Out[1..12]: out(BNC_75) [SDI, G3]
    SDI_Aux[1..6]: out(BNC_75) [SDI, G3]
  }
  bridge SDI_In -> SDI_Out
}

template Disguise_GX2C {
  meta {
    manufacturer: "Disguise"
    model: "gx_2c"
    category: "Media Server"
  }
  ports {
    SDI_In[1..2]: in(BNC_75) [SDI, G3]
    SDI_Out[1..4]: out(BNC_75) [SDI, UHD]
    DisplayPort_Out[1..4]: out(DP) [UHD]
    Dante_Out: out(etherCON) [Dante]
    NDI_Out: out(etherCON) [NDI]
  }
}
`,
      '03-comms-network.patch': `# ============================================================
# Concert Venue — Comms & Network Templates
# Intercom system, Dante network switches, and sync
# generator for tying the whole venue together.
# ============================================================

template RTS_ADAM {
  meta {
    manufacturer: "RTS"
    model: "ADAM_M"
    category: "Intercom"
  }
  ports {
    Trunk_In[1..8]: in(etherCON) [OMNEO]
    Trunk_Out[1..8]: out(etherCON) [OMNEO]
    AES_In[1..4]: in(XLR) [AES3]
    AES_Out[1..4]: out(XLR) [AES3]
    GPIO[1..8]: io(DB25)
  }
}

template RTS_KP {
  meta {
    manufacturer: "RTS"
    model: "KP_5032"
    category: "Intercom Panel"
  }
  ports {
    Trunk_In: in(etherCON) [OMNEO]
    Trunk_Out: out(etherCON) [OMNEO]
    Headset: io(XLR5)
  }
}

template Luminex_GigaCore {
  meta {
    manufacturer: "Luminex"
    model: "GigaCore_26i"
    category: "Network Switch"
  }
  ports {
    Port[1..24]: io(etherCON) [Ethernet, Gigabit]
    SFP[1..2]: io(SFP) [Ethernet, G10]
  }
}

template Sync_Generator {
  meta {
    manufacturer: "Evertz"
    model: "5601MSC"
    category: "Sync"
  }
  ports {
    BB_Out[1..8]: out(BNC_75) [BlackBurst]
    TriLevel_Out[1..8]: out(BNC_75) [TriLevel]
    LTC_Out[1..4]: out(XLR) [Timecode]
    GPS_In: in(BNC_75) [GPS]
    WordClock_Out[1..4]: out(BNC_75) [WordClock]
  }
}
`,
      '04-venue-instances.patch': `# ============================================================
# Concert Venue — Device Instances
# A 5,000-seat concert hall with main PA, monitor world,
# IMAG video, LED walls, and full comms.
# ============================================================

# --- Audio: FOH (Front of House) ----------------------------

instance FOH_Console is DiGiCo_SD7 {
  location: "FOH Mix Position"
  ip: "10.10.1.10"
}

instance FOH_Rack is SD_Rack {
  location: "FOH Rack Room"
  ip: "10.10.1.11"
}

# --- Audio: Monitor World -----------------------------------

instance MON_Console is DiGiCo_SD7 {
  location: "Stage Left — Monitor World"
  ip: "10.10.1.20"
}

instance Stage_Rack_A is SD_Rack {
  location: "Stage Left Pit"
  ip: "10.10.1.21"
}

instance Stage_Rack_B is SD_Rack {
  location: "Stage Right Pit"
  ip: "10.10.1.22"
}

# --- Audio: Wireless Receivers ------------------------------

instance Wireless_Rack_1 is Shure_Axient {
  location: "Stage Left — RF Rack"
  ip: "10.10.2.10"
}

instance Wireless_Rack_2 is Shure_Axient {
  location: "Stage Right — RF Rack"
  ip: "10.10.2.11"
}

# --- Audio: Amplifiers --------------------------------------

instance Amp_Main_L is Lab_gruppen_PLM {
  location: "Main PA Left Amp Room"
}

instance Amp_Main_R is Lab_gruppen_PLM {
  location: "Main PA Right Amp Room"
}

instance Amp_Subs is Lab_gruppen_PLM {
  location: "Sub Array Amp Room"
}

instance Amp_Front_Fill is Lab_gruppen_PLM {
  location: "Front Fill Amp Room"
}

# --- Audio: Network -----------------------------------------

instance Audio_SW_FOH is Luminex_GigaCore {
  location: "FOH Rack Room"
  ip: "10.10.3.1"
}

instance Audio_SW_Stage is Luminex_GigaCore {
  location: "Stage Left — Network Rack"
  ip: "10.10.3.2"
}

instance Opto_Split is OptoSplitter {
  location: "Stage Left — Optocore Ring"
  ip: "10.10.3.10"
}

# --- Video: IMAG Cameras ------------------------------------

instance IMAG_Cam_1 is PTZ_Camera {
  location: "FOH Truss — Center"
  ip: "10.10.4.11"
}

instance IMAG_Cam_2 is PTZ_Camera {
  location: "Stage Left Wing"
  ip: "10.10.4.12"
}

instance IMAG_Cam_3 is PTZ_Camera {
  location: "Stage Right Wing"
  ip: "10.10.4.13"
}

# --- Video: Switching & Processing --------------------------

instance Video_Switcher is ATEM_Constellation {
  location: "Video Control Room"
  ip: "10.10.4.100"
}

instance LED_Processor is Barco_E2 {
  location: "Video Control Room"
  ip: "10.10.4.101"
}

instance Media_Server is Disguise_GX2C {
  location: "Video Control Room"
  ip: "10.10.4.110"
}

# --- Comms --------------------------------------------------

instance Intercom_Frame is RTS_ADAM {
  location: "Production Office"
  ip: "10.10.5.1"
}

instance KP_Stage_Manager is RTS_KP {
  location: "Stage Manager Desk"
}

instance KP_Lighting is RTS_KP {
  location: "Lighting Control"
}

instance KP_FOH_Audio is RTS_KP {
  location: "FOH Mix Position"
}

instance KP_Video is RTS_KP {
  location: "Video Control Room"
}

# --- Sync ---------------------------------------------------

instance House_Sync is Sync_Generator {
  location: "Master Control Closet"
  ip: "10.10.6.1"
}
`,
      '05-connections.patch': `# ============================================================
# Concert Venue — Connections & Signals
# Physical cabling, logical bridges, and named signals
# tying the entire venue together.
# ============================================================

# --- Optocore Ring (Audio Fiber) -----------------------------

connect FOH_Console.Optocore_A -> Opto_Split.Loop_In {
  cable: "SM_Fiber_FOH_A"
  length: "60m"
}

connect Opto_Split.Loop_Out -> MON_Console.Optocore_A {
  cable: "SM_Fiber_MON_A"
  length: "5m"
}

connect MON_Console.Optocore_B -> Stage_Rack_A.Optocore_A {
  cable: "SM_Fiber_SRA"
  length: "10m"
}

connect Stage_Rack_A.Optocore_B -> Stage_Rack_B.Optocore_A {
  cable: "SM_Fiber_Cross"
  length: "25m"
}

connect Stage_Rack_B.Optocore_B -> FOH_Console.Optocore_B {
  cable: "SM_Fiber_Return"
  length: "65m"
}

# --- MADI: FOH to Recording ---------------------------------

connect FOH_Console.MADI_A_Out -> FOH_Rack.Optocore_A {
  cable: "MADI_FOH_Rec"
  length: "3m"
}

# --- Audio Network Switches ---------------------------------

connect Audio_SW_FOH.SFP[1] -> Audio_SW_Stage.SFP[1] {
  cable: "SM_Fiber_NetTrunk"
  length: "60m"
}

connect Wireless_Rack_1.Dante_Pri_Out -> Audio_SW_Stage.Port[1] {
  cable: "Cat6a_WR1_Pri"
  length: "8m"
}
connect Audio_SW_Stage.Port[1] -> Wireless_Rack_1.Dante_Pri_In

connect Wireless_Rack_1.Dante_Sec_Out -> Audio_SW_Stage.Port[2] {
  cable: "Cat6a_WR1_Sec"
  length: "8m"
}
connect Audio_SW_Stage.Port[2] -> Wireless_Rack_1.Dante_Sec_In

connect Wireless_Rack_2.Dante_Pri_Out -> Audio_SW_Stage.Port[3] {
  cable: "Cat6a_WR2_Pri"
  length: "12m"
}
connect Audio_SW_Stage.Port[3] -> Wireless_Rack_2.Dante_Pri_In

connect Wireless_Rack_2.Dante_Sec_Out -> Audio_SW_Stage.Port[4] {
  cable: "Cat6a_WR2_Sec"
  length: "12m"
}
connect Audio_SW_Stage.Port[4] -> Wireless_Rack_2.Dante_Sec_In

# --- Amplifier Feeds (AES from FOH Console) ------------------

connect FOH_Console.AES_Out[1] -> Amp_Main_L.AES_In[1] {
  cable: "AES_Main_L_Hi"
  length: "40m"
}

connect FOH_Console.AES_Out[2] -> Amp_Main_L.AES_In[2] {
  cable: "AES_Main_L_Lo"
  length: "40m"
}

connect FOH_Console.AES_Out[3] -> Amp_Main_R.AES_In[1] {
  cable: "AES_Main_R_Hi"
  length: "45m"
}

connect FOH_Console.AES_Out[4] -> Amp_Main_R.AES_In[2] {
  cable: "AES_Main_R_Lo"
  length: "45m"
}

connect FOH_Console.AES_Out[5] -> Amp_Subs.AES_In[1] {
  cable: "AES_Subs_L"
  length: "35m"
}

connect FOH_Console.AES_Out[6] -> Amp_Subs.AES_In[2] {
  cable: "AES_Subs_R"
  length: "35m"
}

connect FOH_Console.AES_Out[7] -> Amp_Front_Fill.AES_In[1] {
  cable: "AES_FF_L"
  length: "30m"
}

connect FOH_Console.AES_Out[8] -> Amp_Front_Fill.AES_In[2] {
  cable: "AES_FF_R"
  length: "30m"
}

# --- Video: Cameras to Switcher -----------------------------

connect IMAG_Cam_1.SDI_Out -> Video_Switcher.SDI_In[1] {
  cable: "SDI_IMAG1"
  length: "50m"
}

connect IMAG_Cam_2.SDI_Out -> Video_Switcher.SDI_In[2] {
  cable: "SDI_IMAG2"
  length: "30m"
}

connect IMAG_Cam_3.SDI_Out -> Video_Switcher.SDI_In[3] {
  cable: "SDI_IMAG3"
  length: "35m"
}

# --- Video: Media Server to Switcher ------------------------

connect Media_Server.SDI_Out[1] -> Video_Switcher.SDI_In[5] {
  cable: "SDI_MS_Lyrics"
  length: "3m"
}

connect Media_Server.SDI_Out[2] -> Video_Switcher.SDI_In[6] {
  cable: "SDI_MS_BG"
  length: "3m"
}

# --- Video: Switcher to LED Processor -----------------------

connect Video_Switcher.SDI_Out[1] -> LED_Processor.SDI_In[1] {
  cable: "SDI_PGM_LED"
  length: "2m"
}

connect Video_Switcher.SDI_Out[2] -> LED_Processor.SDI_In[2] {
  cable: "SDI_IMAG_LED"
  length: "2m"
}

connect Video_Switcher.SDI_Aux[1] -> LED_Processor.SDI_In[3] {
  cable: "SDI_AUX_LED"
  length: "2m"
}

# --- Comms: Intercom Panels ---------------------------------

connect Intercom_Frame.Trunk[1] -> KP_Stage_Manager.Trunk {
  cable: "Cat6a_KP_SM"
  length: "15m"
}

connect Intercom_Frame.Trunk[2] -> KP_Lighting.Trunk {
  cable: "Cat6a_KP_LX"
  length: "20m"
}

connect Intercom_Frame.Trunk[3] -> KP_FOH_Audio.Trunk {
  cable: "Cat6a_KP_FOH"
  length: "60m"
}

connect Intercom_Frame.Trunk[4] -> KP_Video.Trunk {
  cable: "Cat6a_KP_VID"
  length: "10m"
}

# --- Sync Distribution --------------------------------------

connect House_Sync.BB_Out[1] -> Video_Switcher.SDI_In[20] {
  cable: "BB_Switcher"
  length: "5m"
}

connect House_Sync.TriLevel_Out[1] -> IMAG_Cam_1.Genlock_In {
  cable: "TL_Cam1"
  length: "55m"
}

connect House_Sync.TriLevel_Out[2] -> IMAG_Cam_2.Genlock_In {
  cable: "TL_Cam2"
  length: "35m"
}

connect House_Sync.TriLevel_Out[3] -> IMAG_Cam_3.Genlock_In {
  cable: "TL_Cam3"
  length: "40m"
}

connect House_Sync.WordClock_Out[1] -> FOH_Console.Local_In[12] {
  cable: "WC_FOH"
  length: "5m"
}

# --- Audio Bridges (logical signal mapping) ------------------

bridge Stage_Rack_A.Mic_In[1..56] -> FOH_Console.Local_In[1..12]
bridge Stage_Rack_B.Mic_In[1..32] -> MON_Console.Local_In[1..12]

# --- Named Signals ------------------------------------------

signal Lead_Vocal {
  origin: Stage_Rack_A.Mic_In[1]
  channel: "1"
  description: "Lead vocalist — wireless handheld via Axient"
}

signal Guitar_DI {
  origin: Stage_Rack_A.Mic_In[5]
  channel: "5"
  description: "Electric guitar direct box"
}

signal Bass_DI {
  origin: Stage_Rack_A.Mic_In[6]
  channel: "6"
  description: "Bass guitar direct box"
}

signal Kick_In {
  origin: Stage_Rack_A.Mic_In[10]
  channel: "10"
  description: "Kick drum — inside mic"
}

signal Kick_Out {
  origin: Stage_Rack_A.Mic_In[11]
  channel: "11"
  description: "Kick drum — outside mic"
}

signal Snare_Top {
  origin: Stage_Rack_A.Mic_In[12]
  channel: "12"
  description: "Snare drum — top mic"
}

signal IMAG_Program {
  description: "IMAG program feed to LED walls"
}

signal Lyrics_Feed {
  description: "Lyrics/lower-thirds from media server"
}

# --- Flags ---------------------------------------------------

flag All_Sync_Locked {
  description: "House sync distributed to all video sources"
  severity: "info"
}

flag Optocore_Ring_OK {
  description: "Optocore fiber ring closed — redundant path active"
  severity: "info"
}
`,
    },
  },
]
