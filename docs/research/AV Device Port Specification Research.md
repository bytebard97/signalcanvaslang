# **Technical Analysis of Input/Output Architectures in Professional Audiovisual Infrastructure**

The transition from point-to-point analog signaling to complex, software-defined networked ecosystems has fundamentally redefined the physical back-panel requirements of professional audiovisual (AV) equipment. In the contemporary landscape, the "port" is no longer merely a termination point for a copper conductor; it is a gateway into a sophisticated fabric of audio-over-IP (AoIP), high-definition video transport, and distributed control data. This report provides a detailed examination of the physical I/O specifications for thirty cornerstone devices in the professional AV industry, serving as a comprehensive technical reference for the development of domain-specific language (DSL) templates.

## **Yamaha Digital Mixing and I/O Architectures**

Yamaha’s professional audio solutions, specifically the CL and QL series, along with the RIVAGE flagship systems, have established a dominant paradigm in Dante-native networking. The physical back panels of these devices represent a hybrid approach, balancing the necessity for local analog "Omni" connections with massive scalability via digital expansion slots and networked ports.

### **Yamaha CL5 and QL5 Digital Mixing Consoles**

The Yamaha CL5 and QL5 consoles are architecturally designed around the Dante protocol, yet they maintain a specific set of local physical connectors to facilitate perimeter I/O without requiring a separate stage box for every signal. The CL5, as a larger control surface, prioritizes expansion and control redundancy, whereas the QL5 serves as a more self-contained solution with a higher density of local analog inputs.

| Port Category | Connector Type | Protocol/Standard | Channel Count | Format Details |
| :---- | :---- | :---- | :---- | :---- |
| Analog Input (CL5) | XLR-3-31 (Female) | Balanced Mic/Line | 8 | 7.5k$\\Omega$ Load Impedance 1 |
| Analog Input (QL5) | XLR-3-31 (Female) | Balanced Mic/Line | 32 | 7.5k$\\Omega$ Load Impedance 1 |
| Analog Output (CL5) | XLR-3-32 (Male) | Balanced Line | 8 | 75$\\Omega$ Source Impedance 2 |
| Analog Output (QL5) | XLR-3-32 (Male) | Balanced Line | 16 | 75$\\Omega$ Source Impedance 1 |
| Dante Network | etherCON (RJ45) | Dante (1000Base-T) | 64 In / 64 Out | Primary and Secondary Redundancy 1 |
| Digital Output | XLR-3-32 (Male) | AES/EBU (AES3) | 2 (1 Pair) | Professional Standard 1 |
| Expansion Slots | Mini-YGDAI (MY) | Proprietary Digital | 3 (CL5) / 2 (QL5) | Supports various I/O cards 1 |
| Word Clock | BNC | TTL/75$\\Omega$ | 1 In / 1 Out | Synchronization 1 |
| GPI | D-Sub 15-pin (Female) | Contact Closure | 5 In / 5 Out | Control Logic 1 |
| MIDI | 5-pin DIN | MIDI | 1 In / 1 Out | External Control 1 |
| Network | RJ-45 | IEEE 802.3 | 1 | Computer/Control Connection 1 |
| Lamp Power | XLR-4-31 (Female) | \+12V DC | 3 (CL5) / 2 (QL5) | Console Lighting 1 |

The analog preamplifiers on these surfaces are designed to handle signals ranging from ![][image1] to ![][image2] nominal, with a maximum input level of ![][image3] before clipping occurs.1 For power redundancy, the CL5 features a dedicated port for the PW800W external power supply unit, a feature notably absent on the QL5, which relies solely on its internal PSU.1

### **Yamaha Rio3224-D2 I/O Rack**

The Rio3224-D2 represents the evolution of Yamaha's stage I/O, providing a robust interface between the analog stage environment and the Dante network. Its design emphasizes high-capacity transport and visual feedback via an integrated character display.3

| Port Type | Connector Type | Protocol | Channel Count | Technical Specification |
| :---- | :---- | :---- | :---- | :---- |
| Analog Input | XLR-3-31 | Mic/Line | 32 | Software-controlled \+48V 4 |
| Analog Output | XLR-3-32 | Line Level | 16 | Max \+24dBu or \+18dBu select 4 |
| Digital Output | XLR-3-32 | AES/EBU | 8 (4 pairs) | RS422 Level, 24-bit 4 |
| Dante Primary | etherCON | 1000Base-T | 32 In / 32 Out | Redundant/Daisy-chain 4 |
| Dante Secondary | etherCON | 1000Base-T | 32 In / 32 Out | Redundant/Daisy-chain 4 |

The Rio3224-D2 supports multiple sampling frequencies, including 44.1 kHz, 48 kHz, 88.2 kHz, and 96 kHz.4 The signal delay is remarkably low, rated at less than ![][image4] when measuring from a Rio input to a Rio output via a RIVAGE PM10 console at a ![][image5] sample rate.4 To ensure reliability, the unit contains dual internal power supply units with separate IEC inlets.3

### **Yamaha RIVAGE PM5 and RPio622**

The RIVAGE PM5 system utilizes the CS-R5 control surface and the RPio622 I/O rack, connected via the ultra-low latency TWINLANe network or Dante.5 The I/O density on the PM5 system is achieved through high-capacity expansion slots.

**Yamaha RIVAGE PM5 (CS-R5 Surface) Local I/O:** The CS-R5 surface provides essential local connectivity, allowing the operator to interface with local playback devices and talkback systems without consuming stage rack ports. It features 8 analog inputs and 8 analog outputs on XLR connectors, notably utilizing SILK processing on the input stage to emulate the harmonic characteristics of Rupert Neve Designs circuitry.6 Digital connectivity includes 4 AES/EBU inputs and 4 AES/EBU outputs.7

Expansion is handled through four HY slots and two MY slots. The HY slots are capable of massive throughput; for instance, the HY256-TL card provides 256 channels of TWINLANe I/O, while the HY144-D provides 144 channels of Dante.9 Control ports include 8 GPI inputs, 8 GPI outputs, and MIDI In/Out. Unlike the larger CS-R10, the CS-R5 does not include a dedicated Video Out or a Talkback In on the rear panel, as these functions are integrated differently into its touch-centric workflow.7

**Yamaha RPio622 I/O Rack:** The RPio622 is a 10U modular stage box that serves as the primary physical interface for the RIVAGE system. It is characterized by its modularity, featuring six RY card slots that can be populated with various analog or digital I/O modules, such as the RY16-ML-SILK.5

| Slot/Port Type | Capacity | Protocol/Format | Notes |
| :---- | :---- | :---- | :---- |
| RY Card Slots | 6 Slots | Analog/Digital | Up to 96ch total (16ch/card) 5 |
| HY Card Slot 1 | 256 In / 256 Out | TWINLANe/Dante | High-capacity backbone 5 |
| HY Card Slot 2 | 128 In / 128 Out | TWINLANe/Dante | Secondary backbone 5 |
| Mini-YGDAI (MY) | 2 Slots | Multi-format | Legacy support 5 |
| Word Clock | 1 In / 1 Out | BNC | TTL/75$\\Omega$ 5 |
| Network | RJ-45 | etherCON CAT5 | 10BASE-T/100Base-TX 5 |
| Fault Output | 3-pin Euroblock | Contact Closure | System Health Monitoring 5 |

The RPio622 includes dual redundant power supplies as a standard feature, with a maximum power consumption of ![][image6].5 Its physical dimensions (![][image7]) and substantial weight (![][image8] unloaded) reflect its industrial-grade construction for touring and large-scale installations.5

## **DiGiCo Digital Mixing Ecosystem**

DiGiCo consoles are built upon the Stealth FPGA (Field Programmable Gate Array) engine, which provides a high degree of flexibility in how I/O is routed and processed. The transition to the Quantum series has further enhanced this with the "Ultimate Stadius" 32-bit conversion technology.

### **DiGiCo Quantum 338**

The Quantum 338 is a 128-channel mixing console that provides a significant amount of local I/O on its rear panel, integrated with MADI, DMI, and optional Optocore interfaces.13

| Port Type | Connector | Protocol | Channels | Characteristics |
| :---- | :---- | :---- | :---- | :---- |
| Local Mic/Line In | XLR-3-31 | Analog | 8 | 32-bit Stadius ADC 14 |
| Local Line Out | XLR-3-32 | Analog | 8 | 32-bit Stadius DAC 14 |
| AES/EBU I/O | XLR-3-31/32 | AES3 | 4 In / 4 Out | 8ch Total, SRC on In 14 |
| MADI BNC | BNC | AES10 | 6 I/O | 6 @ 48k / 3 @ 96k 14 |
| UB MADI | USB Type-B | MADI over USB | 48 I/O | Recording/Playback 14 |
| DMI Slots | DMI Format | Multi-protocol | 2 Slots | 64ch I/O per slot 13 |
| GPI/GPO | D-Sub 37 | Contact/Logic | 16 In / 16 Out | Dedicated control 14 |
| Word Clock | BNC | Clock Sync | 1 In / 1 Out | System Timing 14 |
| Optocore (Opt) | HMA/ST/OpticalCon | Optocore | Optional | Single or Dual Loop 14 |

The Quantum 338 also features four switched Ethernet ports for control network distribution and a DisplayPort output for an external overview monitor.14 The DMI (Digital Multi-Interface) slots are particularly versatile, supporting cards for Dante, MADI, Waves SoundGrid, and Aviom, allowing the console to adapt to diverse environment requirements.14

### **DiGiCo SD12 and SD-Rack**

The SD12 is a mid-sized console in the SD range, utilizing two 15-inch touchscreens and a rear panel optimized for both local and remote connectivity.15

**SD12 Local Connectivity:** The SD12 rear panel provides 8 XLR mic/line inputs and 8 XLR line outputs, alongside 4 XLR AES/EBU inputs and outputs.15 For MADI connectivity, it offers two sets of BNC I/O, supporting up to 112 channels at ![][image9] or 56 channels at ![][image5]. Like the Quantum series, it includes a UB MADI port for 48 channels of USB audio and two DMI slots for protocol expansion.15 Control interfaces include a D-Sub 37 for 16 GPI and 16 GPO, MIDI In/Out/Thru, and an RS422 port. Video monitoring is facilitated by a DVI port.15

**DiGiCo SD-Rack:** The SD-Rack is a modular 14-slot I/O frame that can be populated with a variety of analog and digital modules.16 The chassis is split, with the first 7 slots typically used for input cards and the last 7 for output cards, providing a maximum of 56 inputs and 56 outputs.16

Connectivity between the SD-Rack and the console is achieved via MADI or optional Optocore. The rack features a "MADI POD" containing the primary digital ports. This includes a MADI Main I/O BNC pair and a MADI Aux I/O BNC pair. At ![][image5], both ports are used to transport all 56 channels of I/O, whereas at ![][image9], they can provide redundant cabling.16 Additionally, two MADI Split outputs are provided, allowing the rack to send a direct copy of its inputs to a secondary device, such as a broadcast truck or recording rig, with optional sample rate conversion.16

## **Allen & Heath Digital Ecosystem**

Allen & Heath has standardized its professional line on the 96kHz XCVI FPGA engine, ensuring low latency (![][image10]) and high bit-depth processing across the dLive, Avantis, and SQ series.

### **Allen & Heath dLive S7000 and DM64 MixRack**

The dLive system is distributed, consisting of a control surface and a "MixRack" containing the actual DSP engine.19

**S7000 Control Surface:** The S7000 is the largest surface in the range, featuring 36 faders and dual 12-inch touchscreens. Its local I/O includes 8 balanced XLR inputs with digitally controlled preamps (+5 to \+60dB gain) and 8 balanced XLR outputs.20 Digital connectivity is provided by 2 AES3 stereo XLR inputs and 3 AES3 stereo XLR outputs, all featuring sample rate conversion.20 For surface-to-rack connectivity, it uses dual redundant GigaACE links on etherCON ports. Two additional I/O ports are available for 128x128 channel expansion cards.20

**DM64 MixRack:** The DM64 is a 128-input, 64-bus engine with a dedicated connection hub.19

| Port Function | Connector Type | Protocol | Count | Detail |
| :---- | :---- | :---- | :---- | :---- |
| Mic/Line Input | XLR | Balanced/48V | 64 | digitally controlled preamps 19 |
| Line Output | XLR | Balanced | 32 | soft-patchable 19 |
| GigaACE Link | etherCON | 96kHz GigaACE | 2 | Redundant Surface Link 19 |
| DX Link | etherCON | 96kHz DX | 2 | Redundant I/O Expansion 19 |
| I/O Ports | Option Slot | Multi-format | 3 | 128ch @ 96kHz each 19 |
| ME-1 Port | etherCON | 48kHz ME | 1 | 40ch Personal Monitoring 19 |
| Network | RJ-45 | TCP/IP | 2 | Control and Tunnelling 19 |
| Word Clock | BNC | Clock Sync | 1 In / 1 Out | System Timing 19 |

The DM64 features dual redundant, hot-swappable power supplies and a quiet cooling system designed for near-stage operation.19

### **Allen & Heath Avantis and SQ-7**

The Avantis and SQ-7 consoles represent Allen & Heath's standalone offerings, integrating the XCVI core into a single chassis with significant local I/O.

**Allen & Heath Avantis:** The Avantis provides 64 input channels and 42 configurable buses. Its rear panel features 12 XLR mic/line inputs and 12 XLR line outputs.23 Digital I/O consists of one stereo AES input and two stereo AES outputs. For expansion, the Avantis includes a single SLink port for connection to remote expanders (DX, GX, dSnake) and two I/O ports for 128x128 option cards, including Dante and Waves SoundGrid.23

**Allen & Heath SQ-7:** The SQ-7 is the largest in the SQ line, offering 32 onboard mic preamps (XLR) and 16 XLR line outputs.26 It also includes two 1/4" TRS stereo inputs and a 3.5mm stereo input for auxiliary sources. Local digital connectivity is limited to a single AES output (XLR).26 The SQ-7 features one SLink port and one I/O port for 64x64 expansion cards. A dedicated USB-B port provides a 32x32 channel audio interface for DAW connectivity, while a USB-A port (SQ-Drive) handles stereo and multitrack recording directly to external drives.26

## **Wireless Systems and Networking Infrastructure**

Shure's wireless systems have redefined the back-panel requirements of receivers, integrating them deeply into the Dante and control networks.

### **Shure AD4Q Axient Digital and ULXD4Q**

The AD4Q and ULXD4Q are quad-channel receivers that maximize rack density while providing extensive signal routing options.29

**Shure AD4Q (Axient Digital):** The AD4Q is designed for mission-critical RF environments, featuring "Quadversity" mode which allows for four antennas to be connected via BNC ports to provide superior coverage.31 Its audio I/O consists of four transformer-balanced XLR outputs; notably, outputs 3 and 4 are switchable to AES3 digital format.30 Additionally, four transformer-balanced 1/4" outputs are provided. Networking is handled by four Ethernet ports: two dedicated to Dante (Primary/Secondary) and two for network control with Power over Ethernet (PoE) support.30 The unit supports AC power cascading via locking connectors and offers an optional DC module for redundant power.31

**Shure ULXD4Q:** The ULXD4Q provides four channels of digital wireless reception in a similar 1RU form factor. Each channel has a dedicated XLR output.29 Networking is facilitated by two Ethernet ports which support Dante digital audio over Ethernet and allow for integration with Shure's Wireless Workbench software for frequency coordination.29 RF signal sharing is simplified through RF cascade ports, which allow the antenna signal to be passed to an additional receiver unit without an external splitter.29

### **QSC Q-SYS Core 110f**

The Core 110f is a software-based DSP that utilizes a versatile physical I/O configuration to address various installation requirements.

| Connector Type | Label/Color | Function | Channel Count |
| :---- | :---- | :---- | :---- |
| 3-pin Euroblock | Orange | Mic/Line Input (Balanced) | 8 |
| 3-pin Euroblock | Green | Line Level Output (Balanced) | 8 |
| 3-pin Euroblock | Blue | Flex Channel (Configurable I/O) | 8 |
| RJ-45 | LAN A/B | Q-LAN / Control / VoIP | 2 (Gigabit) |
| USB Type-B | USB Device | Digital Audio / Video Bridge | 16x16 Audio |
| RJ-11 | POTS | Analog Telephone Line | 1 |
| DE-9 (Serial) | RS-232 | Control Integration | 1 |

The 8 Flex Channels are a unique innovation, allowing each port to be independently configured as a mic/line input or a line output during system commissioning.33 The USB device port enables the Core 110f to appear to a host computer as multiple virtual USB audio devices.34

## **Modular Converters and Connectivity Tools**

Converting between disparate digital protocols and bridging analog gear to networks is the role of specialized I/O interfaces like those from Focusrite and Audinate.

### **Focusrite RedNet A16R MkII and D16R**

The RedNet range utilizes Dante as its core transport protocol, emphasizing audio quality and hardware redundancy.35

**RedNet A16R MkII:** This 1U interface provides 16 channels of bidirectional analog conversion. Physical I/O is concentrated on four DB25 connectors, each carrying 8 channels of balanced audio wired to the AES59 standard.35 In addition to the analog I/O, it features a pair of AES3 I/O on XLR connectors, with the input capable of being used as a DARS clock source.35 Network and power redundancy are provided through dual etherCON ports and dual IEC power inlets.35

**RedNet D16R:** The D16R is a 16-channel AES3 interface. Its primary I/O is also provided on two DB25 connectors (AES59 standard). A pair of XLR connectors duplicates channels 1 and 2 of the digital I/O for quick patching.36 It uniquely includes RCA connectors for S/PDIF I/O, which is ideal for integrating consumer playback devices into a professional network.36 Like the A16R, it features dual etherCON ports for redundant Dante networking and dual PSUs.36

### **Audinate Dante AVIO AES3 Adapter**

The AVIO AES3 is a compact 2x2 interface designed for portable and cost-effective Dante integration. Its back panel (if it can be called such for a cable-style adapter) consists of one RJ45 port for the Dante network connection (PoE powered), one XLR-M for AES3 output, and one XLR-F for AES3 input.40 It supports sample rates up to ![][image5] and features asynchronous sample rate conversion to allow non-synced digital gear to connect to the Dante network.40

## **Professional Video and Routing Systems**

In the video domain, port density and protocol conversion are critical, especially as workflows transition from SDI to IP-based transport.

### **Blackmagic ATEM 4 M/E Constellation**

The ATEM Constellation series provides massive SDI I/O density within a compact rack-mount chassis.

| Model | SDI Inputs | SDI Outputs | Audio I/O | Format Support |
| :---- | :---- | :---- | :---- | :---- |
| ATEM 4 M/E HD | 40 x 3G-SDI | 24 x 3G-SDI | MADI BNC In/Out | 1080p60 43 |
| ATEM 4 M/E 4K | 40 x 12G-SDI | 28 x 12G-SDI | MADI BNC In/Out | 2160p60 44 |
| 4 M/E 4K Plus | 80 x 12G-SDI | 52 x 12G-SDI | MADI BNC (64ch) | 2160p60 45 |

The 4K Plus model is a significant outlier, offering 80 standards-converted 12G-SDI inputs and 48 customizable 12G-SDI outputs.45 All 4 M/E models include balanced audio on 1/4" jacks, a 5-pin XLR for talkback headsets, and redundant international power supplies with separate AC inlets.46

### **Ross Ultrix**

Ross Ultrix is a hyper-converged platform that integrates routing, multiviewers, and signal processing. Its port specifications depend heavily on the frame size and the cards installed.48

**ULTRIX-HDX-IO Card:** This card is a primary SDI interface for the Ultrix frame. It features 16 HDBNC connectors for inputs and 16 HDBNC connectors for outputs, supporting SDI rates from ![][image11] to ![][image12].48 Additionally, it includes four SFP (Small Form-factor Pluggable) auxiliary ports. AUX A and B are used for SDI or MADI, while AUX C and D can be licensed for Dante I/O or multiviewer outputs.48

**ULTRIX-IPX-IO Card:** For ST 2110 workflows, this card provides four QSFP28 (100GE) or QSFP (25GE) ports via adapters, facilitating massive IP video transport.48

### **Sony HDC-3500**

The HDC-3500 is a high-end system camera that supports optical fiber transmission as standard. Its rear panel is densely populated with specialized connectors for broadcast environments.49

| Connector | Function | Type |
| :---- | :---- | :---- |
| CCU | Camera Control Unit Link | LEMO 3K.93C (Electro-optical) |
| MIC 1 IN | Local Microphone Input | XLR 3-pin (Female) |
| AUDIO IN CH1/CH2 | Line/Mic Inputs | XLR 3-pin (Female) x 2 |
| INTERCOM 1/2 | Comms Headsets | XLR 5-pin (Female) x 2 |
| SDI 1/2 | Digital Video Out | BNC x 2 |
| SDI MONI | Monitor Video Out | BNC |
| PROMPTER/GENLOCK | Sync/Return | BNC (tri-level or black burst) |
| NETWORK TRUNK | IP Data Link | RJ-45 (1 Gbps) |
| REMOTE | External Control | 8-pin |
| TRACKER / CRANE | Control Data | 12-pin x 2 |

The camera body is constructed from magnesium alloy with carbon fiber panels for durability and lightness (![][image13]).49

## **Synchronization and Timing: Evertz 5601MSC**

The Evertz 5601MSC is a master sync pulse generator and master clock system, providing the timing foundation for broadcast facilities.51

**Sync and Timing I/O:** The unit features six BNC outputs that are independently configurable for black burst, tri-level sync, or BES (Bespeak timing).51 For master clocking, it provides two longitudinal timecode (LTC) outputs on XLR connectors and a 15-pin D-connector.51 Frequency stability is provided by a temperature-controlled oscillator, and the unit can be referenced to an external 5 MHz or 10 MHz master oscillator or a GPS/GLONASS antenna via a dedicated BNC input.51 Networking capabilities include an RJ-45 port for NTP (Network Time Protocol) and PTP (Precision Time Protocol) server support.51

## **Intercom and Communication Nodes**

In modern production, intercom is no longer a separate analog system but is integrated into the IP backbone.

### **Clear-Com Eclipse HX**

The Eclipse HX matrix frames (Delta, Median, Omega) utilize a modular card-based architecture. For instance, the E-IPA card provides high-density IV-Core and AES67 connectivity over Ethernet.54

**Physical Interface Modules:** Clear-Com utilizes a variety of interface modules to bridge to legacy systems. The CCI-22 provides a 2-wire interface, while the FOR-22 provides 4-wire analog connectivity.54 Digital interfacing is handled via AES-3 cards (XLR or RJ-45) and MADI cards for high-channel-count trunking between frames.54

### **Riedel Artist-1024**

The Artist-1024 is an ultra-high-density intercom node, providing 1024 non-blocking ports in just 2RU.56

**Universal Interface Card (UIC):** The Artist-1024 relies on the UIC concept, where software-defined hardware can be configured as a router, Artist fiber link, MADI, or SMPTE 2110-30/31 (AES67/Dante) interface.56 The frame supports redundancy at its core, including hot-swappable subscriber cards and dual redundant power supplies.56 The system is completely non-blocking, allowing any port to communicate with any other port across a distributed fiber ring network.57

## **Synthesis of Connectivity Trends**

The exhaustive detail of these thirty devices reveals a clear architectural trend toward **hyper-convergence**. The "back panel" of a modern AV device is increasingly a mix of high-density legacy connectors (XLR, BNC) and high-bandwidth data ports (SFP, QSFP, etherCON). For the purpose of building DSL templates, it is essential to categorize these ports not just by their physical form factor, but by their logical bandwidth and redundancy schemes. The prevalence of dual-redundant power and network ports across nearly all categories—from mixing consoles like the Allen & Heath dLive to intercom matrices like the Riedel Artist-1024—underscores that in professional AV, reliability is a physical design requirement.

Furthermore, the shift toward **Software-Defined I/O**, as seen in the QSC Flex Channels and the Riedel UIC, suggests that future signal flow diagrams will need to account for dynamic port personalities. A single 3-pin Euroblock or SFP cage can no longer be assumed to have a fixed direction or protocol, necessitating a DSL that can describe "run-time" configuration states. This technical baseline provides the necessary granularity for such a system, ensuring that every physical input and output is represented with the precision required for professional-grade AV automation and design.

#### **Works cited**

1. CL Series \- Specs \- Mixers \- Professional Audio \- Products \- Yamaha ..., accessed April 3, 2026, [https://usa.yamaha.com/products/proaudio/mixers/cl\_series/specs.html](https://usa.yamaha.com/products/proaudio/mixers/cl_series/specs.html)  
2. CL5 Data Sheet \- Rentex, accessed April 3, 2026, [https://www.rentex.com/wp-content/uploads/2020/01/CL5-Data-Sheet.pdf](https://www.rentex.com/wp-content/uploads/2020/01/CL5-Data-Sheet.pdf)  
3. R Series (AD/DA): 2nd-generation \- Overview \- Audio and Network Interfaces and YGDAI Cards \- Professional Audio \- Products \- Yamaha USA, accessed April 3, 2026, [https://usa.yamaha.com/products/proaudio/interfaces/r\_series\_adda\_2/index.html](https://usa.yamaha.com/products/proaudio/interfaces/r_series_adda_2/index.html)  
4. R Series (AD/DA): 2nd-generation \- Specs \- Audio and Network ..., accessed April 3, 2026, [https://usa.yamaha.com/products/proaudio/interfaces/r\_series\_adda\_2/specs.html](https://usa.yamaha.com/products/proaudio/interfaces/r_series_adda_2/specs.html)  
5. RPio622 Data Sheet \- SalesWL, accessed April 3, 2026, [https://saleswl.com/wp-content/uploads/2020/01/Yamaha\_RPio622-IO-Rack-Stage-Box\_Data-Sheet.pdf](https://saleswl.com/wp-content/uploads/2020/01/Yamaha_RPio622-IO-Rack-Stage-Box_Data-Sheet.pdf)  
6. YAMAHA PM5 RIVAGE CONTROL SURFACE (CS-R5) \- Orbital Sound, accessed April 3, 2026, [https://www.orbitalsound.com/easihire/product/pdf/1024854/yamaha-pm5-rivage-control-surface-cs-r5](https://www.orbitalsound.com/easihire/product/pdf/1024854/yamaha-pm5-rivage-control-surface-cs-r5)  
7. RIVAGE PM Console Mixer Specs \- Yamaha USA, accessed April 3, 2026, [https://usa.yamaha.com/products/proaudio/mixers/rivage\_pm/specs.html](https://usa.yamaha.com/products/proaudio/mixers/rivage_pm/specs.html)  
8. Yamaha RIVAGE PM5 \- RGEAR.com, accessed April 3, 2026, [https://www.rgear.com/yamaha-rivage-pm5](https://www.rgear.com/yamaha-rivage-pm5)  
9. Yamaha RIVAGE PM Series CS-R5 \- AOE- Your Audio Visual Specialist, accessed April 3, 2026, [https://www.aoe.com.sg/product/yamaha-rivage-pm-series-csd-r5/](https://www.aoe.com.sg/product/yamaha-rivage-pm-series-csd-r5/)  
10. Yamaha CS-R5 Rivage PM5 Digital Mixing Surface \- Muzeek World, accessed April 3, 2026, [https://muzeekworld.com/products/yamaha-cs-r5-rivage-pm5-digital-mixing-surface](https://muzeekworld.com/products/yamaha-cs-r5-rivage-pm5-digital-mixing-surface)  
11. Yamaha RPio622 6-slot I/O Rack \- Sweetwater, accessed April 3, 2026, [https://www.sweetwater.com/store/detail/RPio622--yamaha-rpio622-6-slot-i-o-rack](https://www.sweetwater.com/store/detail/RPio622--yamaha-rpio622-6-slot-i-o-rack)  
12. RIVAGE PM Console Mixer Components \- Yamaha USA, accessed April 3, 2026, [https://usa.yamaha.com/products/proaudio/mixers/rivage\_pm/components.html](https://usa.yamaha.com/products/proaudio/mixers/rivage_pm/components.html)  
13. Quantum 338 \- DiGiCo, accessed April 3, 2026, [https://digico.biz/consoles/quantum338/](https://digico.biz/consoles/quantum338/)  
14. Quantum 338 \- DiGiCo, accessed April 3, 2026, [https://digico.biz/wp-content/uploads/2022/10/DiGiCo-Quantum-338-Data-Sheet.pdf](https://digico.biz/wp-content/uploads/2022/10/DiGiCo-Quantum-338-Data-Sheet.pdf)  
15. DiGiCo-SD12-Data-Sheet.pdf \- OVERVIEW DATASHEET, accessed April 3, 2026, [https://digico.biz/wp-content/uploads/2020/04/DiGiCo-SD12-Data-Sheet.pdf](https://digico.biz/wp-content/uploads/2020/04/DiGiCo-SD12-Data-Sheet.pdf)  
16. SD-Rack | DiGiCo, accessed April 3, 2026, [https://digico.biz/wp-content/uploads/2018/11/DiGiCo-SD-Rack-Data-Sheet-web.pdf](https://digico.biz/wp-content/uploads/2018/11/DiGiCo-SD-Rack-Data-Sheet-web.pdf)  
17. SD-Rack \- DiGiCo, accessed April 3, 2026, [https://digico.biz/racks/sd-rack/](https://digico.biz/racks/sd-rack/)  
18. DiGiCo SD-Rack Input/Output Rack \- Sonic Circus, accessed April 3, 2026, [https://soniccircus.com/product/digico-sd-rack-digital-stage-box/](https://soniccircus.com/product/digico-sd-rack-digital-stage-box/)  
19. DM64 Technical Datasheet | Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/content/uploads/2023/07/DM64-Datasheet-2.pdf](https://www.allen-heath.com/content/uploads/2023/07/DM64-Datasheet-2.pdf)  
20. S7000 Technical Datasheet | Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/content/uploads/2023/07/S7000-Datasheet-2.pdf](https://www.allen-heath.com/content/uploads/2023/07/S7000-Datasheet-2.pdf)  
21. Allen Heath DLIVE DM64 S Class MixRack 64x32 I/O Digital Stage Box \- Muzeek World, accessed April 3, 2026, [https://muzeekworld.com/products/allen-heath-dlive-dm64-s-class-mixrack-64x32-i-o-digital-stage-box](https://muzeekworld.com/products/allen-heath-dlive-dm64-s-class-mixrack-64x32-i-o-digital-stage-box)  
22. DM64U Technical Datasheet | Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/content/uploads/2025/03/DM64U-Datasheet.pdf](https://www.allen-heath.com/content/uploads/2025/03/DM64U-Datasheet.pdf)  
23. Allen & Heath AVANTIS-SOLO 64 Channel 12 Fader Digital Mixing Console with 15.6-Inch HD Capacitive Touchscreen \- Markertek, accessed April 3, 2026, [https://www.markertek.com/product/ah-avantis-solo/allen-heath-avantis-solo-64-channel-12-fader-digital-mixing-console-with-15-6-inch-hd-capacitive-touchscreen](https://www.markertek.com/product/ah-avantis-solo/allen-heath-avantis-solo-64-channel-12-fader-digital-mixing-console-with-15-6-inch-hd-capacitive-touchscreen)  
24. Avantis Technical Datasheet, accessed April 3, 2026, [https://www.ggvideo.com/ah/avantis\_specs.pdf](https://www.ggvideo.com/ah/avantis_specs.pdf)  
25. Avantis \- Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/hardware/avantis/](https://www.allen-heath.com/hardware/avantis/)  
26. Technical Datasheet | Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Technical-Datasheet\_G.pdf](https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Technical-Datasheet_G.pdf)  
27. SQ-7 \- Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/hardware/sq/sq-7/](https://www.allen-heath.com/hardware/sq/sq-7/)  
28. sq-7 \- digital mixer \- Allen & Heath, accessed April 3, 2026, [https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Cut-Sheet.pdf](https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Cut-Sheet.pdf)  
29. ULXD4Q \- Quad-Channel Digital Wireless Receiver \- Shure USA, accessed April 3, 2026, [https://www.shure.com/en-US/products/wireless-systems/ulx-d\_digital\_wireless/ulxd4q](https://www.shure.com/en-US/products/wireless-systems/ulx-d_digital_wireless/ulxd4q)  
30. AD4Q \- Digital Quad Receiver \- Broadcasters General Store, accessed April 3, 2026, [https://bgs.cc/content/SHU-AD4QUS%20MANUAL.pdf](https://bgs.cc/content/SHU-AD4QUS%20MANUAL.pdf)  
31. AD4Q User Guide \- Shure, accessed April 3, 2026, [https://www.shure.com/en-US/docs/guide/AD4Q](https://www.shure.com/en-US/docs/guide/AD4Q)  
32. ULX-D Dual and Quad User Guide \- Shure, accessed April 3, 2026, [https://www.shure.com/en-US/docs/guide/ulxd-dq](https://www.shure.com/en-US/docs/guide/ulxd-dq)  
33. Core 110f \- Q-SYS Help, accessed April 3, 2026, [https://q-syshelp.qsc.com/q-sys\_8.0/Content/Hardware/Cores/Core\_110f.htm](https://q-syshelp.qsc.com/q-sys_8.0/Content/Hardware/Cores/Core_110f.htm)  
34. Q-SYS Core 110f \- QSC Audio, accessed April 3, 2026, [https://www.qscaudio.com/resource-files/productresources/dn/dsp\_cores/core\_110f/q\_dn\_core\_110f\_specs.pdf](https://www.qscaudio.com/resource-files/productresources/dn/dsp_cores/core_110f/q_dn_core_110f_specs.pdf)  
35. REDNET A16R | Focusrite, accessed April 3, 2026, [https://fael-downloads-prod.focusrite.com/customer/prod/s3fs-public/focusrite/downloads/27394/rednet-a16r.pdf](https://fael-downloads-prod.focusrite.com/customer/prod/s3fs-public/focusrite/downloads/27394/rednet-a16r.pdf)  
36. REDNET D16R | Focusrite, accessed April 3, 2026, [https://fael-downloads-prod.focusrite.com/customer/prod/s3fs-public/focusrite/downloads/21449/rednet-d16r1.pdf](https://fael-downloads-prod.focusrite.com/customer/prod/s3fs-public/focusrite/downloads/21449/rednet-d16r1.pdf)  
37. RedNet A16R MkII (English) User Guide.pdf \- Focusrite, accessed April 3, 2026, [https://fael-downloads-prod.focusrite.com/customer/test/s3fs-public/downloads/RedNet%20A16R%20MkII%20%28English%29%20User%20Guide.pdf](https://fael-downloads-prod.focusrite.com/customer/test/s3fs-public/downloads/RedNet%20A16R%20MkII%20%28English%29%20User%20Guide.pdf)  
38. Focusrite \- RedNet A16R MkII \- Dante, accessed April 3, 2026, [https://www.getdante.com/product/focusrite-pro-rednet-a16r-mkii/](https://www.getdante.com/product/focusrite-pro-rednet-a16r-mkii/)  
39. RedNet D16-D16R User Manual.pdf \- Focusrite, accessed April 3, 2026, [https://fael-downloads-prod.focusrite.com/customer/test/s3fs-public/downloads/RedNet%20D16-D16R%20User%20Manual.pdf](https://fael-downloads-prod.focusrite.com/customer/test/s3fs-public/downloads/RedNet%20D16-D16R%20User%20Manual.pdf)  
40. Audinate AVIO Dante 2x2 AES3 Adapter | FrontEndAudio.com, accessed April 3, 2026, [https://www.frontendaudio.com/audinate-avio-dante-2x2-aes3-adapter/](https://www.frontendaudio.com/audinate-avio-dante-2x2-aes3-adapter/)  
41. Audinate Dante AVIO 2x2 AES3/EBU I/O Adapter for Dante Audio Network \- B\&H Photo, accessed April 3, 2026, [https://www.bhphotovideo.com/c/product/1417856-REG/audinate\_adp\_aes3\_au\_2x2\_2x2\_dante\_avio\_aes3.html](https://www.bhphotovideo.com/c/product/1417856-REG/audinate_adp_aes3_au_2x2_2x2_dante_avio_aes3.html)  
42. Audinate ADP-AES3-AU-2X2 Dante AVIO 2x2 AES3 Adapter \- Dale Pro Audio, accessed April 3, 2026, [https://daleproaudio.com/products/audinate-adp-aes3-au-2x2-dante-avio-2x2-aes3-adapter](https://daleproaudio.com/products/audinate-adp-aes3-au-2x2-dante-avio-2x2-aes3-adapter)  
43. ATEM 4 M/E Constellation HD – Tech Specs | Blackmagic Design \- Syntegra Partners, accessed April 3, 2026, [https://syntegrapartners.bg/index.php?dispatch=attachments.getfile\&attachment\_id=696](https://syntegrapartners.bg/index.php?dispatch=attachments.getfile&attachment_id=696)  
44. ATEM 4 M/E Constellation 4K – Tech Specs | Blackmagic Design, accessed April 3, 2026, [https://www.bhphotovideo.com/lit\_files/1092718.pdf](https://www.bhphotovideo.com/lit_files/1092718.pdf)  
45. ATEM 4 M/E Constellation 4K Plus \- Blackmagic Design, accessed April 3, 2026, [https://www.blackmagicdesign.com/products/atemconstellation/techspecs](https://www.blackmagicdesign.com/products/atemconstellation/techspecs)  
46. ATEM Constellation | Blackmagic Design, accessed April 3, 2026, [https://www.blackmagicdesign.com/products/atemconstellation](https://www.blackmagicdesign.com/products/atemconstellation)  
47. ATEM Constellation \- Blackmagic Design, accessed April 3, 2026, [https://www.blackmagicdesign.com/products/atemconstellation/design](https://www.blackmagicdesign.com/products/atemconstellation/design)  
48. Ultrix Specifications — Ross Video, accessed April 3, 2026, [https://www.rossvideo.com/products/routing-systems/ultrix/ultrix-specifications/](https://www.rossvideo.com/products/routing-systems/ultrix/ultrix-specifications/)  
49. HDC-3500 \- Pro Sony, accessed April 3, 2026, [https://pro.sony/ue\_US/products/4k-and-hd-camera-systems/hdc-3500](https://pro.sony/ue_US/products/4k-and-hd-camera-systems/hdc-3500)  
50. HDC Series HDC-3500, HDC-3100, HDC-3170 \- AV Broadcast, accessed April 3, 2026, [https://www.avbroadcast.fr/media/productfile/s/o/sony-hdc-3500-3100-3170-brochure.pdf](https://www.avbroadcast.fr/media/productfile/s/o/sony-hdc-3500-3100-3170-brochure.pdf)  
51. Evertz 5601MSC Master Sync Pulse Gen/Clock Rental \- PRG Gear, accessed April 3, 2026, [https://prggear.com/product/evertz-5601msc-master-sync-pulse-genclock/](https://prggear.com/product/evertz-5601msc-master-sync-pulse-genclock/)  
52. 5800MSC \- Sync Generator and PTP Root Leader Clock \- Evertz, accessed April 3, 2026, [https://evertz.com/products/5800MSC](https://evertz.com/products/5800MSC)  
53. 5600MSC, 5601MSC \- AV-iQ, accessed April 3, 2026, [http://cdn-docs.av-iq.com/dataSheet/5600MSC.pdf](http://cdn-docs.av-iq.com/dataSheet/5600MSC.pdf)  
54. Eclipse HX Interface Modules \- Clear-Com, accessed April 3, 2026, [https://clearcom.com/DownloadCenter/datasheets/EclipseHX/EclipseHX\_Interface-Modules\_Datasheet.pdf](https://clearcom.com/DownloadCenter/datasheets/EclipseHX/EclipseHX_Interface-Modules_Datasheet.pdf)  
55. Eclipse® HX Matrix Intercom \- Markertek, accessed April 3, 2026, [https://www.markertek.com/Attachments/Specifications/ClearCom/110\!340-Specifications.pdf](https://www.markertek.com/Attachments/Specifications/ClearCom/110!340-Specifications.pdf)  
56. artist the intercom \- Riedel Communications, accessed April 3, 2026, [https://www.riedel.net/fileadmin/user\_upload/800-downloads/02-Brochures/EN/ARTIST\_-\_The\_Intercom\_Brochure.pdf](https://www.riedel.net/fileadmin/user_upload/800-downloads/02-Brochures/EN/ARTIST_-_The_Intercom_Brochure.pdf)  
57. riedel-br-artist.pdf \- AVC Group, accessed April 3, 2026, [https://www.avc-group.com/assets/products/Riedel/pdf/riedel-br-artist.pdf](https://www.avc-group.com/assets/products/Riedel/pdf/riedel-br-artist.pdf)  
58. RIEDEL Artist 1024 Intercom Matrix Product Video \- YouTube, accessed April 3, 2026, [https://www.youtube.com/watch?v=hicwui8mAaQ](https://www.youtube.com/watch?v=hicwui8mAaQ)  
59. ARTIST \- Riedel Communications, accessed April 3, 2026, [https://www.riedel.net/fileadmin/user\_upload/10-products/11-intercom/111-artist/1112-configuration-software/Riedel\_Artist\_EN.pdf](https://www.riedel.net/fileadmin/user_upload/10-products/11-intercom/111-artist/1112-configuration-software/Riedel_Artist_EN.pdf)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACwElEQVR4Xu2XS6iNURTH/x4TkRKJ5BF5lSEKyUCRkmeiKEkUUQYmJlJmBhRJiYQBRigplIGJgUxIGHnm/SiR92P979rrnHXW2ft0657u7eb71b/7rf/ar7O/bz8uUFFR8Z/SVzQ5mr2J2aJPor+iAyFHxogeQ/N3RYMbsmU+QutQkdeiX6jn2f870WfnLamV7iFuQgdq3BftcvFKaBnjEXTgS53XCradmxyyFpqbERPCT2hudEx0F3vROPBhKb7sPMarXGxe6QdHWO5KNBN3UG5nHzTHMfYI7Px48KaEODcRb5M3Nfg5WG5ONBO5tg1bcgNiojtYCO2cy4aU1vcW0ZrgfYfWze0940VnoMtuK8o/njB3IZrCNmjubPCHi86JJgWfsN+2wU44gHWiPaIhomeil65MidIb52Z6Mj2fRrkcWQzNLRNNEE0UzYJu4px8xp5RoqvQ0y+2+SrjdYmH0AY/BJ/eoeB5DkPLxOOZGyjlYbnrwTNuQ/MLRPOT+Mz96Y9obL1oB/TIejRPBOOjwetgqGh6J+V/0C1oo5udR1q9bfbF3Mzg84flJozevOAZrfp5As0Nch4nhfxAfaJIP2jZcc6rwRnmftEZzU11yAloo/ykPaVB94H6uaP1N5rrbMp4HuYuRTOxH5r3VwqDPvcyY2fy2soiaKPxxClNDr2BLt6I+sTm6jzPeAYPAebiF2jY3Si+iOXJ99hFs+2wUW7I0fOfLfkm6h+8e+6Zda652Lwb7tnDi2b0DN7GmXsfE8IRNNdjfDE9x3F3iVPQJWFwjbOzEc57mrycDJ4iPravZjt02e9wORLrG7uh/oOYSKxGY72DKd4APcq5lNvKeWgHL9LfaY3ppgnx8tgG/wV6cVuRYv+vyVfo5Y5v2LfDF8SN9phoZK10Hp5mVo9bAu9gfH7jC1VUVFRUVFT0av4B4w/e4rDvJM4AAAAASUVORK5CYII=>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACsUlEQVR4Xu2XS6iOURSGX6QQA4XSySVCZgYojlKUkVwHlIEMTIQkl5TOxEwyMKCQyACZUCYoA0bCxABTMXBNieRuvWet9f/7rLP3fz79X53ke+rt7P2u/d3W2Zf1Aw0NDf8pI0Xzovmv8TAagaui36InonEhVuID9Boq8lr0A+34R9E70afEW9MaPQzcR/tFch9ARkNj060/yvpTWyM6wySU7r0FGlsUA8J3aGxaDHQLP2h/NDuwF+UPuCd6GbxjKI+PcNzNaBqPUb7PUWjsSAx0yxjUlxz6J4O31PwqcFxvNI1OM9aX3NgY6JbxqCc5voQOB3+G+RuCT2aJLonWinYgf1+HsWvRFHZCY5eDP0V0RTQ3+ITPrcQE1JOcBVCf8ZTJ5h8MPjfTC9a+iM4zYzU0tk40WzRHtAS6iX+1fkqP6Bb09Iv3fJXxitSVnBVQf3fwJ5p/JvG4gVIpHHMneM4jaHyVaKWJbe5Pv6CzM4Ue2YrB78r+6eD1szCj5aITGZ/KUUoOpy/9PcGfZL5vlvww9mM9Q4/vkoOx3DPJc2iM24PDpJBvaCeK+NKfmXgtWAtEbRKdz/iluqGUnBFQ/1DwebzS51FMflo/ZXvGS2HsRjSN48g/l9DnXubsM68ydS0rQr90Wnmtw3a8nsd/9JyN0NjiGDC8Noo1znrzU7zQrEydyeEUZj2ScgADx7N9O+m7dzdppzzNeA6LTcbex4BwCoOvY/+6tdPlVuRvk+MFF0+hyDLkX4h7msNTJB3js2YXdGONexZj8Z6kD+o/iwGD20V6Hd+B/W3Qo5xLeUiqJoebG49BfswL+/sWegyn+ExhjcFrzg0M9/MAOuYztHBjDcQ+l4jzBVrc8T/sCaK4Z/G+ZzH0zxKeZn7dfNFma79JB3WianIaGhoaGhoaho0/CVjTI13KoTgAAAAASUVORK5CYII=>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAAC90lEQVR4Xu2XS8hNURTHF1KIgTxK4hMhAyWhvEMxkWeiDGRgIq/kkZIJIyEZUEgeA2RCmaAkhjKhPIbKAJGI5G3971rr3HXXPefe/XUHX7J/teqs/9pn77v32XvtdYkymcx/Sm+2iVH8V9jD9pvtD9v5xlAD10jaPGUbEGJVfCB5BxZ5w/aT6vFPbO/YPjttWdG6B3jBNtf5X6h5In1VG61+H/VHFC1ag0WIfRrrSWLTY4D5QRIbFQOdggntjmIJ8atuV/+y0x6wvXI+OELVE46g3a0oKo+pup/DJLGDMdAp/Sh9cbBbjL2qHXUa/JPOB7NUTwHtZkdRiR/HY0eufwx0ykBKW5zIR5If1Et9O0L7ixZCl+qrgg7Gkuy85WybqXryALHrUWS2kMSuBH0421W2CUEHGDeJQdT9xbFtPNlpU1Tb6TQwTHXsNA+S6QV9vkStd8ZSktgKtnFs49lmkiTxb+p7RrLdJrn9Yp+vS7RKurM4+BrHqZ5bhrjYQpJBtzkNDFb9jNOQQGEetLkbNOMRSXwx2yI1PCM/4fbsqjetAQ1soOaFgH86aDWmldh8thMlOqwVh0gGmqc+ti/8HUULYajqliwxMfixnoGG31IGYnGSxkuSGNKDgUUB36m+UMCO/hinFaAWiLaWpGaJeru6AbnG/2jz9xUtBFyv0HEVg1/qezaVaB7EbkZROUbl4wLoyGXGLtWSSTlWON/odEbQ4xfFc9VtZbVOfAfgiEbNWE3lYxtWG8UaZ6XqHis0k0lZnCcknd4LepwotjDqEQ+q6riAd5xv2n337HlWohkoNhF7HwPMKWp+D/4NffbHrZKUxZlKzQn0IslgS5w2RzUPfOQ0A7eIb2O7ZitJYo05K34A4wCJ/jwGFKQL/x5+A/yNJFc5jnJbUhYHrCHpHFew5Y0FDS0E2ymoMZAQzzWGazwkaYOiEoUbaiD4OCLGV5Lizv7LmWFs9HuW2v8twW1m701iW6fPb32jVqQuTiaTyWQymR7jL4Sn4WPCxu1iAAAAAElFTkSuQmCC>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADMAAAAYCAYAAABXysXfAAAB/UlEQVR4Xu2WzUtVQRjGXyVE0NKFoW4s1EUtChISxFWuatEiwY+FES5alCQJEm4iRBcuRIUWossCC1roQtwF/gmtaueiDyJEg1oo2ofP47zDnfvOXLsXIV2cH/xw5nln5pw5d85BkYyMjOMyCh/Y8Aj+wkV4Hw7AfrUP3grG/TdewT1xN0Yf5pePxM9JOROMOxFK3cwX2A4vwVbYDC/Dn+Ggk6KUzXTA2zYUt8apoJTNpFiDbSY7D1/CC9o/Ax/DCVjhB4m77mt4PcgsnDsG52CTqUUcZzM8Yj9sCLZgrbi1u+G65jc0Owu34TlYpdkdHRNyBf6C5drfgHW5cgwXGrJhkfyGLSYbh43wori13+ZVXcaPj81WTUaY39R2pfZrcuUYDnhkwyLgUUq9K36tBYnr/CWZhUeNFHqgzOlTieck4eBhGxbBZ4lvNoS1fZO90DzEH70U85LbEJ3NL8dwEF/OUvEXKARrU4ls12R8d1LrlAXtLvn39Q7hgBEbKj2w3oYK5323odIgrl5tcmbPEtlk0Cb+CD/RPrkH/wT9CH4ZOGnaFsQ9mUJPw9/sR1tQnks8L/W+8Cvlx3XCQW33wq/a9vAXvWayQ97ATXHn/pP+/SbxV2ZF3P9uKXgTyzZUPsB3Jrsr6Sf7Xtxa9kguaU534NX8ckZGRkbGKeQAjSN/QmXSfpQAAAAASUVORK5CYII=>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACo0lEQVR4Xu2WT4hPURTHT0NDU2YSmlEioyTyZ4QtjSSyGGVngaysJUmRzezEbIZmkqwkFlaTlKkZM0mZBVIWmkkkkf///zvf3znn907Hfc9rNhbep769d77nvHfvu+++ex9RRcV/y1TW0mj+SxpZw6xfrKshFznM+qHaHHLgGcl9oDI8p6weesvaoblx1ieX+8Ka0FxpVpNc3KLxWo1TfGP16/k01k/W9CxdB37ePVJcIamfHRMkg4/czZgoCy6+nfDuBu8FyWgbPSR165xnwO+NZgH2hlIcIsltiokyzCG5uC/4Y+obCzWe7zywIsRgLkltW0wUgHpM+RQfKP/h/8oukotPBn9QfeOGizFlikYTgxU7dIK1JngGFiPUH48JpejtYuAvsmY475Q7pwWUfoMP1Z+lsTVynrWetUxjTNOI79Ai1hCr3XmRCyS5vaxtrK2sLSqcI4cFMLKEsrag3azHrAZXUwPJOwkPwoLj42P1iqzT85wH4GEhWkXy/ZiX94CW2x90gHVdc5316oynIb7P6ghejY0kN8HUAwdJFhh4Nhp5HYT3xMX2/V1i7XF+Eaj/Hk3lPaXbjZxhbYimp5Vk/7vHWk6y//gbFz2g909rjKmCI+Ii0BbqJvP9GddoEj8VuCn2MgN7UKqh2AEfN+m5bdopLpPUzIwJyva/kZhwYKZhJygkdtK87S62DzoC72yI/f6HeJ87j6TaNo6Q5PJW7Ackg+jx+3Qd3OSji2+xXrrYQA3+OAz8svnONWuM6W4gxoqLUe52PrA3lPeAXyk/N0DyV4X8O9aonuNb/IOVJMlHekwtyYbVoHFMYb8kd2nOc1Q9fNPGFJLN+zXJ39Er1mfWTs2/USGHgcYD4H/UWMw652LUog288YqKioqKirL8Bib31NV33QxWAAAAAElFTkSuQmCC>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAYCAYAAAC4CK7hAAACM0lEQVR4Xu2WP0hWURjG3/5QSBDR0NLi0CRSYiBEQUNNTVHQ0lZLIc6CqOBcS/+mjJZoiMBAC/pHWATaIAlRuCs1aGGmVPTvfTzn1ed7vN/1c2u4P3j4zvs895x7zz333vOZVVRUNMoB15zrr2vMtas2XqHP9dW15DovWbDPNW5prGeSgVlLWeiHqylnnyRbyD44JNkXypa54LpK9R1LB7aTB967nlL9zvWaanDUUt+gTeqgw5J/TQOny1J2VoPMNzWCmGGZt1PqAB6vHuqLVIOfllaZQR8c+0h8gFVH1quBM+zarmYwbWsvUifyVuoA3mBu78k1fpkn2VfgTanp/LGU3RZ/s+uFeKX0WBroBHk6sYD9fmozuKAiH94v8UZdO3L2SjK8Pw1z0tIgV8RvZCIPqM3csGJfx8Qj85Cyj5Q1u7qpLuWy657rt+uYZHrSgP2X1GZwU+DvFV/HnKe2ZkXjrgtOiI4j5OnAAft3qc1ct+RvFZ/7tro662TnXAcp2xB64VoH7Nd7R25Zsc99Nees7udWwaN0U7wY6EiusTHpyQC8D7l9ONcb+WpB2Dda6mRDtrpZlnLaamcfhLcl12dyrcDjZUd9imqAO/pZPIB3Asev2aFt9fy6/5SCDrzJ7M+eblbw8C8guJQ95rHVflI3WTqmmbxgwlK2TQMrvrnrstvSJgTF/yB8MhUsMbI3rknXd0sXqiBbdN23dPzx2ngFfCGfq5mZcQ2oWVFRUVHxX/EPB+zEuFXm1ZUAAAAASUVORK5CYII=>

[image7]: <data:image/png;base64,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>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAYCAYAAACIhL/AAAAB9ElEQVR4Xu2WPUgcURSFb9AQFEwkIajYmS4SE2NAEAvBzkpQ7MUmomAdJaRKmgiioIiKlViIlYqNlYUo2hgipkoqm6hpDAQR/Lln77ubO3detS644H5wYM73HjNvd3beLFGRIjlTynnp5W14zfnDuebscCqTw1k+cs44/zh9bkw5JjkPkhfecyZMXyA5+VvjwCFnw/QDzpbplivK4wJjn9a7x64rcLFvG37Ky1w5ovTF/QL3XVfg5pyrCb7a+bwxTHKBDuP8gpWYn4m4Mc4755SHJOMjfiBGJ8nJx52PLQTEvHUvOJucOuMsnzkXJE/9M84lxedlGOUskUxqd2OxhYCYR5/lvOF8MM7Pwy7g3feIS1FLMmnNuNgFgPf6+1vm9BofA/Pw2/but3NR/IV9V7yfDl0fPPQYAyTjjc7DDTqXuaW4JRa9cGvof0P3wP1wXeeVh+Pu/8NZVil9vubgHljZFaSfrK4k9J7QPXBNrtv9D73fHCtfXAfbEZcB8pHpDcGtGwfg8NZRvgan6GZeZRx6Pec5yaIsGKsIx4uh4zWZ4inJqwk5JZk4mZghlJGM7XK+cc4peTt0i7J8Cu6X8+AJZ4/zk2QvxDz9tguOFkp/uDvjFaUXc0LysBYEuK12gfiHhI7bXDAMcVY485y25FCRe8QNve6fVIqXfDMAAAAASUVORK5CYII=>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACiElEQVR4Xu2WOYgVQRCGC10MvEBUVHTxAvFAUDA0EMFgZYMN1MRE0cRYDMQzMhMzbwPBRFTcaBVBYT0CEUxUEBENNPC+xfv4/9fVMzX1umf1RYLzwc/r+qvo7uk33dMiDQ3/LV3QfG/+K0yHfnlT2S0h9wPaXk0VPJNQk+vD81zKeuodtEpzD6BPJvcFeqi5jslN7iK03MTboLcmtvyUdB85zkuon+ATYISE3HWf6IQL0Edpn9ww6KnzCOumelOCv9+bNeQWlWyVkFvhE3/LNKgfeiHtg/UlPEJvhvOmqD/Z+XWwnq99itSCd0TsJPWAo9SjZqs3T2PPYWn390FLnBfhYcT6vT6h1P27M6FT0BjjHTDtgjNQt7ZTD0jOSjnYTehDNV1gJ8TFGIRmGc9zUkJuA9QLrYR6VGwzd7moLpkr5VjUOuixhO1UYTx0ycS5ByS3pdrpwmq6Bf0j0CIJ+yd6uT5jbrPTFuiK5uzhFnni4rvQYue18APnHpB7YbW270k5sdFFRbn/TkPrjV8H6797U+FbkpqL5xi0zJvkkIS/2pJ6wBPQVectlVB3x3gH1eOrwl/GdfANYF0n+y/Cz1f2UnFOwmtgFTtl+6jWMY6Hi2WHVCdgJzRS2/GjnYKnNmvG+YSU3z+/sJZb0ERvDkVq1XIT3QjdNzHr7PeP8SbT9qTGiuyUkMt9/zguF9HCW9GQpAZdk/AIPa40GavxpDLdihdIWOU9xifxH0r1S75KPjcAfZOQfw9d0zb3YhYe/TyZHqnYvmHyPDTYCTc+DwW27dUqdRnYpR7vlJHhEg6sN9BL6DX0GVqreV7/KOZeSXgA3kcjc6DjJmYtx+A/3tDQ0NDQ8Kf8BsHXzwDZH3wEAAAAAElFTkSuQmCC>

[image10]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACZElEQVR4Xu2WS4iOURjHH4aYTJOokWZFpiQ7KwuXsZpIU8q4LEQoRUnZSklqsprVSGY2I2XJ1sxCUpNLKLFCFhY2Mm65zOD/d84xzzzfeb7bgmk6v/rVd/7P+d7Led9z3iNSKBQKc4cz8AP8Ag+bmscAfAVPwgNwH9wL90TnBM/gLdV+Cu+qtsdL+KuKs5odNsjQLvkbYbbUhgb22QbXwy64OvodLlb9ZhWX4Q+41hYyPBZ/cK7Y0PDTBuCi1D8t/ymjcAJ22EIVvCng5dVYBl/YEPTDU6q9EY7ArSpbAwfhBZXl2A6vSljbarJAwprxGraaWj14g+Dl1cj1vwZXSHjL7sD3sDvW2H8Y3pbwQSBjMc/xER6JvzvhG1WbAdeKt/ABnG9qjeANgpd7HJKwQFvS1EvH48NMPIzZFpXtjplls8zMp+A31f4DR+wzvGkLTeINgpd7sO9BG8r01GH9vMpTxq2DhveVO+9pCfk43GRqf+EiOwkv2UKTeIPg5Tm4XrBvmy1EVkqoLzE5s7OZjEtEjnRNNa8tvUE3bKFBOI9zJ2L23IYO/DrmjpHgg7T1VTHTn/yFMdugMgsHmOsY+50wtQr4tLgw3YPzTK0e+qTywom9SN7EcdXWcF3JHSPBGrcWGn6t7H+GVNYDd8bf/K/ty/Zyk7m0wEcStvONbsB4omOqzb1K7mLoOpOTVPNg7Vwm+2QyLrB8k4k+Hn/vV+1dEjaaTcGp9k7qH1luAXgBfPuewK9S+Rb2wvsmS/DLUWtw+PA0fNvSm5FIaxfV518k09OfXle1pjlqg0KhUCgUCv+L37tmqMHzZecGAAAAAElFTkSuQmCC>

[image11]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAE0AAAAYCAYAAAC/SnD0AAADU0lEQVR4Xu2XWchNURTHlykyFIUXGUJCCCmJklIkJWRKXihlCPHgQSkUikiGUPJAhkJePJAHQwpPKKTQl6nMkXne/2/v5a77v+uc715K6Tu/Wt27//919t733LX3PkekoKDg39CfhcZG0xB9WGyAnyzUwqgQLyR2clniBBh460LMDTE7xMwQM1J0NXm9Q1yRmH/W6B7PJeZp5DFYSnnfQ9QZ74nxqgVVtoLFatkeYpdpf5A4eE+jdUtaVsAHo1Nb0R+aB8beLQ3nPZaY85qNxENpuA/LIxZqAQMNdzQ7gUUhtoQYInEJ9JJ4U1eG2GPycM0C0wZfJFZvFqiaYRKv7UiecjxEG4k5S8hT4F1kMQcUxx+hE+F/iLXT5rvSXMoH7izxGnxaziQ9C/XwiaXOtJV4o9ZLzGlSbv8G3hgWM5gTYgKLtYB9aiRpfNM8vlF7tfjX7BdfB7ghB9N35NiqVT6nz6+S3c88KXk7Qmw1ngeqOw+svL0h5rORBybwg0UD9qE1pJ0U/0ftFF8HGyQeHAA5d40HlkpcDQD+U+NZHkj0b6b2pNTO4jYLhltS2uNbSWVxuNyQOGBrNgzehC6Ir2+TqHdhQ2L1KMjh67GXAVQkvIXGs8Crc7RlpIHNIbqzmGgh5XO4RG0XlCWSOrFhwKBeR4fE17FcoGMPZGz+G2rb022j+H0r8MY52hHSQF4/eiDdDzGRPJcOEi9oyQaBnKssSvaetk98vV2Iw6Ztl/eIEAONhyXi9QFmSaU3KGmTSUfFHiWNwbaEazWaldsl8DDLAx+gtoI8/PMMDhN41Z6em0L0NW3sX5p3z+gA+jPSFK/CTyWNT1rMxat4RfORs1ZiH8dKdjnepu+dMLrB4gd6wJtC2rsQr0gDPOYAidfzDWuf9MWkK16Fo32CNMB5lmtS6Xu/px48fML0gtEnd7xKeeB5zp42+OeQ38NoYFrS+V/3+kbFQ+eqUfBAbOd6PsRH01aw3FexaEAfOM2VfkmrACca3ygNb+CpEr2hbBiuh3gvsayRO7bcrq+8lxJfhz6FWG68O+Y7ltJbiVWKXMxHn9mY8VKaN56vPKp5bcIjjfZzrtxqnHhFUJDDdPnL16bGSNayLsgBbzkFNYJ3yIKC/4hfz+j+1GjxTk4AAAAASUVORK5CYII=>

[image12]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD4AAAAYCAYAAACiNE5vAAACqUlEQVR4Xu2XS6hNURjHP1wid+JRV1EGSp4xUGaeSQlFUooJJe+IlKsMSCZGimJIkVdJ5DGQuleKlBRG3AEpJEnI2/fvW8v+9v+sve+5m4Hjnl/9u2f/vm/tvZe99jmLSJMmjcgEFv8SYzW7NUc0Q51f4z6D/pqJ5LrjJ4sq7NCsZ+m4Lnaht5rVVEvRKdZ/WzNXM0VzVdOh2aT5kbXKa7HenkwET3s7y3o5pfki2UU35Mu/QQ1PBKwIx6+ycg70of5G04dq4JpY/RB5uM/kynjOoipFE7+iuUDuslj/YvIA/gNLAj3DEm4fuTI+sqhK0cS/idWWOYd3Ee6lcyCunO7gnjHBDSJfxCrNApZVKZr4SM1xcrPE+u85h/cW7qRzRXyn4xOS/WPgc7urpeDxzHTNMc1aLqQomniK+J5Ocq7ep50ijr0Yjg9I+eQes3A8EvsFAQPFVmwpuPBGlgn6ifXeJf+nE7+UcFPJgYOa0SwD8Ys1couOk6BhM8sEnyS/xME0sfFd5COLNHM0MzWzNfNcbZTY2MHODQhunXORsonE+3iqWUi1QjBgC0vioeYMS7HfaIw/ywWlVez9x5NCD7LN1Y8G58F9wA0hj3OdJsdgfxCvg2CFloKmrSwd5zT7yT1znzH+qztmxon14EvH8yJ4z7uEA9hEtbB0xH0DevaKnQP3XQo/Cc9Oqd3VtWkOu2O8AqmbjZwXq48gf0Nqv6zQl1p9Zee/L7V1HC8ll2O4WBOWI4PtZlw2nPmur6/zzAwpri2RvMf2tcsdRyaL7fmLwDnwaxAZH1wSvJO4ELZ/WLb4i00JtrERnqwPJss8kaz+PvxdGWp3YhOxR7Ixu6gWqWeLinuP57mZLzUueJV6HcvlL25RG4me/K/tv+IBi94C9txNmlTgF2sawVLVy9X5AAAAAElFTkSuQmCC>

[image13]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC0AAAAYCAYAAABurXSEAAACDklEQVR4Xu2WPUtcURCGx4hgrAQLNfgFFlqJIIKgfyAoiIqWqSxEEBTETn+DhZ8oWISkEUVFECxESBFBkBQWosWCSaFok4AWFuq8e+Ysc2fvLmdxISvsAy975p055849956rREWKFBRNrDJr5oNG1os1s9DJ+kNuzqzJeRLk8lClyeUFv3gI3yhau866U7FmgcLXzYlD1gOFLd5Arm7A+PCGjAdy2Yxg6lg7rHsKW3yRXN0n48P7azwAf8Wab8U3Gtp0guLf0bgdxY3Bq1XeJGtcxZYZ1hqr1CY8W6x6GYc2vUGZd9rOXzXeP9ZH1jWrWvmgjVxts8TPEg+mKpgq1pGKQ5vGvLTFxLPzvfeBdSXesXga3Ai8duXhfNi6NCO0abBP0VrsXKamf7FOjG8fvd9Vjb1G8rG1aoNyaxr0sy7ILY6dtE3XSPxTfi9VzoL8bowX+YwesH4Y+YtijO9urmDudxUvi+fB+FzFng5yuW7jw5swXhp2pzzDlH5oUIcb9+BdtHPtI8f4VsZ6brnk8LQ8n8UrUV4scU1jUpyP+KuK0eC8igFqlky8KeNH5QPk5mTcI7G9ZoQz1g3rtwjjU5XHH55pFQP/KcMXAb9T0XQS+BUqbhHvSXmaPXI3g7OGOvTxrkDTo9YsJLDzIyrGf4xZX43/TS+5Bv1B/yJxV6qiQOkjd0C3WWMmV6RICK8oAKHsFEelPAAAAABJRU5ErkJggg==>