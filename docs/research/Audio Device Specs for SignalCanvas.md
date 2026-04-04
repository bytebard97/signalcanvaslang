# **Technical Engineering Specification Report: Professional Audio Infrastructure and I/O Topology for SignalCanvas Template Integration**

The current paradigm in professional sound reinforcement is defined by the transition from analog-centric signal chains to highly complex, software-defined audio networks. The development of SignalCanvas templates requires a meticulous understanding of the physical I/O ports, electrical characteristics, network synchronization protocols, and expansion capabilities of the industry’s most prominent hardware. This report provides an exhaustive technical analysis of thirty-four professional audio devices, categorizing their functional roles and documenting the specific parameters required for high-fidelity system modeling.

## **Architectural Foundations of Large-Format Mixing Systems**

The core of any sophisticated audio system is the mixing console, which serves as the primary aggregation point for both local and networked signals. The physical I/O on these surfaces is typically optimized for mission-critical monitoring and talkback, while the bulk of high-channel-count processing is offloaded to dedicated DSP engines or networked I/O racks.

### **Yamaha RIVAGE PM Series Ecosystem**

The Yamaha RIVAGE PM series represents a modular approach to high-density digital mixing. The system architecture separates the control surface from the digital signal processing engine, a design choice that allows for scalable processing power and redundant operations. The flagship CSD-R7 (PM7) and the CS-R10/CS-R10-S (PM10) surfaces utilize the TWINLANe and Dante protocols for inter-component communication.1

The CS-R10 and CSD-R7 surfaces share a standardized layout to ensure operator familiarity. The CS-R10 features two 15-inch touchscreens and a fader configuration of 38 faders (12+12+12+2). The local analog connectivity for the CS-R10, CS-R10-S, and CSD-R7 is standardized at eight analog inputs and eight analog outputs.1 A distinguishing feature of these local inputs is the integration of SILK processing technology, developed in collaboration with Rupert Neve Designs, which utilizes hybrid microphone preamplifiers to provide an analog-modeled saturation and "natural sound" character.2

Digital connectivity on these flagship surfaces is robust, providing eight AES/EBU inputs and eight AES/EBU outputs with integrated sample rate conversion (SRC). For expansion, the CS-R10 and CS-R10-S support four HY slots and two MY (Mini-YGDAI) slots when paired with a DSP-RX or DSP-RX-EX engine. The PM7 (CSD-R7) incorporates three HY slots and two MY slots directly on the surface.2 These HY slots are critical for high-bandwidth networking, supporting cards that can transmit up to 256 channels of digital audio at 96kHz.2

The more compact surfaces in the range, the CS-R5 (PM5) and CS-R3 (PM3), prioritize space efficiency while maintaining the 96kHz XCVI processing core. The CS-R5 features three 15-inch touchscreens, whereas the CS-R3 utilizes a single screen. Both provide eight local analog inputs and outputs, though they lack the high-count AES/EBU ports of the PM10, offering four AES inputs and outputs on the CS-R5 and none on the CS-R3.3

| Feature | CS-R10 (PM10) | CSD-R7 (PM7) | CS-R5 (PM5) | CS-R3 (PM3) |
| :---- | :---- | :---- | :---- | :---- |
| Analog Inputs (Local) | 8 (SILK) | 8 (SILK) | 8 | 8 |
| Analog Outputs (Local) | 8 | 8 | 8 | 8 |
| AES/EBU I/O (Local) | 8 In / 8 Out | 8 In / 8 Out | 4 In / 4 Out | N/A |
| HY Slots (Expansion) | 4 (via DSP) | 3 | 4 (via DSP) | 4 (via DSP) |
| MY Slots (Expansion) | 2+2 (via DSP) | 2 | 2+2 (via DSP) | 2+2 (via DSP) |
| Fader Count | 38 | 38 | 38 | 38 |
| Touchscreens | 2 x 15" | 2 x 15" | 3 x 15" | 1 x 15" |
| Power Consumption | 380W | 300W | 200W | 190W |
| Weight (kg) | 85kg | 94kg | 42kg | 38kg |

Data compiled from Yamaha RIVAGE PM Series Specifications.1

The electrical specifications for these units are designed for the most demanding touring environments. The internal signal processing occurs at 40-bit or higher floating-point resolution, and the units feature dual redundant power supplies as standard.2 Connectivity for peripheral devices is extensive, including eight GPI inputs and outputs, MIDI In/Out, and five USB ports (one of which is dedicated to 2-track recording).1

### **DiGiCo SD and Quantum Series Topology**

DiGiCo consoles are built upon the Stealth Digital Processing and Super FPGA (Field Programmable Gate Array) technology. This architecture allows for a massive number of processing paths without the latency typical of traditional DSP chipsets. The SD7 and its Quantum 7 iteration are the definitive examples of high-density I/O platforms for international touring.6

The SD7 and Quantum 7 consoles utilize a dual-engine design, which provides total hardware redundancy. Each engine is capable of processing up to 256 channels at 48kHz or 96kHz.7 The local physical I/O on the rear panel of an SD7 or Quantum 7 includes 12 Mic/Line inputs, 12 Line outputs, and six AES/EBU inputs and outputs (representing 12 mono channels of digital I/O).6 For synchronization, the console provides BNC connectors for Word Clock I/O, Video Sync, and AES/EBU Sync.6

The MADI (Multichannel Audio Digital Interface) connectivity on the SD7 is a core component of its I/O flexibility, featuring four redundant MADI BNC I/O pairs. The Quantum 7 expands upon this with the inclusion of two DMI (Digital Multi-Interface) slots per engine, which can be populated with cards for Dante, Aviom, ADC, DAC, or KLANG integration.8 The physical dimensions of the SD7 and Quantum 7 are ![][image1] (width) by ![][image2] (depth) by ![][image3] (height), with a weight of ![][image4].6

The SD12 serves as a more compact 72-channel surface with 26 physical faders and dual 15-inch touchscreens.10 Its local I/O is slightly reduced compared to the SD7, featuring eight Mic/Line inputs, eight Line outputs, and four AES/EBU inputs and outputs. However, the SD12 maintains significant expandability via two DMI slots and two MADI BNC I/O pairs.10

| Parameter | DiGiCo SD7 | DiGiCo Quantum 7 | DiGiCo SD12 |
| :---- | :---- | :---- | :---- |
| Processing Channels | 253 | 256 | 72 |
| Local Analog I/O | 12 In / 12 Out | 12 In / 12 Out | 8 In / 8 Out |
| Local AES/EBU I/O | 12 Ch / 12 Ch | 12 Ch / 12 Ch | 8 Ch / 8 Ch |
| MADI Interfaces | 4 Redundant BNC | 8 BNC (4 at 96k) | 2 BNC I/O |
| Expansion Slots | Optocore / Waves | 2x DMI (per engine) | 2x DMI Slots |
| Fader Count | 38 x 100mm | 38 x 100mm | 26 x 100mm |
| Sample Rates | 48kHz / 96kHz | 48kHz / 96kHz | 48kHz / 96kHz |
| Weight (kg) | 107kg | 107kg | 42kg |

Data compiled from DiGiCo SD and Quantum Series Datasheets.6

The audio performance of the DiGiCo range is defined by a frequency response of ![][image5] between ![][image6] and ![][image7], with a total harmonic distortion (THD) of less than ![][image8] at unity gain.6 The maximum input and output levels are rated at ![][image9], providing significant headroom for professional environments.6

### **Allen & Heath dLive and SQ Series Architecture**

Allen & Heath dLive systems are designed around the XCVI 96kHz FPGA core, which provides a 96-bit accumulator for massive internal headroom and extremely low latency (![][image10]).11 The S7000 is the largest surface in the dLive S-Class, featuring 36 faders across six layers, resulting in 216 assignable fader strips.11

The rear panel of the S7000 surface provides a connection hub for local I/O and expansion. This includes eight XLR mic/line inputs with remote-controlled analog gain (adjustable in ![][image11] steps) and eight XLR line outputs.11 Digital I/O consists of two stereo AES3 inputs and three stereo AES3 outputs, all with independent sample rate conversion.11 For system expansion, the S7000 features two 128-channel I/O ports for modules such as Dante, MADI, or Waves SoundGrid, as well as dual redundant GigaACE links for connection to a MixRack.11

The SQ-7 represents a more integrated, compact solution for 96kHz mixing. It features 32 onboard mic preamps and 16 XLR line outputs.15 Its SLink EtherCON port is a versatile interface that supports multiple protocols, including dSnake, ME, DX, and GigaACE, allowing it to interface with a wide range of remote expanders.12

| Specification | dLive S7000 | Allen & Heath SQ-7 |
| :---- | :---- | :---- |
| Faders | 36 (6 Layers) | 33 (6 Layers) |
| Local Mic/Line Inputs | 8 XLR | 32 XLR |
| Local Line Outputs | 8 XLR | 16 XLR \+ 2 TRS |
| Digital I/O | 2x AES3 In / 3x AES3 Out | 1x AES3 Out |
| Expansion Slots | 2x I/O Ports (128-ch) | 1x I/O Port (64-ch) |
| Networking | Dual Redundant GigaACE | SLink / USB-B |
| Latency | \< 0.7ms | \< 0.7ms |
| Sample Rate | 96kHz | 96kHz |
| Weight (kg) | 41kg | 17.8kg |

Data compiled from Allen & Heath dLive and SQ-7 Technical Datasheets.11

The electrical characteristics of the Allen & Heath preamps include a dynamic range of ![][image12] and an equivalent input noise (EIN) of ![][image13] with a ![][image14] source.11 The analog gain range is ![][image15] to ![][image16], supplemented by a switchable ![][image17] pad for line-level sources.11

## **Workhorse Mixing Consoles: Yamaha CL and QL Series**

For the past decade, the Yamaha CL and QL series have served as the standard for networked audio integration, primarily due to their native Dante implementation and the inclusion of high-quality internal processing modeling classic hardware.

### **Yamaha CL Series: Dante-Centric Modular Mixing**

The CL series (CL5, CL3, CL1) is designed with a minimal local analog I/O footprint to encourage the use of the Dante network for all primary signal paths. The CL5 features 72 mono and eight stereo input channels, controlled by 34 faders (16+8+8+2).18 Its rear panel OMNI I/O consists of eight analog inputs (XLR-3-31) and eight analog outputs (XLR-3-32).18

Key technical specifications for the CL series include a fader resolution of 1024 steps and a signal delay of less than ![][image18] from OMNI IN to OMNI OUT at ![][image19].20 The consoles include three MY expansion slots, which are essential for adding legacy format connectivity (AES/EBU, ADAT) or specialized processing such as the MY8-LAKE card.18

| Model | Mono Channels | Stereo Channels | Fader Config | OMNI I/O | MY Slots |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Yamaha CL5 | 72 | 8 | 16+8+8+2 | 8 In / 8 Out | 3 |
| Yamaha CL3 | 64 | 8 | 16+8+2 | 8 In / 8 Out | 3 |
| Yamaha CL1 | 48 | 8 | 8+8+2 | 8 In / 8 Out | 3 |

Data compiled from Yamaha CL Series Brochures and Reference Manuals.18

Electrical performance is characterized by a frequency response of ![][image20] from ![][image6] to ![][image7]. The DA converters provide a typical dynamic range of ![][image21], while the entire OMNI IN to OMNI OUT path provides ![][image22].18 Crosstalk is exceptionally low, measured at ![][image23] at ![][image24] between adjacent channels.18

### **Yamaha QL Series: Standalone and Integrated Processing**

The QL series (QL5, QL1) provides a higher density of local analog I/O, making them suitable for standalone operation without a dedicated stagebox. The QL5 features 32 Mic/Line inputs and 16 Line outputs, while the QL1 provides 16 inputs and eight outputs.22 Both consoles utilize the same Dante networking capabilities as the CL series, allowing them to function as remote I/O devices for other consoles via the "Port to Port" feature.22

A standout feature of the QL series is the integrated Dugan Automixer, which can manage up to 16 channels of speech microphones simultaneously, making these consoles ideal for corporate events and broadcast applications.22 The physical dimensions of the QL5 are ![][image25], while the QL1 measures ![][image26].22

| Model | Mono Channels | Local Analog I/O | Dante I/O Channels | MY Slots | Power (W) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Yamaha QL5 | 64 | 32 In / 16 Out | 64 In / 64 Out | 2 | 200W |
| Yamaha QL1 | 32 | 16 In / 8 Out | 32 In / 32 Out | 2 | 135W |

Data compiled from Yamaha QL Series Specifications.22

Synchronization for both CL and QL consoles is managed via BNC Word Clock I/O (TTL/75$\\Omega$ terminated) and MIDI 5-pin DIN connectors.18 They also feature a 5-in/5-out GPI interface on a D-Sub 15-pin connector.18

## **Networked I/O Racks and Digital Stageboxes**

The shift from analog multicore cables to digital snakes is facilitated by networked I/O racks. These devices perform the crucial task of high-quality analog-to-digital conversion as close to the source as possible, reducing signal degradation and noise pickup.

### **Yamaha R-Series: The Standard for Dante I/O**

The second-generation Yamaha R-series (Rio3224-D2, Rio1608-D2, and Tio1608-D2) provides high-performance Dante connectivity with significant improvements in monitoring and redundancy. The Rio3224-D2 is a 5U rack unit providing 32 analog inputs, 16 analog outputs, and four stereo AES/EBU outputs (8 channels total).21

A critical feature of the Rio-D2 units is the character/icon display, which allows engineers to visually confirm Dante settings, gain levels, high-pass filter status, and phantom power directly from the front panel.26 The units also feature a "Gain Compensation" function, which automatically adjusts digital gain to compensate for analog gain changes, ensuring consistent levels across a multi-console network.26

The Tio1608-D2 is a more compact 2U stagebox designed primarily for TF and DM3 series consoles, but it is fully compatible with CL, QL, and RIVAGE PM systems at up to 96kHz.21 It features 16 combo jack inputs (supporting both XLR and TRS) and eight XLR outputs.29

| Unit | Analog I/O | Digital Output | Sample Rates (kHz) | Power (W) | Weight (kg) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Rio3224-D2 | 32 In / 16 Out | 4x AES/EBU | 44.1, 48, 88.2, 96 | 120W | 13.5kg |
| Rio1608-D2 | 16 In / 8 Out | N/A | 44.1, 48, 88.2, 96 | 72W | 9.6kg |
| Tio1608-D2 | 16 In / 8 Out | N/A | 44.1, 48, 88.2, 96 | 50W | 5.7kg |

Data compiled from Yamaha R-Series Specification Sheets.21

Electrical characteristics for the Rio3224-D2 include a frequency response of ![][image20] (20Hz to 40kHz at 96kHz sampling) and a dynamic range of ![][image21] for the DA converters.27 The inputs are balanced XLR-3-31 with a ![][image27] load impedance.27

### **High-Performance and Modular Stageboxes**

The Midas DL32 is a fixed-architecture stagebox featuring 32 Midas PRO microphone preamplifiers and 16 line-level outputs.30 It uses the AES50 protocol to deliver 32 channels of audio over a single CAT5 cable at distances up to 100 meters. Additionally, it features two AES-3 (AES/EBU) ports for direct connection to digital PA controllers and ULTRANET connectivity for personal monitoring.30

The DiGiCo SD-Rack is a flagship modular I/O system, offering 14 card slots that can be configured for a total of 56 inputs and 56 outputs.31 It supports sampling rates up to 192kHz and features DiGiCo's proprietary Gain Tracking™ technology, allowing multiple consoles to share the same rack without interfering with each other’s gain settings.31 The rack can be equipped with various modules, including 32-bit ADC/DAC cards, Dante, Aviom, and AES/EBU.32

| Device | Primary Protocol | Max Inputs | Max Outputs | Expansion Type |
| :---- | :---- | :---- | :---- | :---- |
| Midas DL32 | AES50 | 32 | 16 | ULTRANET / ADAT |
| DiGiCo SD-Rack | Optocore / MADI | 56 | 56 | 14x Modular Slots |
| DiGiCo A168 | 3232 (Proprietary) | 16 | 8 | Cascade Mode |
| A\&H DX168 | DX (96kHz) | 16 | 8 | Redundant / Cascade |

Data compiled from Manufacturer Datasheets.30

The DiGiCo A168 and Allen & Heath DX168 are portable 16-in/8-out expanders designed for floor placement or 19-inch rack mounting.33 The DX168 is a 96kHz device that connects via EtherCON and supports redundant connection to compatible hardware.34

## **Digital Wireless Receivers and RF Infrastructure**

Professional wireless systems have transitioned to digital transmission to increase spectral efficiency and integrate with networked audio workflows. The physical I/O on these receivers now includes sophisticated Ethernet switching and high-resolution digital outputs.

### **Shure Axient Digital: High-Tier RF Reliability**

The Shure Axient Digital AD4Q (four-channel) and AD4D (two-channel) receivers are engineered for the most demanding RF environments. They feature wide tuning ranges (up to 184MHz) and "Quadversity" mode, which uses four antennas to ensure a stable signal even in complex multipath environments.35

The AD4Q rear panel includes four Ethernet ports which can be configured in "Split-Redundant" or "Switched" modes. In Split-Redundant mode, two ports are dedicated to Shure Control traffic and two ports to Dante/AES67 digital audio, providing a physical separation of control and audio networks.36 The unit also provides four transformer-balanced XLR outputs, which can be reconfigured to deliver AES3 digital audio on outputs 3 and 4\.36

| Feature | Shure AD4Q | Shure AD4D | Shure ULXD4Q |
| :---- | :---- | :---- | :---- |
| Channels | 4 | 2 | 4 |
| Ethernet Ports | 4 | 2 | 2 |
| Audio Networking | Dante / AES67 | Dante / AES67 | Dante |
| RF Features | Quadversity / HD | True Diversity / HD | RF Cascade / HD |
| Analog Outputs | 4x XLR | 2x XLR | 4x XLR |
| Digital Outputs | 2x AES3 | 1x AES3 | N/A |
| Latency | 2ms | 2ms | 2.9ms |

Data compiled from Shure Product Guides.35

The Shure ULXD4Q is a quad-channel receiver that emphasizes density and efficiency. It supports Dante networking and features RF cascade ports, allowing the RF signal to be shared across multiple units without external antenna splitters.39

### **Sennheiser Evolution Wireless Digital (EW-DX)**

The Sennheiser EW-DX EM 4 is a four-channel digital receiver housed in a 1U rackmount chassis. It provides a frequency response of ![][image6] to ![][image7] and a massive dynamic range of ![][image28], which eliminates the need for transmitter gain adjustments.41

Rear panel connectivity includes four network ports (RJ-45) that can be flexibly assigned to different network roles, such as Control, Dante Primary, or Dante Secondary.42 It also includes four XLR and four 1/4-inch outputs, along with BNC Word Clock I/O for digital synchronization.41 The system latency is rated at 1.9ms, utilizing the proprietary Sennheiser Performance Audio Codec (SePAC).42

| Parameter | Sennheiser EW-DX EM 4 |
| :---- | :---- |
| Channels | 4 |
| Network Ports | 4x RJ-45 (Dante/Control) |
| Latency | 1.9ms |
| Dynamic Range | 134dB (A-weighted) |
| Audio Outputs | 4x XLR, 4x 1/4" |
| Word Clock | 2x BNC (In, Thru) |
| Antenna Ports | 2x BNC (50$\\Omega$) |
| Dimensions | 483 x 44 x 373 mm |

Data compiled from Sennheiser EW-DX Technical Specifications.41

## **In-Ear Monitoring and Personal Mixing Systems**

For performers, the accuracy and latency of the monitor mix are critical. Modern systems utilize stereo wireless transmitters and networked personal mixers to provide high-fidelity audio with local control.

### **Wireless IEM Transmitters: PSM and G4 Series**

The Shure PSM1000 (P10T) and PSM900 (P9T) systems are the industry standard for high-performance in-ear monitoring. The P10T is a full-rack, dual-channel transmitter that features internal power supplies and networked control.45 The P9T is a half-rack, single-channel transmitter that offers state-of-the-art CueMode for multi-system monitoring.45 Both systems feature balanced 1/4-inch and XLR combo inputs and loop-out connectors for daisy-chaining multiple mixes.45

The Sennheiser SR IEM G4 is a half-rack stereo transmitter that provides wideband FM stereo modulation. It offers up to ![][image29] of RF output power and includes an Ethernet port (RJ-45) for remote monitoring via the Sennheiser WSM software.46 The frequency response is ![][image30] to ![][image31], and it features two balanced XLR/1/4-inch combo inputs.46

| Device | Channels | Inputs | Outputs | RF Output |
| :---- | :---- | :---- | :---- | :---- |
| Shure P10T | 2 | 2x Combo XLR/TRS | 2x 1/4" Loop | Up to 100mW |
| Shure P9T | 1 | 2x Combo XLR/TRS | 2x 1/4" Loop | Up to 100mW |
| Sennheiser SR IEM G4 | 1 | 2x Combo XLR/TRS | 2x 1/4" Loop | Up to 50mW |

Data compiled from Shure and Sennheiser Product Guides.45

### **Personal Monitoring Mixers: ME-1 and A360**

The Allen & Heath ME-1 is a 40-channel personal mixer that is natively compatible with dLive, SQ, and Qu consoles. It can also be integrated with DiGiCo, Yamaha, and Avid consoles via the ME-U hub.50 The ME-1 is powered via PoE or an external 12V DC supply and features an OLED display for channel naming.50

The Aviom A360 is an advanced personal mixer that supports up to 64 channels. It includes a "Dual Profile Channel" for the musician's most critical input and features a mono mix out on an XLR connector, specifically designed to drive tactile transducers (bass shakers) for drummers and bassists.51

| Feature | Allen & Heath ME-1 | Aviom A360 |
| :---- | :---- | :---- |
| Channel Count | 40 | Up to 64 |
| Network Protocol | ME-Dante / A-Net | Pro16 / Pro16e A-Net |
| Stereo Mix Out | 1/4" and 3.5mm | 1/4" and 1/8" |
| Mono Mix Out | 1/4" Mono | XLR Balanced Mono |
| Power | PoE / 12V DC | 24V DC / A-Net |
| Latency | \< 0.1ms (internal) | \< 0.88ms |

Data compiled from ME-1 and A360 Technical Datasheets.50

## **Power Amplification and Integrated System DSP**

The final stage of the signal chain involves converting low-level digital or analog signals into high-current loudspeaker drives. Modern amplifiers integrate significant DSP for loudspeaker management and utilize network protocols for system-wide monitoring.

### **Networked Performance Amplifiers**

The Crown CDi 4|1200 is a four-channel DriveCore amplifier providing 1200W per channel at ![][image32] or ![][image33]. It is capable of driving 70V/100V high-impedance loads directly without the need for an output transformer.52 Its rear panel includes terminal block inputs and barrier strip outputs, alongside an RJ45 Ethernet port for HiQnet control.52

The QSC CXD4.3Q is a network-integrated amplifier designed for the Q-SYS ecosystem. It utilizes Flexible Amplifier Summing Technology (FAST™) to distribute its total power capacity across one to four channels.54 The unit features dual gigabit Ethernet ports for network redundancy and includes eight bi-directional GPIO ports for external hardware control.54

The Lab.gruppen PLM+ 20K44 is a flagship touring amplifier, offering four channels of 5000W output and integrated Lake Processing.57 It provides eight dual-redundant Dante inputs and four analog inputs with Iso-Float™ ground isolation.57 The PLM+ series utilizes a regulated switch-mode power supply (R.SMPS™) to maintain performance during severe mains voltage fluctuations.57

| Amplifier | Channels | Max Power | Network | Inputs |
| :---- | :---- | :---- | :---- | :---- |
| Crown CDi 4|1200 | 4 | 4x 1200W | HiQnet | 2x 6-Pin Block |
| QSC CXD4.3Q | 4 | 4x 625W (8$\\Omega$) | Q-LAN | 4x Euroblock |
| Lab.gruppen 20K44 | 4 | 4x 5000W | Dante | 4x XLR / 2x AES |

Data compiled from Manufacturer Specifications.52

### **Open Architecture and Ecosystem DSP**

The Yamaha MRX7-D is a fully configurable signal processor providing eight balanced Mic/Line inputs on Euroblock connectors and two stereo unbalanced RCA inputs.61 It features a 64-channel Dante interface and a 16-channel YDIF digital I/O port.61 The unit includes a 16-in/8-out GPI interface and an SD card slot for playback of scheduled messages or background music.62

The QSC Core 110f is a multipurpose DSP with 24 channels of total analog I/O, including eight "Flex" channels that can be software-configured as either inputs or outputs.65 The Core 110f provides 128x128 channels of network audio via Q-LAN and features a single POTS telephone line alongside four VoIP instances.65 Its GPIO logic ports include 16 inputs and 16 outputs with a \+12V DC terminal for driving external components.68

| Processor | Analog I/O | Network I/O | GPIO | Power (W) |
| :---- | :---- | :---- | :---- | :---- |
| Yamaha MRX7-D | 8 In / 8 Out \+ RCA | 64x64 Dante | 16 In / 8 Out | 65W |
| QSC Core 110f | 8 In / 8 Out \+ 8 Flex | 128x128 Q-LAN | 16 In / 16 Out | 120W |

Data compiled from Yamaha and QSC Product Manuals.61

## **Engineering Implications for System Design**

The integration of these 34 devices into a unified system requires precise attention to electrical characteristics and synchronization. For SignalCanvas templates, the physical pinouts and grounding schemes are essential to ensure the virtual model matches the physical build.

### **Electrical and Signal Integrity Considerations**

Professional audio devices are characterized by balanced input and output topologies to reject common-mode noise. For example, the Yamaha Rio3224-D2 outputs have a source impedance of ![][image34] and are designed for use with ![][image35] lines.27 In templates, these values must be documented to calculate signal loss over long cable runs.

The crosstalk between channels is another critical performance metric. In high-density surfaces like the Yamaha CL5 or QL5, crosstalk is measured at ![][image23] at ![][image24] using a ![][image36] filter at ![][image37].18 This high level of isolation is necessary for maintaining spatial imaging and mix clarity.

### **Synchronization and Word Clock Management**

In a large-scale digital network, all devices must be synchronized to a common clock to prevent audible clicks, pops, or signal loss. Most devices analyzed, such as the DiGiCo SD-Rack, Yamaha CL5, and Allen & Heath S7000, feature BNC Word Clock I/O with ![][image34] termination.6 The clock stability of these units is typically ![][image38] for external synchronization across standard sample rates of ![][image39], ![][image19], ![][image40], and ![][image41].22

### **Network Redundancy and Reliability**

Reliability is bolstered by redundant connectivity. The Yamaha RIVAGE PM and Shure AD4Q systems utilize Primary and Secondary Dante ports to ensure that a single cable failure does not interrupt the audio stream.1 For power redundancy, flagship consoles like the Yamaha CL5 and Allen & Heath S7000 support simultaneous use of internal and external power supplies (such as the Yamaha PW800W), or feature hot-swappable dual redundant PSUs.11

## **Conclusion**

The meticulous documentation of these 34 devices reveals an industry that is rapidly standardizing on high-resolution digital networking while maintaining the ruggedness required for live environments. For SignalCanvas template developers, the data points regarding physical dimensions, connector types (XLR, Euroblock, etherCON, BNC), and electrical levels (nominal ![][image42], max ![][image43]) are the foundational elements of an accurate system design. As the industry moves toward ![][image41] as the baseline sample rate, the expandability through HY, MY, and DMI slots will remain the primary method for ensuring long-term hardware utility in evolving network topologies.

#### **Works cited**

1. RIVAGE PM Console Mixer Components \- Yamaha USA, accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/mixers/rivage\_pm/components.html](https://usa.yamaha.com/products/proaudio/mixers/rivage_pm/components.html)  
2. RIVAGE PM10 PM7 Brochure \- CCI Solutions, accessed March 4, 2026, [https://shop.ccisolutions.com/StoreFront/jsp/pdf/rivage\_pm10\_pm7\_brochure.pdf](https://shop.ccisolutions.com/StoreFront/jsp/pdf/rivage_pm10_pm7_brochure.pdf)  
3. RIVAGE PM Console Mixer Specs \- Yamaha USA, accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/mixers/rivage\_pm/specs.html](https://usa.yamaha.com/products/proaudio/mixers/rivage_pm/specs.html)  
4. RIVAGE PM Series \- Overview \- Mixers \- Products \- Audio \- Yamaha \- Business \- Malaysia, accessed March 4, 2026, [https://my.yamaha.com/en/business/audio/products/mixers/rivage-pm/](https://my.yamaha.com/en/business/audio/products/mixers/rivage-pm/)  
5. RIVAGE PM Console Mixers \- Yamaha USA, accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/mixers/rivage\_pm/index.html](https://usa.yamaha.com/products/proaudio/mixers/rivage_pm/index.html)  
6. SD7 \- Digico.biz, accessed March 4, 2026, [https://digico.biz/wp-content/uploads/2020/03/DiGiCo-SD7-Data-Sheet.pdf](https://digico.biz/wp-content/uploads/2020/03/DiGiCo-SD7-Data-Sheet.pdf)  
7. SD7 \- DiGiCo \- Digico.biz, accessed March 4, 2026, [https://digico.biz/consoles/sd7/](https://digico.biz/consoles/sd7/)  
8. Quantum 7 \- Digico.biz, accessed March 4, 2026, [https://digico.biz/wp-content/uploads/2020/03/DiGiCo-Quantum-7-Data-Sheet.pdf](https://digico.biz/wp-content/uploads/2020/03/DiGiCo-Quantum-7-Data-Sheet.pdf)  
9. DiGiCo X-SD7-Q7-NC SD7 Quantum Digital Mixing Console with OpticalCon Optics, accessed March 4, 2026, [https://shop.solotech.com/products/digico-x-sd7-q7-nc-sd7-quantum-digital-mixing-console-with-opticalcon-optics](https://shop.solotech.com/products/digico-x-sd7-q7-nc-sd7-quantum-digital-mixing-console-with-opticalcon-optics)  
10. SD12 \- DiGiCo, accessed March 4, 2026, [https://digico.biz/wp-content/uploads/2020/04/DiGiCo-SD12-Data-Sheet.pdf](https://digico.biz/wp-content/uploads/2020/04/DiGiCo-SD12-Data-Sheet.pdf)  
11. S7000 Technical Datasheet | Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/content/uploads/2023/07/S7000-Datasheet-2.pdf](https://www.allen-heath.com/content/uploads/2023/07/S7000-Datasheet-2.pdf)  
12. DIGITAL MIXER \- Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Cut-Sheet.pdf](https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Cut-Sheet.pdf)  
13. Allen & Heath dLive \- Sweetwater, accessed March 4, 2026, [https://www.sweetwater.com/allen-and-heath-dlive/series](https://www.sweetwater.com/allen-and-heath-dlive/series)  
14. dLive Surfaces \- Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/hardware/dlive-series/dlive-surfaces/](https://www.allen-heath.com/hardware/dlive-series/dlive-surfaces/)  
15. Technical Datasheet | Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Technical-Datasheet\_G.pdf](https://www.allen-heath.com/content/uploads/2023/06/SQ-7-Technical-Datasheet_G.pdf)  
16. SQ-7 \- Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/hardware/sq/sq-7/](https://www.allen-heath.com/hardware/sq/sq-7/)  
17. Weights-Measures-dLIVE-S7000 \- Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/content/uploads/2023/07/Weights-Measures-dLIVE-S7000.pdf](https://www.allen-heath.com/content/uploads/2023/07/Weights-Measures-dLIVE-S7000.pdf)  
18. CL5 CL3 CL1 \- Novelty, accessed March 4, 2026, [https://www.novelty.fr/wp-content/uploads/downloaded/downloads/materiel\_fiches\_techniques/yamaha\_CL\_datasheet.pdf](https://www.novelty.fr/wp-content/uploads/downloaded/downloads/materiel_fiches_techniques/yamaha_CL_datasheet.pdf)  
19. CL Series \- Specs \- Mixers \- Professional Audio \- Products \- Yamaha ..., accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/mixers/cl\_series/specs.html](https://usa.yamaha.com/products/proaudio/mixers/cl_series/specs.html)  
20. CL5 Data Sheet \- Rentex, accessed March 4, 2026, [https://www.rentex.com/wp-content/uploads/2020/01/CL5-Data-Sheet.pdf](https://www.rentex.com/wp-content/uploads/2020/01/CL5-Data-Sheet.pdf)  
21. YAMAHA RIO1608-D2 DANTE INTERFACE 16 mic/line in, 8 line outputs, 3U, accessed March 4, 2026, [https://www.canford.co.uk/Products/95-8962\_YAMAHA-RIO1608-D2-DANTE-INTERFACE-16-mic-line-in-8-line-outputs-3U](https://www.canford.co.uk/Products/95-8962_YAMAHA-RIO1608-D2-DANTE-INTERFACE-16-mic-line-in-8-line-outputs-3U)  
22. QL5 QL1 \- Acuson, accessed March 4, 2026, [http://www.acuson.it/wp-content/uploads/2020/01/Yamaha-QL-Series-Datasheet.pdf](http://www.acuson.it/wp-content/uploads/2020/01/Yamaha-QL-Series-Datasheet.pdf)  
23. QL Series \- Specs \- Mixers \- Professional Audio \- Products \- Yamaha ..., accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/mixers/ql\_series/specs.html](https://usa.yamaha.com/products/proaudio/mixers/ql_series/specs.html)  
24. QL Series \- Resources \- Mixers \- Products \- Audio \- Yamaha \- Business \- Denmark, accessed March 4, 2026, [https://dk.yamaha.com/en/business/audio/products/mixers/ql-series/resources.html](https://dk.yamaha.com/en/business/audio/products/mixers/ql-series/resources.html)  
25. Yamaha QL5, QL1 PDF \- Scribd, accessed March 4, 2026, [https://www.scribd.com/doc/297727652/Yamaha-QL5-QL1-pdf](https://www.scribd.com/doc/297727652/Yamaha-QL5-QL1-pdf)  
26. R Series (AD/DA): 2nd-generation \- Overview \- Interfaces \- Products \- Audio \- Yamaha \- Business \- Denmark, accessed March 4, 2026, [https://dk.yamaha.com/en/business/audio/products/interfaces/r-series-adda-2/](https://dk.yamaha.com/en/business/audio/products/interfaces/r-series-adda-2/)  
27. R Series (AD/DA): 2nd-generation \- Specs \- Audio and Network Interfaces and YGDAI Cards, accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/interfaces/r\_series\_adda\_2/specs.html](https://usa.yamaha.com/products/proaudio/interfaces/r_series_adda_2/specs.html)  
28. Rio3224-D2 Rio1608-D2 Leaflet \- Yamaha, accessed March 4, 2026, [https://usa.yamaha.com/files/download/brochure/1/1157751/rio3224-d2\_1608-d2\_leaflet.pdf](https://usa.yamaha.com/files/download/brochure/1/1157751/rio3224-d2_1608-d2_leaflet.pdf)  
29. Tio1608-D2 Dante Capable I/O Rack Specs \- Yamaha USA, accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/interfaces/tio1608-d2/specs.html](https://usa.yamaha.com/products/proaudio/interfaces/tio1608-d2/specs.html)  
30. Product | DL32 \- Midas, accessed March 4, 2026, [https://www.midasconsoles.com/product.html?modelCode=0606-ACR](https://www.midasconsoles.com/product.html?modelCode=0606-ACR)  
31. SD-Rack \- Digico.biz, accessed March 4, 2026, [https://digico.biz/wp-content/uploads/2018/11/DiGiCo-SD-Rack-Data-Sheet-web.pdf](https://digico.biz/wp-content/uploads/2018/11/DiGiCo-SD-Rack-Data-Sheet-web.pdf)  
32. SD-Rack \- Digico.biz, accessed March 4, 2026, [https://digico.biz/racks/sd-rack/](https://digico.biz/racks/sd-rack/)  
33. A168 Stage \- DiGiCo, accessed March 4, 2026, [https://digico.biz/wp-content/uploads/2020/04/A168-Guide.pdf](https://digico.biz/wp-content/uploads/2020/04/A168-Guide.pdf)  
34. DX168 \- Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/hardware/everything-i-o/dx168/](https://www.allen-heath.com/hardware/everything-i-o/dx168/)  
35. AD4Q \- Four-Channel Digital Wireless Receiver \- Shure USA, accessed March 4, 2026, [https://www.shure.com/en-US/products/wireless-systems/axient\_digital/ad4q](https://www.shure.com/en-US/products/wireless-systems/axient_digital/ad4q)  
36. AD4Q User Guide \- Shure, accessed March 4, 2026, [https://www.shure.com/en-US/docs/guide/AD4Q](https://www.shure.com/en-US/docs/guide/AD4Q)  
37. AD4D \- Two-Channel Digital Wireless Receiver \- Shure USA, accessed March 4, 2026, [https://www.shure.com/en-US/products/wireless-systems/axient\_digital/ad4d](https://www.shure.com/en-US/products/wireless-systems/axient_digital/ad4d)  
38. Control of Axient AD4Q in Split/Redundant Mode with QL1 : r/livesound \- Reddit, accessed March 4, 2026, [https://www.reddit.com/r/livesound/comments/1qgj94o/control\_of\_axient\_ad4q\_in\_splitredundant\_mode/](https://www.reddit.com/r/livesound/comments/1qgj94o/control_of_axient_ad4q_in_splitredundant_mode/)  
39. ULXD4Q \- Quad-Channel Digital Wireless Receiver \- Shure USA, accessed March 4, 2026, [https://www.shure.com/en-US/products/wireless-systems/ulx-d\_digital\_wireless/ulxd4q](https://www.shure.com/en-US/products/wireless-systems/ulx-d_digital_wireless/ulxd4q)  
40. ULX-D Dual and Quad User Guide \- Shure, accessed March 4, 2026, [https://www.shure.com/en-US/docs/guide/ulxd-dq](https://www.shure.com/en-US/docs/guide/ulxd-dq)  
41. Sennheiser EW-DX EM 4 Dante Wireless Receiver \- Q1-9 Band \- Sweetwater, accessed March 4, 2026, [https://www.sweetwater.com/store/detail/EWDXEM4Da-Q--sennheiser-ew-dx-em-4-dante-wireless-receiver-q1-9-band](https://www.sweetwater.com/store/detail/EWDXEM4Da-Q--sennheiser-ew-dx-em-4-dante-wireless-receiver-q1-9-band)  
42. EW-DX EM 4 DANTE (S1-10) \- Sennheiser, accessed March 4, 2026, [https://www.sennheiser.com/en-ie/catalog/products/wireless-systems/ew-dx-em-4-dante/ew-dx-em-4-dante-s1-10-509372](https://www.sennheiser.com/en-ie/catalog/products/wireless-systems/ew-dx-em-4-dante/ew-dx-em-4-dante-s1-10-509372)  
43. EW-DX EM 4 DANTE (Q1-9) \- Sennheiser, accessed March 4, 2026, [https://www.sennheiser.com/en-us/catalog/products/wireless-systems/ew-dx-em-4-dante/ew-dx-em-4-dante-q1-9-509370](https://www.sennheiser.com/en-us/catalog/products/wireless-systems/ew-dx-em-4-dante/ew-dx-em-4-dante-q1-9-509370)  
44. EW-DX EM 4 Dante rack receiver \- Sennheiser, accessed March 4, 2026, [https://docs.cloud.sennheiser.com/en-us/ew-d/ew-d/specifications-ew-dx-em4-dante.html](https://docs.cloud.sennheiser.com/en-us/ew-d/ew-d/specifications-ew-dx-em4-dante.html)  
45. P9T \- Wireless Transmitter \- Shure USA, accessed March 4, 2026, [https://www.shure.com/en-US/products/in-ear-monitoring/psm900/p9t](https://www.shure.com/en-US/products/in-ear-monitoring/psm900/p9t)  
46. Sennheiser SR IEM G4 Stereo Transmitter (A1: 470 to 516 MHz) \- B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/product/1385697-REG/sennheiser\_sr\_iem\_g4\_a1\_sr\_iem\_g4\_stereo.html](https://www.bhphotovideo.com/c/product/1385697-REG/sennheiser_sr_iem_g4_a1_sr_iem_g4_stereo.html)  
47. Stereo transmitter SR IEM G4 \- Sennheiser, accessed March 4, 2026, [https://www.sennheiser.com/en-us/catalog/products/wireless-systems/sr-iem-g4/sr-iem-g4-a-509618](https://www.sennheiser.com/en-us/catalog/products/wireless-systems/sr-iem-g4/sr-iem-g4-a-509618)  
48. Sennheiser SR IEM G4-A Transmitter \- Solotech, accessed March 4, 2026, [https://shop.solotech.com/products/sennheiser-sr-iem-g4-a-transmitter](https://shop.solotech.com/products/sennheiser-sr-iem-g4-a-transmitter)  
49. Sennheiser SR IEM G4 Stereo Transmitter G-Band (G: 566 to 608 MHz) \- Tour Supply, accessed March 4, 2026, [https://www.toursupply.com/sennheiser-sr-iem-g4-stereo-transmitter-g-band-g-566-to-608-mhz.html](https://www.toursupply.com/sennheiser-sr-iem-g4-stereo-transmitter-g-band-g-566-to-608-mhz.html)  
50. OVERVIEW \- Allen & Heath, accessed March 4, 2026, [https://www.allen-heath.com/content/uploads/2023/06/ME-1-Datasheet-WEB.pdf](https://www.allen-heath.com/content/uploads/2023/06/ME-1-Datasheet-WEB.pdf)  
51. A360 Personal Mixer \- Aviom Products, accessed March 4, 2026, [https://www.aviom.com/AviomProducts/A360-Personal-Mixer.php](https://www.aviom.com/AviomProducts/A360-Personal-Mixer.php)  
52. Crown Audio CDi 4|1200 4-Channel DriveCore Series Power Amplifier (1200W), accessed March 4, 2026, [https://www.bhphotovideo.com/c/product/1362989-REG/crown\_audio\_ncdi4x12\_u\_us\_cdi\_drivecore\_41200.html](https://www.bhphotovideo.com/c/product/1362989-REG/crown_audio_ncdi4x12_u_us_cdi_drivecore_41200.html)  
53. CDi 4|1200 | Crown Audio \- Professional Power Amplifiers, accessed March 4, 2026, [https://www.crownaudio.com/en/products/cdi-4-1200](https://www.crownaudio.com/en/products/cdi-4-1200)  
54. QSC CXD4.3Q CXD-Q Series Multi-Channel Network Processing Amplifiers \- AVGear.com, accessed March 4, 2026, [https://www.avgear.com/products/qsc-cxd4-3q-cxd-q-series-multi-channel-network-processing-amplifiers](https://www.avgear.com/products/qsc-cxd4-3q-cxd-q-series-multi-channel-network-processing-amplifiers)  
55. QSC CXD4.3 Power amplifier \- 500w x 4 at 70V, 625W x 4 at 8 ohms at Crutchfield, accessed March 4, 2026, [https://www.crutchfield.com/p\_907CXD43/QSC-CXD4-3.html](https://www.crutchfield.com/p_907CXD43/QSC-CXD4-3.html)  
56. CXD-Q Series CXD4.2Q \- QSC Audio, accessed March 4, 2026, [https://www.qscaudio.com/resource-files/productresources/amp/cxd-q/q\_amp\_cxdq\_specs.pdf](https://www.qscaudio.com/resource-files/productresources/amp/cxd-q/q_amp_cxdq_specs.pdf)  
57. PLM 20K44 · High-end professional sound systems, accessed March 4, 2026, [https://prodgsystems.com/3-5-33-121-producto-plm-series-lab-gruppen-amplifiers-plm-20k44.html](https://prodgsystems.com/3-5-33-121-producto-plm-series-lab-gruppen-amplifiers-plm-20k44.html)  
58. Lab Gruppen PLM 20K44 SP \- 5000W 4-Channel at 4-Ohm Power Amplifier with Lake DSP/Dante Networking and 3 x SpeakON Outputs in Black Finish \- Farralane Lighting & Audio, accessed March 4, 2026, [https://www.farralane.com/lab-gruppen-plm-20k44-sp-20-000-watt-amplifier-with-4-flexible-output-channels-on-speakon-connectors-lake-digital-signal-processing-and-digital-audio-networking-for-touring-applications.html](https://www.farralane.com/lab-gruppen-plm-20k44-sp-20-000-watt-amplifier-with-4-flexible-output-channels-on-speakon-connectors-lake-digital-signal-processing-and-digital-audio-networking-for-touring-applications.html)  
59. Lab Gruppen PLM 20K44 BP \- Professional Audio Design, Inc, accessed March 4, 2026, [https://www.proaudiodesign.com/products/lab-gruppen-plm-20k44-bp](https://www.proaudiodesign.com/products/lab-gruppen-plm-20k44-bp)  
60. PLM 20K44 / PLM 12K44 / PLM 5K44 \- Evi Audio France, accessed March 4, 2026, [https://www.eviaudio.fr/wp-content/uploads/2020/04/PLM-Series-Lake-Technical-Data-Sheet.pdf](https://www.eviaudio.fr/wp-content/uploads/2020/04/PLM-Series-Lake-Technical-Data-Sheet.pdf)  
61. MRX7-D \- Specs \- Processors \- Professional Audio \- Products \- Yamaha USA, accessed March 4, 2026, [https://usa.yamaha.com/products/proaudio/processors/mrx7-d/specs.html](https://usa.yamaha.com/products/proaudio/processors/mrx7-d/specs.html)  
62. MRX7-D Data Sheet \- Adeo Group, accessed March 4, 2026, [https://adeogroup.it/sites/default/files/prodotti\_allegati\_pubblici/yamaha\_scheda\_tecnica\_mrx7-d.pdf](https://adeogroup.it/sites/default/files/prodotti_allegati_pubblici/yamaha_scheda_tecnica_mrx7-d.pdf)  
63. Yamaha MRX7-D \- Commercial Audio, accessed March 4, 2026, [https://commercialaudiosolutions.com/product/791/yamaha-mrx7-d](https://commercialaudiosolutions.com/product/791/yamaha-mrx7-d)  
64. Yamaha MRX7-D Open Architecture DSP Signal Processor \- Markertek, accessed March 4, 2026, [https://www.markertek.com/product/ymh-mrx7-d/yamaha-mrx7-d-open-architecture-dsp-signal-processor](https://www.markertek.com/product/ymh-mrx7-d/yamaha-mrx7-d-open-architecture-dsp-signal-processor)  
65. Q-SYS Core 110f, accessed March 4, 2026, [https://www.qsys.com/resource-files/productresources/dn/dsp\_cores/core\_110f/q\_dn\_core\_110f\_spec\_sheet\_v2.pdf](https://www.qsys.com/resource-files/productresources/dn/dsp_cores/core_110f/q_dn_core_110f_spec_sheet_v2.pdf)  
66. Core 110f \- Q-SYS Help, accessed March 4, 2026, [https://q-syshelp.qsc.com/q-sys\_9.5/Content/Hardware/Processing/Core\_110f.htm](https://q-syshelp.qsc.com/q-sys_9.5/Content/Hardware/Processing/Core_110f.htm)  
67. Q-SYS Core 110f Q-SYS Core 110f \- AAT, accessed March 4, 2026, [https://aatsys.com/wp-content/uploads/2019/03/q\_core\_110f\_specs.pdf](https://aatsys.com/wp-content/uploads/2019/03/q_core_110f_specs.pdf)  
68. Technical Notes \- Q-SYS Core 110f \- QSC Audio, accessed March 4, 2026, [https://www.qscaudio.com/resource-files/productresources/dn/dsp\_cores/core\_110f/q\_dn\_core\_110f\_technote\_gpio.pdf](https://www.qscaudio.com/resource-files/productresources/dn/dsp_cores/core_110f/q_dn_core_110f_technote_gpio.pdf)  
69. GPIO Out (Core 110f, Core 110c) \- Q-SYS Help, accessed March 4, 2026, [https://q-syshelp.qsc.com/q-sys\_9.4/content/Schematic\_Library/io110\_gpio\_output.htm](https://q-syshelp.qsc.com/q-sys_9.4/content/Schematic_Library/io110_gpio_output.htm)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEkAAAAYCAYAAAC2odCOAAAC4ElEQVR4Xu2XS6hOURTHFyFKnmHEvUJ5jmRiQCYmyiOPERMDM/IoJJOL5DExUkoGJp5RSiFlZiClKCWRPEZEHnm/1t9e6571/b+973cN1EnnV//u2b+9v3PPWWefvb9PpKGhoaH+jGTRB0tYBOZrxrHMMFYzm2Vdmal5q9nDHQUeaLayVG5pvmq2aR5pvrd297JS80tzXbPXjie2jKgR3ZrBdowL7am6ijyUNJaLdFHzitwWzQ1y8yQ9EGe6pPMhtQcX2WkmLdCcl3yR4E6TG2M+gvZUckc0k8jVkv4UyW+4VKRj5EAsEgrBRSsxnIUyjIX83To6kIUxgEWJTkXCazbBjktFOkFuhHmfOWesjWJ90jyxNtaoCNa1g9Y3SPND0mzDWuhFxqt9TXM2OAeFe2Z+lmaN5ptmd3BYh3GMNfG1pPN3pK8iYbe6ENqlIl0lt9r8cmv7TT7vHVG9koutvVCz2Y7hv9ixA/dTWp++37jzLngEG4mDgmFDwUOPcKGzYNA+lgafIFekGeYjb8zhxgFmCI8BfjPgg/3FV4jSWF6/4OaG9ufgbwcP1ptncq4NDNrPUvIfzhUJYMZ9lNSP7R3Ez1+hthOL5NzLuHUZNyTjnJyH85nt4Louk8uCDx9gKdUNlLKoGtoGtvt4oTuo7fi5OrnHGXcu48B4yfuSG8UyBwZisewPGMszCa+Azx7nlOYwudJFskf7aMZhES+5eA589n5og25p/z9dweEXwPHQ1wYG8g2VwNi4GAKcnC+A2wBrzmRyGHcntFeZw0VH4A5l3CbNFM0u8itCG5w0H8GDdMdF/QPezRdSLajIe83TOCgAj93Gx76UtA4B/KzYKWnLnmP9pfccfXgd8F0Ii7sv1g5+3vDNrDU3lDzcBknbOHvGrzvi50XxN1LfP2G75q7mkqStvQS+FN6UtDgvoz7QpZlGDlv+UnIObpTJOTzQ0SwlzUKetQ0NDQ0NDf8hvwEb/eCrZRzi7wAAAABJRU5ErkJggg==>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD8AAAAYCAYAAABN9iVRAAACqElEQVR4Xu2WS+hOQRjGX7cQSpKNEtmIiLAgG8pCkg1lQ3+XLCgLG1nIBv0jGxE2pBQ2JBvJyiU2Iil2Iim33ELk9jzfzPje83zvd8Lqq//51VMzzztnzsycM/OOWUNDw0BnJjRIzcwFNf6BMWr0Ev3QJ2g39Aj6UQ23+FWjB0G7n6480cV7iqPQNvGGWRq05yG0A9oKbYY2QTuhL74RuAW9gO5BayXWc/Arn1HTOic/QupE25DFavQyjy1N4rjz+qC3rh4xCpqvpv3/5KPFHayGg3+nMlYNMFoNzzir7t8lFu95JfrqhJM/YSnOfhZVwzYPum1pu3Bge6E30B5Lz3BRV+fyLugbtKL1ZJvz0A3oe65zS16Fzlp77JegV5b6Z1/RYrWYYtUFmF4Nd3AAeqZmhs8Pz+Uhue5fXBbtYi4vcLFjlgZ8znmTrXOhX0Jzsk/5DMU643PF48KEnILeQdet3aEegh7GuWARI6V+GvqQy0Ohm7nMPg7lcoFt+Rd4Zll18jxkl0JHsq/joLc/8MJUzcBG8d5nf534hGlNv0Qdyyxu383TffsR2i4eKR9J6ebxL6wwKQci6Pv8Xej2UsKBcv95yuT9r1n2tPK3HqHPNO3ps87zSv+cP/DyEQYsXXa4B5W6ydN/Ih63j7bnAqm3IfDWO28ltMbF6E9wdfIUOigex3M4l+/6AGEnW9S05GuamJF9HWThmhqWtpB+IT6vX4gHqPbLy1LxeOp7tC2hp1dpeiVlvvYBMs1SAx42PKW5uqzzJqfwZK6bPHluKX2Ot5R+orb0mILU4xXbczL7l6GFzl8OfXX1QvQutpsKfbaadLcPum8p39ZdLq5As9V0cOWZh+9YvICEv7PCc6DjULJ0kdIMwvFFE+HWiVilRkNDQ0PDQOA3FQm8v5XyhtQAAAAASUVORK5CYII=>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD8AAAAYCAYAAABN9iVRAAACqklEQVR4Xu2WS8hOURSGl2uEcsvA6HeLIncDSiaYSMqlmGFiwMhl8A+UxESZyK3Eb2DAALkkAxNFMWImYeAycB25i3hfe2/f+t6zTt8//Mp56q2z3r3P3meds/fax6yhoaGhxSQ1MhOh+WoKg6EF0BBt6DZmQr8D7fadwLjs90Froa/QlLYeiZ/QRWgH9NHSPV0Lk38LPc2q+6pMgl9dvXku3p49D+Nf4nUN06HbagZoUoTeQxevzt4A55WV1JVMs87JL7U4gf4kxvaTagYMUwMMVMMR1ZPRaoCRanimWkp+E/QNemnVQfZZnGSn5G9YtX0hdM9SzeCDHYQ+QAcs9R0BbcjXe6EfllaU5xJ0x1J9IY+gW9B5a22xa9A7S+NzrOhlWY+lxpU5npvjZaUDOJY9pS754keTlv5X8vVi18YVwge+4Lweq87BGsVaU+bQbcZ2njbe44upMBRaLp4mdVzigvZT9lh7O4/Au/ma/hHXRs5ZWgWe2dY+Ri+0wlofRI9keocC77J4tdy3dMO2HHNLREl2Sp6wILLPIPGj++jpluNxuVM8Ujd3nafz/yUapCS/K8czcqzovVssFUfPfkt9+AILZU8r/fUI/RPibbbqsaorpw1NgHDp0RvrPO1D6F2VWPudzZ7/H2CB0n5bA48vs3hroI2ujf4EF5MX0GHxnkNH8/UD30A4CPeiep8Cjw+g3hiJ9VhjRdakGOsXepV9zxvnsep7tC+hNyrwypH53jcQvj124BHEfcEjiA/sKygpR9H4HLPgsEB55kCvoVk5Pm31D8kjSL3P4vVl/ya0xPmroO8uLkRzsR9/w79Y9eT5xynoCbROGxxM/Dr02Kr//oXJlo6xZ9AZiyfkclZYB6KitAgaLh6/ZDQut07EejUaGhoaGv4H/gC788gm8vIhEgAAAABJRU5ErkJggg==>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADIAAAAXCAYAAABJXhw0AAACE0lEQVR4Xu2Wv0scQRTHn6IQNU1CQAyIErRJFQhYK8HOdAlYRFATfxSChZWICFZWIkax0SaavyCxsLAIIpgygoVpAkGDjZ2I4q+8780b9t3bmTtNDlHYD3xh3+fNzt3szd4uUUZGxk2p5kxbqRjkXEmGTA+cc3Y5o+TG9nK6VTxtnCNK5ioJ7zg7lEwaW8g3zoWq9zm/VA38HLFY4Pas/FcaOZVyHFvII3K9BuXKxb1SDvUkZ5gzwOnjfBD/RI0DdeIxpuTEFrJJ17uiW+rYs8wZt5JZovCcJSG2kEJbI+Q9FZx2KwV77pjUPcpp8Asfc6akblK9FKVeyJkVCpz3SY47OE+lDs13wlmX449U/HNzzRkrKX5izIMWivdwX6D3jNPP6eI8EHeoxgFcDDsP6j/G5YEBs1ZS/AvHPIBfsFLw5yH2T0BzQG5MlXLYqnDPlUuBAXNWUvwLx/xrcr7VeA96eN5gS8XmAKHe94BLgQHzVjI/KXwy3KWVzAa53kPbENB7Y2oPtqQHHje4JrS4FBgQ2g4vKXwy3IiVVPzDbM/XZeSuuPY/VO3dZ+NyYBvgKX1KyRfA68NvPYiZkF6t1Dj+mrTzKLSQTnKvMhqMfRHwNdJbI3dzr0r93zzmfCG31epNT/Oe3AMvBN4IEMtbKwL4N4x7zyJn28q7zgqlrz5q/154b2im5F5DsKXxZ5CRcVv8BRG/rWxofGWXAAAAAElFTkSuQmCC>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEIAAAAYCAYAAABOQSt5AAACjElEQVR4Xu2XTaiPQRTGj28WpKRI2SisLChS6CZJSoodKVnYWFiRlA0Le8qCy4LCzlr5KBuFQoqSKBaKhZSv8jlPZ073eO6Zmf//xm5+9XTvPGfOeec9933nnSvS6XQ6E2cRGxVmJm1gcwimJq1js8JvJ8+VpK9J35y+JH1K+pD0aGzq4KDIIHxOepW0VXRhR/8OV5ktmvMi6WD+fRAWiM4d5UAmahJ4Iuov5kCN72wEoOiZwLtJXsQ1Gb9YjC+QF3Faxud6So0AtVjIDzaIzaIFl5OPR7B1IbwKmHOZ/ItJU8iLqN3MetHYOQ5kEHvDZo1WI26IFsX+4HmY/RpYJOas5kCDWfkncl/6gMPWtZADifMSr7lKqxE/Jb7h26J+bbO1v+i0pF9J9/J4vp+UmST6lN1POi5juXv8JIfFmaui/gwOGIcKwo2yB9nXoXTB66L+CPkey+V8jKcHHr4Gxs7slfC1vW6JNr7IioLwRLAHzdO08EaANWIbBxyWuz/w37vxsux58MizZ2wUjWEzZQ5LOa9K69V4JnHhO6L+XA44rBGTC74f43XwPE86S56BL120JgMxPOlD0WoEdnwUtifEwEJriwEfJZ4TNYI3NnjRRgg4n0HsAZstWo2YI1p4E/mtxQB7z7mJPhd7RVTHvJVJB5yPWog9dZ7nlGgch7GhaDUCoPDjwMNFjX3Ze+08AI9PhvD20tiDT6Z5XA+vC2I7yAfHRGPVQ+KlgvBZYw/CjRk4quICu/IYC8W53rNbdA72FM+S7G/PY9TGzu45InpyHRH9hK4VzVmTdDfPWZX0LvsQ/rd4K9oo75/M8/8rOBajCVs4MACWe4IDmaWizfRM5DqdTqfT6fwD/gBb09nS6KrFtgAAAABJRU5ErkJggg==>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC0AAAAYCAYAAABurXSEAAACCUlEQVR4Xu2VPUjVURjGXwzEhhBrqEG9YEtQDWKQOAlCQS6CYE2uUluBuAQKgkNboIuDDro7aIG0CKKoIEFDRNggNfSh+dHmR/o+nvd4z308R6/XTf4/ePC8z/Oe/zn377nnimRkXH4a2YhwS/WATeKJalv1U/XD/q6rblu+pvquWlF9s57HlhUNJv5TLagOVJuF8RFV4rJxVZuN7xV0nOSDuL5KDpQ34rKPHBQDJt6JeJDHbzgXeGXmtQQeg3yWTcOvcYWDYvCTsQkP3jq8AavnrGbg4V+bAnkzmwa/mHOBs7dD3hdxDxyyOrVAygfPJJ01iMt+cxDAR+oa1Sfwm7lPNZPywVdJZ+/FZS84UKol/9xX5n1S3TzuiIAvFybgiHhSm0v5wGenKQb8flW7asvq1wUdEdC0H/Fii6R8UGoWgiM2wiazqKpnU9KLpHxci/C7OTCQjbFJvGUjxqhqKagrVD02Tm3uLD92nZWLy+o4CHinamWTeamaIO+56qmNcfWlNjfPpqQ/DOiTdAaWVXfJ26NaHqn+izsauI/xtj+Le/DVoA91R1DnzKsNPPDQ/A3yPbuS3jTW/6UaFHdMm8TdQtNhE/BvJaaQXvP81YPxVD4++nHCnR/O/xPk+BHCG/PZX3Fn39OpmrHxsOT7Vn1DqVxXTYr79DWUXZQuqnOqG+RlZGRklMghI/msRN8BbvAAAAAASUVORK5CYII=>

[image7]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACeklEQVR4Xu2WS6hPURTGFwkpkZBHTDzLIzExMFAGBggpKck1MFAKZayUuiZKipEoMVBSSgaUkhAyVorrFUVCUt6sz17LWfe7a//di4HB+dXXf6/v2/vs8z9nn7OPSEtLy//GYjYSJqoWsRn4rPpuOkxZxnvVS9Uz0wvVU8tmqZ6relT3VI8sHzAPVO9UN6Wc2Ove8U/GSMlOqNZae3avHg37peRTOegA+p9i0/ALNpaD/oCBuFLsQY7/uSnBG2LekuA5PP53dEm9P1YVMr+rA8ZPZlDwPpi3x+rrVjPw7rMpxX/IZgewgrLjg0tSsi0c9BcszY/k+YQHrK7dkcyfbt5m8uMFZLLjOJ0yMJTq8VSn+EFnUs1kPp6j6D1RnZTyshgc/Eh2HKdT5ivN8zmq802cs0DKgLfBq02S+dHbZL8XzFtodaRLmjE14Q3KHFNdUa2UstLQj1diCjp+STz+IyDzUfdIeX5XmXdayvaRgTuMMfM4MJBtYDPhKxsZt1Xz2ZT8jwD2twUPuhqyGnyMyD6pZw7e8HG1VcH+diPUI1W7rF07CfaxCcca7WWhZuZK6fOKA+OT5PM60yRfvn3YrTpD3k7Vamt3Sz4RvMtUfwv1HWm+Zo6oRoUMnJMyZjv5DrLa1xCeZ14hO1RryJPlUk4KXzHXVLdUd6UcHJu5g3pdqH07mBA81AdDjc+wFSFjfAVkW8g4KVn22p8hJcMF2ihl/CHz+uCTZIrsNc8nRPtsE//y4l3aqjouZZtYGvyLUl4KPg+W4mPLjqrehAz9PHPgj1BNsrZrdOz0J+CBxj6DdT+ZMpA9b+tVw9n8S4ZRjZdbS0tLS8s/5Qe/p9ao5PNe2gAAAABJRU5ErkJggg==>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADMAAAAXCAYAAACmnHcKAAACZklEQVR4Xu2WOWhWURCFxxUEI4KgIChEsLCSgKiFmBQKQbEIxEoDgnaWEpdGrMRGKxET0gWtTJEmIooISdSQEMQUwTUQFO0sVAS3ZM4/9/6Z/7zJUzsh74MDb87M3d5y7xOpqFiSzKmeqt5wwrFZ1c5mxEHVcjb/go2qTjYdq9hQdlF8S7UpXV8SW9jxetY4l/xSbosVHVVNqT41pkv5oXqu2i3Wx9XGdA34rNMNFcVJIt6n+pauX6t6VT2+iJmQYkc3Ai8imjy8YfI+qMZU46rzlMvweByfUl0nrwAavSdvZ/KbyfccFqvZTj6eKk9klOIIbuPjZapfLg5ZIdboMfnrkz9AvuehWM1q8p8l3/Ovi1kr9sZkvqrWuDikVayTu5wQ81+y6cCd4kmDR2I+dp0MFnNSFtpcdrnMVtUDsc3iu9iNBmdVV3JRGdiB0PkgJ8T8skeLfLSY+2I+blQGcZuL36k6XJzpUvWrmpyHDcaDtybkT4uJJptZLJ8Xc8h5+901wFOL2jIt0rit4ybcEXty65xfA/s6OsUEPCuTP0m+Z1riCY2I+YXBCNR0s+nYK7YDZt6qZl0cjV0zcbZ4tiS/j3wPDjnUbCAfp7cf6CLFGXjILQa3QYxvKvPRXddB0U/yjiS/bAfBe42aA+TD8xPB7wlPDMDDTYvAU9hGHuqHXIw3oMBNKQ72IvB4ktnD6c/eNRe3SvF35oLYyR6BX6p7bIr1+4TiEHT8Kl2fECvk1ydaDLZTeMdSPKP6vJCug5o9Yv99Z1Kct16Gx8j4vxIcol9c7r/kNxvEDrEF+detoqJiqTMP0u6qadk5fHAAAAAASUVORK5CYII=>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACxElEQVR4Xu2XS8hNURTHl8grZULJZyIJRVEGBvL1pchjwkAGyIBkYkCRGWXEQAoheQ1MJGWmmCghKckjJBGZeZZHnuvf2qu77v/sc879bhep86t/9+z/WufufdbZe59zRBoaGhr+T4ar5rKZYZ5qJJsVTFP9SjpMsQ+qz6ovQZ9UH1VvVBdaqf+GyWIDvyE2IBwfbcswnooN+qZYztv2cCV7xc6ZxIGEFy+Cm/Ut4/eM72wQM8XuXmSj2IAwMAdtzIBI7oLK+Cnluf1isRMcSCB2j81eUFccTHN0jsFH+MK9PSR4WA7wdgWvDOQ9YzNxRSzexwFlqFhsAwd6QV1xJoh1fpJ8Lg5m19fQBlhmyNlPfg7krWMzwX1FsHRzsdFsdENdcXKMkeoBO54znXwsVfhnVOdUj1K7jFxfY1UvMz7Ast8tFot72JLkdUw3xXkh1slqDhDIec+mmI8COXeTl6NfWsVh7Qh5ERQNIOdQ8PGwKOsnSzfFQQfH2SRQuB9sis2WV+QdUT0hz7kq1t84DiiPpXixZ1XjVWtTDNuCgzZubIGDJcJGyx60yk4rwIPJcUuKBQBYCjh/FPnwJpLn+CwpA7F9bIr5vAfCm0FeJYOZOfjzOaF9LBw7p8XehxzsT1vT8RbJX2jOA17M+xxITBWLL+eAmL8p4w2KTouDxzK/oHFn21TnyUNBVqTjPVI8Z03wLsaAckAsVjaL8YTk/3PYXxA8bAm4abV0UhxscFjf11XXVLdVz6V9AIvFlijejpGD3IcpB2+ywO90BG33qmKMb+J3OJBAbFhoY5lfCrGOqCsO9h8fJCs+iTgWFXmgWqbanGID6RefD0tTzimx//bz8cmCi8Nmiu8q96u+8xC/rJqteq1ar3qn2q7a2Uqrpq44f4KFqvmhPUKK70K9YJFqZWjPElteHTOFjYaGhoaGhr/Obw8Z5N8oTxVKAAAAAElFTkSuQmCC>

[image10]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACgklEQVR4Xu2XS6hOURTHlzcDRXGVx5QBRYYG+oxuXgOKMiBKioHEwIgwMFA3kaSEEiJMZCYTZsQEeYQJMTKRRx5h/a21u/v7372/s++X20X7V/++c/5r7fNYZ5+1zydSqVQq/xeTVJPZ7ECP6gCb/wITVa/YzDBC9VN1S7Xbt1e2ZaTZJpab0+b+1L+DS6rbYjdcCm5kbcLbQx7zUnVatUu1VbVFrCA3VdeivGFlpOqu6ptqLsWaOChWCCY8/U6k4pix39kcLt6o3qmmcKCQL5K+yZLirGdDbAweVhPj2JDy2T6eDeaE2IWM4cAgyRUh53cCRVnHpnJH9VF1XzVKbGbtFzv+Q895q9qr+ux+igmqH2J9Ea/t4/awyGqxwS3yuyVXhDCjSmZB4D0bymL/vSJ2vK9R7JB7KBaKBvBawlsSkiJiPywcA1gmFrjIgS5oKg4utgRM9dRxPvgvnjjHj7s3O/Kmubc08gLwN/r2Dim4/yNigzZwoJBccXJ+jqZ8xE4lPB7zOuEFsOCEMVDxrN4uNmAf+U2goacuJnXhnUDuMzad0ApmkQ8PnwLsvSAvpqX6JJb3qD3UzHKx93oOBzL0ip2IVw94aKIloA8g/wYHnAcysNBr3JsZeSvcm+f7YUyfb4e+FGKNr1WOM2LveQk40bFof7R7eOKBs+6di7zAebHYVQ44iPG1PHE/BjMheFNVR30bHudifzp5QwKaH0620PfRRJ/3h39zUiwHnxDMdbEYCpgCMaxM7KGHxITVa760z9rDqqdifXWR5+yM4kPOWNVlsb8eCyjWBFa0e5L/iNvEhrJK7JwMCtNiU6z5YiZjXKUbZqguDEKVSqVS+cP8AvWdtjNO6SDVAAAAAElFTkSuQmCC>

[image11]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACMAAAAYCAYAAABwZEQ3AAABjElEQVR4Xu2Vvy5FQRDGRyERiUYiJEI0ggdQ0dIpKBQKhVruA0jEGxARolEovIFIVLfQihqFglAqRPwp/JnPzLqTObt7XZFo9pd8OXe/b86eOeees0tUKPwN3d7I8M76UC267IH1zHoxemI9sm5Zy43SKqOse9a6D5pQJ2kmBbI1562qXzlviNWuvxG22kx0UmWC0tkuSXblg8BvmznwpnJM6WYuSbKaDwI/baZNj3Mk5wybzIJsz5vMGOWf6BfNmpklqdnWY7i7FMj6ndel/rnzK6Bow5vKCkneqeMeHaeamaRG7jVg6pKgcNObJC84skPnw9t3XqBO6Ubh49PPgqItb1L8CexEPEvsnMApSYa/O0mqIDZxzAuE9+LCB0o4d94HFhTgjj3wTyJeWCPubEDyESBfcH4gdyPfoACLkQf+tRl3qLek41eTgdzFzkiyXh+AGZK9AhOGSbB/2ItPqT9OspRjz8EYi9oRa1DrMM+bZhC2F8xzQ7Iv5ZpsGawzI2Y8zeoz40Kh8O98AkUIglxBFUyrAAAAAElFTkSuQmCC>

[image12]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADcAAAAYCAYAAABeIWWlAAACNUlEQVR4Xu2WTUiUURSGT2oguctqJTK0kFZtqkVRqzYiuUkkIhCC2riUNoGtguwHivKHIIpoYdIicG3rFu6l3BiVrvoho+gPqfPOPbc583Lv94GouPgeeHHuc87cb47zzcwVqaio2A7sZpFgBwuiRXNc08qFAv66dFPtu+anyw9zXzUfNL2N1jQHNJ81t7ng2KmZkfACcryRcOHTmsea6aZqMd8kv3eXhFon+YPmV8nXqUl40QBNueH2298Lkn8B8Avkfmm+kMuB56+wNOak+LrIEBc8RcNFyoYbJvfMfBn4OKDvIheMOECKWMPHIctGDNdH7qb5Q+Qj8a65J/l9Qdlwr1gyGzHcUXKXzZ8nP2p+TPPWHuf2PSah9oD8EfMT5JOg8Q5Lomy4w+Qumb/m3Kw5D9bvyEVeSGN4zknXVwia77Ik1juc/+9iDe+B43c3EgdJAf+UZQo0jrMkyoY7Qe6K+TO2xm2fen7KgV0Saq+5YJySUN/LBQZNkyyJsuEGyWE/+B5bf5Lw8+DB4Lk9b0ioneWC8VDyz20CTVMsibLh/GcLvDQfWdNcdWuAd+WRPea90c/OgxpOKaWg8T5Lomi43xKORh70+q/pJ5ZITUJPPCTMN0p1UEtdD0fAP5Ku/adfwqkAt0rcCMcg/ubC+qM0ehC4EdfTZv6WrffZmoHDcQ+/bcua55rrEo5u2AMsWV8Mrv3e+qPDcO3Wv2VguEUJX/k5zmn2uPWApsOtKyoqKio2nX9wCbuCyKqwhQAAAABJRU5ErkJggg==>

[image13]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEYAAAAYCAYAAABHqosDAAACVElEQVR4Xu2XO2hUQRiFfwUbQYMYgyHRRhQCYicEUoiRgE06QcFCjCIigrWIhWAhWMZXYTohXYrEKiRWPkAsLNQE0vggEUkVxQcaov9hZrL/np0ZWPamcJkPDrv3nH/vPHbu3HtFCoVCob04pOpg0zPBRhP8NbIcVP1Q/TL6qfquWlEtqnrXqzeAk+I6tZ0Dz6zqj2pGXN2qamtdRf3gYsqBc6HmEweehxI/x3Vx/l0OWqXffE9NDPxjEQ/qNt5X1TXVZdUF1Xmv2ICYYXF1+znw5CY3l1VCbmK401Pee2684+Z7AJdBF5sRXkt+cLnB57JKSE3MR2ls+IH33pLP3GCD2OY/c4PDqkY2xoHSJy47x0GVpCYmBvYb1OOySbHMhmFB9UF1VWqT8qiuosa0uLyH/EveP0p+5aCR1B2Hyf3D4Iikc/jvzfEu7+0zniW0xfqm2mPqNgw0toPNCHvF1W7iwID8CpvKFolPWMwLhIlgdovzD3MQ6FTdaUIp0MhONolR1W82idsSHwiA/5S8e1K/giw3xf3mFAeeV5JuqzLQAJZ1irPSuG+M0zHAM06qs/AHI94Z8gJhL0uRWk2VggZSt9YB1Tx5uCPcJw/kOhvzg4fLbNIGkj8XNmNktzioGjSC65bBBofspeqZ6oXqjbiVEXt2yQ2G/cfGw+3Y7nFhD3lnvMABybfTMk9Un6XWCPRF3LNLwGaszaYukOvwkLg9Bu9AuGVfFFeLdzA8OQfwDhTOsya1Pi0Zf269+j8B7zWn2TRgJYyQd4KOC4VCoVBoE/4BNjzA5pRIcRsAAAAASUVORK5CYII=>

[image14]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACsAAAAXCAYAAACS5bYWAAAB00lEQVR4Xu2VvSuHURTHj8QkUpIoi8ngZTFTmGTBYDAZFFZRJHkpBov8Ab8yGMmilH/AYJTFoIik5LW85OV8f/dczj2e33MXk55PfXPu93x/13mebvchysj4H9RYQ2ixBlNrDQF79LNKbcNww/pk7bJyrDdZR+lgvbKmje/BJlZXQcIxwTpktbH2WC9h+5t31r41mSZye+Nhf1GtaoTShsUbOGFtsirCdp41+v1mRhI8/7CFeCbXX7QNTWzYGElDFBtvStY2p1mheOZPhv2wJoW/3ZZ12n7DFM9Ehy1nXUp9FLa/3+Ct8QH8EqlHZZ02yDrFM/nmjDUF9I7VepDc+fXg7CNzrTwP/EazThvkgVx/1TY0CMxaU5i0Brl8p9SxYXE7eHBbpA0be5g8CMxZMwW7KepHtfYk/eMnCh9Ag3zsjs6H5q0poFeX4NlhkwbTXrf8xRm35x7gStyQulU3LNh0wZpMGbleg/HhHag1PgB22Hrj4Qh4bBbkVI0PR0Hw4yVrCvaWKCKXxxvyDImn2TLeOGuH3BfqTvkeZHtYY1IH9LEu6Oe7DOHcnekQc89aJjdkF7lcb5BwnLMGpG4nl2v+aVOleF5Vqgd07zRsZWRkZHwBTQSTuk6vXk4AAAAASUVORK5CYII=>

[image15]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABkAAAAWCAYAAAA1vze2AAAA00lEQVR4XmNgGAXDEcwG4otA7A3EklD6ChA3IyuiFKwG4v9o+CSKCgIApIEQWAbEe4D4OhDvB2IOVGnCgBhLFgKxO7ogKYAYSxYwICxRRBInGhBjyTwg3gTEb4G4kAGi5yCKCgKAGEumAfF2JD4bA0QfyHIUgJ46COFTEG04AUwdUYAYhSAXq6OJwSyRRxPHCghZEsKA3dXYxHACQgqlgfg7EJugiVPVEhDIAOKXQCwKxMIMED23UVQQAMRYAgI8QLwWiM8DsTGaHEHwA11gFAwPAAAQyT5f1sNIgAAAAABJRU5ErkJggg==>

[image16]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADwAAAAYCAYAAACmwZ5SAAACa0lEQVR4Xu2WTYiPURTGj6Qkk80UGzULitTsJuSjJLKwsJKwncb3QpKyUGRrNT5WEhuynBQLWciKlJqpQSk0s0TIV77OM/fc3vM+c+/9vwqr+6un7nnOee973/f+3/s/IpVKpdJmNRvECtVSNgt8VP0yXabce9Vn1RenTxKueaM61JT+fcZV31VHJSxubzste8w/odovYXFzWhV5zkq4dhknDOROkTfPfKgzXYoXSqjb4jy+0WmKwa6El+OH5GvXST53SULuBSdy5CbypBZzTLXBxfwCIvCG2UyAuik2jTuSnhs8k5A7wokcuYk8qMHPs0TpgfEdluiX8ovJzQ1KuSS9ikck1DxQXVc9t/icLzLvK3kgtaBV5l1T3VCNWZwjNUef6qVqkvye8ETMRQk12OE1zoc3SjFOToYXi3MA8aDzuMazXpo866Sr60zuRpHbkl7QT/K6PjDG0y6OHnYrxT0J+SWcUCak8LlgN1LCZOxB8S/nuNXctTiCUxH+QYsxxuHG+AdeZOMFTXoGeAPkRfiFMcidZ7NEaTIwJKHmFvnxgQ9YnFsYvLijhy1mUh7Ad4rcU04YaG6Q38mJErmbeVCDn4/nnfnYNYBvnOeab95Wi8+ovjXpGfx/9U1pNyo4GJHb7TzPW5l9z550ueCJzK5DjJYvEhc+13kXzIugi0rNc9WNPYjZizyWkFvMiV7kJmRQ98jGDy1mcPB8cDFqNrk4ettV+2y8UfVKQmu5w2quSNPsQOiZ0ZS8ltBHR3+t1f8RqYXn2Cahs8G3mAMPcF/Cy/G77dks7S4Nf3crXfxPWc5GpVKpVP4jvwHO69N9K6cWfwAAAABJRU5ErkJggg==>

[image17]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADwAAAAYCAYAAACmwZ5SAAACNElEQVR4Xu2WPWgVQRSFrxEl+EMaJWkECwWDYB0xCkGSykIQRLQTFAW1DlgIim2qRGwSQmwUS6vYpBDBQsRG0IQEVNRUKipqBH/uydxh7ztvdnnxbQiE+eDA3HNmd2Z/ZndEMplMJrBBdYjNBPtUu9is4Jvqr2mCsi+qH6qfTt8lHPNRdanoWh9bVX9UL1VvJExspqFH4IyEbFh1UcLkcJNa4aaEY/dwYCC7Rt4m86FawQk3u3qveX6g61SDUwmvjN9S3hdvVVl2W0K2wMH/clKaLw5EbxvVDLxzbCZAv3dsGtOSPjeYlZBd4aAdcEK8zuxBO6lm4GEdVrFDqm9M2blBVVYrPBDaS66OcD+w37w7qnuqB1aXkTrHdtVraX4Qq8JdCRMYdx5qfDkZnuyg1Qecx308/VLkrKuu36qCwZ4nvFYuGO33ro4enlaKGQl5DwfKC6lYLlgnoytQGfg9dbApYVL40jL+grusvaWIl4G3m7wI3zAG2RibdfFZddbV56W482UT80/0stVMygNYp8hecWBgc4Mcf5LamVMdJW/RtbHJ4Il3mjdk9Q3VryJexv+r70vjRmXEstPO83yS5jFr4aHqg+qJ6rHqqWpeGgeLE9/ovFvmRbCL4gminnJtD2r2Is8kZN0ctMsJKQZOyYMPz1dXIx9wdfSOqS5Y+4iE7Sq2lsetz6QUOy8Ie2ZsSt5K2EdH/6D1X1NwAY8kvAX+aXuwNA67uk/V6+pMJpPJrFv+AY9Dv99V/zSgAAAAAElFTkSuQmCC>

[image18]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADMAAAAYCAYAAABXysXfAAACIElEQVR4Xu2WO0gdQRSGj8FIEA3BgJUPAqKVQQghAcWUPhpDSCNWapdKQQsbxcpYhBRigppKxEKshbRBkSiIKdRKwUdhEx/YKL5yzj0z1zNnz+VeiVxB9oMfd74zO7uzO3dHgJiYmP/lrRZpqNDCkaNFNtnCnGB+Y64xh2E5JfvA/SmX4viJ7JRN6OJVhqOkgybzE3OAWcLkheXs429cLo1T5/qFs9jDPNbyPqHldaYcLTuazBflNbuQ2WQKtEDytUCeaZGCTK6ZxL+tSl1QbAMPPAfc/1tYTvAK8xm4/hQzgDnCrDtHE2h2x1Pub3vizCgjwKtmEHMF9kMKqAEe8FgXDDYh/G3Nqzbh2/4BvVO1v8APw3PhvOYDhJ6O10TbhDrRgJnQqgXw+X/ccS5mEVPk/Jjv5CBHy1w7azL0kZGe3m6ZaEdYxrzU8pZYN7NquI/OlQj3yLkh4TyNcDM2ZTYsh0wCP0VPIaZbtDV1wIP+UN6ajOVoiWg3YTgJ7V99YI+XpAeiM+3CtCgnGQYecEZ560LUHjec1c87+m+E7sF72gY89c5FaAD+MtDuvwC8+W0Ad6Y179EXf47ZEW3iDXCfDuXJFRtu1HAD7th/gGqdlw/jO/BvJoK/SSsSy01jVoD3jBfAdfll8ujzCHL0qdauF/htNyn/FVON+YQ5F7U7pRx4Ar8wparmsfar91o42sDeP15jOoHffkxMTMwD5R+yrJ2hspnRngAAAABJRU5ErkJggg==>

[image19]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACfklEQVR4Xu2Wy6tOURjGHxQyQC6nKCZuA0wMZKRTIoUyISWdYyAp5T8wkGKipJyJ2wADMyIDBhIDEyNlYOC4KyLK/f4+veu13v2etXacz8Bg/+pp7/U8a639rW+vywY6Ojr+Vy5GwzFdtDyajq+in0lHQ1biveil6FnSC9GTlC0SPRcNi+6JHqa8J45Bf1xkMtQ/Ldqd7tc1amQOQvO5MWiB9c9GM2F/2IwYjIaPKA+w5m2KJvIP+lMGUa+/AprZW+2Jp6K7GPmwNQWPvBPtjya07oNotnAf5f7JVWi2PQZ/ywbRCZQHOCd5H0TjnB/rkflQfyD4Y0LZ0/bG2zIyPpT7Qvk31klpgOQL8sO4xr6J1jZqKFxHvv1j0RnoZjHW+Z62QbRln9DMF4su5TjD+T0l3dcGSLjLWYcXQmb4B25L18vJW5bKnkHkNjVxB42cFF0XrRcdgtb73KiR4Fvg1DRqA5wJ9Xema61D+sOivdBpT85Bj48SfMNsszQGCWZbolngezTIajQHR0oDPACdkp59yAM1djmPuuGyGrEPDzewWmZME72NpmGDaRPhdSDde+y8M3gI+zLvV7lyZAm0zqsYJGzd15iH8vRtxQ/Me6Vt2g5/g/c/XPk28tfMEPI6N7iO2YYfDiWY1b6GuJ7jDNkj2hi8EZQGeBPl9XYezV2L7Q67Mj/D7Gsn9knsWaUjxNZ8adtfAM34B22Ftj+SvCqz0Nwl+Sa4ARhcg9yWF0I7vCV67XLCdv4t7RCdgh4T/c6/At0U7Fmcio9Sdlz0xmWsZ5lBf5Jodro3TfWVRgPX0zXRHdHKkJHSetssmhjNHpkQytzcOjo6Ojr+Kb8A5VbTuhxhcIYAAAAASUVORK5CYII=>

[image20]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAHIAAAAYCAYAAAAmsqlBAAADfklEQVR4Xu2YS6hOURiGv9yvCUdIOEiZkYnLQEkGKFFEBhgwIkYnRgYGBgwkkonOKVIUSSkxMSDkmltJkUu5JdFxz2W91lr/v/Z71l57bc6/nVPrqbd/7/f79t7r39/ae6+1RBKJRCKRSDSa/kpz2CxgMBuKEWxUSDMbAcawYRjChodfjlz6KX1U+uLos/HeK71U2lrLbgC40COlhUoXlVqy4VzcP/TT/D7MZFTDKNHtv8CBAMekY9uhsW5SDr2lfpyP9aLjJ8hfY/xz5BfSzoYHnHivxztNng/k3VB6rXRc4npzZzJMaZDZfiPlCnlU6bLSK6VbSquz4SB4mvHfl3PA8E06Pq2WTaJjeHqjiS3kZPLwhOY1xCUmpyrKFvKI0jw2Izko4f9un24fJyU/lktsIfGqcLlv/CJicqqibCEPS/lC2jdOqFB40hC7zQHFcNGxXRwooqiQPcTfIHwn4ePVFQI5o5Wem230tv9F2UIeEl1I+3T9yIYz4LwYrGyRehHvZTLqbBcdX0L+CuO/ID+KokI2ib+Q50X7M8hnkHPG2d+vdMrZrxIUEh0wllbR7bffKtup+9QyNPjM4JtnsXnLHM/lq9SL7eq70hQnrxRFhcRUIVTI+eQz29gQfdxINisAhbzEZoBZSgPIQ9tROPYGerw8bOEYTO9sQXPZlyMcxB60WB/2B99Fr4j2S42sDDjuLZsFcPtCwnfGBwqJdv8LXARsYx7oslbypx2LRB+zmwOGNvHf70La2fCAEw8l74nxQ6A3+3LgfWKzAlDIq2wGQDsfeDyop7PPbx3cmz3kWW6KPiZvbPFO/PeskNhCzvV4RRdcIDqnF/nwDpBXBSjkNTYDoJ3PPJ77v7HN30x49q2Aorrw8S59JRwPEltITOrZ2+nxHpPHHQBLdtzQccabSX5ng0Ly/7D4biAGJQxyMIq14BWK9lt2SP08G5VmOzGAmG/0a6cd0DSKRRFTyGbRF1hp9nHDP9SidZBz1+NtMNtLzT43dLzxGzGaxaAKUx+7gAFhffOpm+TEXFAITAVwk5uUzoqeP7tMEH3+SaJHxHilYgQ7XbLfSVzPXgPCSheedsh6WEH6a2IKaWkVvV7ZRn4RmFvdEd2Tp1LMBQvGXQ28QbC0eF10YX0gZx15q2i/4aBHdQU2K01kM9H94CF8opuC1ZBEIpFIJKrkN2lKEaI5UiGYAAAAAElFTkSuQmCC>

[image21]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADcAAAAYCAYAAABeIWWlAAACJUlEQVR4Xu2Wv0uWURTHT1Ih1dQPaJAQJ6e2lhpbGnJqkAiCovwDxLGtoVoK+0UQOrToJLha0BRIU1tBVFiKgRUlioVona/3XN/zfj33fcxCHJ4PfHmf+z3f5z738N7nfa9ITU3NTuAgGwG72CBQP8VmBb+djlFtUfXTacm8edWc6kwjGtOt+qa6zQXHHtWopAVE7Fetqt6oPkrKPW9KtGZBynN3SKodIv+4+T/IX6NT0qIBQqXmuuzzipQXAH9v4JXyDHIzbBpPpTxPfsZFLnhaNZepao5r2TtAPoPXAbmrXDCiuTO51sYFz/9oDluSPegI+Zm8a+5KeV5Q1dxrNpl/bS6itKhrkvwbqim7jnLgpKTaY/JPmH+f/BAE77BJbKW5IfLGzfdgjB+hiGfSaJ512uVagvAgm8TfNPfKxOD+gcC7RF4mNxIBf4TNCATvsUlstjn8JTxgU9K2j+6PPLBPUo3f5cxZSfXSO70OQtGCPJtp7rvqshv3qY7a9VfVL1cDvVKe85ak2nkuGNjypXubQOghm0RVc29l43vw2V2vqK67McC3MmzXPDfy7HlQwymlEgQfsUm0am5CNauaVL2wz3fSnH9iynRKqudDwstGaQ3UoufhiLcscW2dHkmnAmyVPBGOQfzLhfEXaWQgeP0u42ssD8Y47uG/bVo1prqp+qDabZn3lsvCsz9ZPntort3yO4oLqsNufE7S2bSmpqamZtv4A/TQucRmvRD2AAAAAElFTkSuQmCC>

[image22]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADcAAAAYCAYAAABeIWWlAAACcElEQVR4Xu2WS6hOURTHFyGvgccV5XFvRgYo5ZYYmiiUUiSlDAyUMjFUBspjQtcrJQYiMyWMmFLMReTNxDMiJLH+317Lt87/rv3dAcng/Opfe/3/6+zd/s53ztkiLS0t/wPT2CBmqZaySYxSrVBN5qAHP4PmUfZZ9TXoi3kfVa9Vq7qtOQtU71SHODCmSln4nGqdjRc2OgoPVHdUO1XnVU8aaW8+SZk3Y46UbDr5i83/QH6HAdVYG6Mp25xvrD94o81bGTzU8CNbVD/Iq4HrX7JpXJP6xv2OY60qtc3dkHxieC+oXhRqsMT8kcDjgL5tHBi+gQzP+IdtUNtcbWL2vV4dvFuqo6Fm/F9zRPI1HF4rAv8um8yfbm5X8N6rxkt5BjN2S+nbr3pq42wNsFxKdor8QfOPkZ+CxsNsSn3hzN8RfGhcM+5wSYZfh/oZec51ac4ZFZ/5nqB5iE3JNwEy/5XqqpTXtOezGx3Fw11mbyt5TraOA/8CmxlozJ6P2uTsYzwh1O5B/rDjb1+bK2OilOweB8YaKfkMDhg0HWdThm/CYT/r8U/Gdqvfqr514w4bJb8WHJSSbeLAOC31axug6QSbyj7JJ4B3k+oM+P6JwDdvb8gA7soZG/Mc6GcvggynlBFB40k2DWQbQt1vXjwqoZ4SajDJfOesyRmQks+3+nY36oAs2xyOeN8lz36zVsqpAH8VnwjHIH5z7bFsptUYX+nGHfAygu8vhmVW44eIwMNxD982HAIuqg6oHqvGWM8j63O9UT23fvewOXxu/go4RVxW3VfNpczBncKZ8qGU56HGZlVfqNdLubalpaWl5Z/xCxun0QDAbMM6AAAAAElFTkSuQmCC>

[image23]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEYAAAAYCAYAAABHqosDAAACbUlEQVR4Xu2XzYtPURjHv0ikkLxNGSsRJStKKUWZKaWmZoqyUKTwF7CwwcLSAit2VpaysrBhaWFlFsprQ5beSV7O13Oeuc/9zr13MHdS0/nU05zn+zznPud35txzzgUKhUJhbrE52XoVheXJdqk4DT+DRbYm+5TsS7DPyT4me5tsItngZPYscBA2qGUayByGxU8nOwkb4LxahvnMuZdsf7InyUZqGe0sgfV9qYHMNUydNHIWpl/RwEzZGdptE3MOUwd1qEGjT12186I1cQCWu1EDmabV5HTFeqFtYtoKUzsuvtLWV3mI7ryu53TFeuFfJuZdbs/PvtLW11ma/3blcVUzdl0DiS2w2DEN9EnXxHxVEfUfszu0I+zXpD9O9jzZGVTPuVHLqLgDi68T/VTW94jeOyzCE0Wh/kFF1CdmNLQj7Ed9UdDoPwv+6qxtCFrE66i9x/QnZC+w2AoV8WcTMxbaEZ+YVdlfmH2lSXNincgATN+hAYdFL/+FtcEiK1WE6d9VRH3Aa0M7wn5RZ/t+8MlV1FdQ5AKsj552zgM01+0VFuCyVtr+Y9Reia9oX7b3Bt+1I6I539D8XEefPyuwwBoVYZc5Lb44a0NBo089Qo39o6+4xtfsVgyg+4dzM2bsogb6hkX43ip+mVsQNC5/HTB96qrxVh39yO2g8TiOe5zvIY+C5mxC96TNmLvJXqMqQnuT7EVMgh2tPAUc5ukxOZz17dnnZ8TTKvybfbA9ht9APLJPwPpsQ3UnIvwG8vH8QDUmvrquj09m/2d4V+F3EDe8uHoifB1uwr6TLknM4Uo4KtqY+IVCoVAozBF+AZIJz3MrAdXtAAAAAElFTkSuQmCC>

[image24]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC4AAAAYCAYAAACFms+HAAAB+UlEQVR4Xu2WO2gUYRSFL6hBEUmiRdBCk04bY2UlpLRKEEll4aNSIZWVlYUgJhCREEws8rCw1tYqbaogIhaCWvhAi5CoiY15mHP47yR3z+7M7Bo754MDc8+5+/93Zndm1qyi4v+kQw1nP/QnqIwuaBX6Bn2GvkKLUL/nPP4EvYPee8+QZy1xCvoO3ddAaHbwjLuW+o9pAC5ZypY0aIZuaJ8fc5FmBp9VswD288o34pelvEeDVikb/JqlnhMaFMD+22o6rX57uZQNzt+hbnRA6shxS/17NbD0ubLB26U+KPU2ZYPHjY5CG9Cd4ClPLD+7Zyl7rIHtnDCV3bBvocPbHQIbh9UMMJ/245ng5Q2XZUXi00ehz5t6EPrp9a2aDoENI2o6NyzlvBrxZluDpkIdaeakyrgMTaipcKFRNR29Untq4zpuWup7poHD7LWawqQaeXCxB2o6zF75cZ/XRfAFw57TGjjMLqoZeAGdVzMPLvZQTUsvJ2YXghcHb3QS2TfTCL498zLyEjop3qbUNXCxMTXBU6vfKKsPQY9iANqsePA3lp/NW/qbMA6dgc5BH6DnsYkMQF+g37az2Qr0MfQ0GoJ1L7QePD7ilj3LFF/nXDPu8wNaCPkVaM6P+dTK+jj4X3HW6h/+R6Cr4u2W61J3Q53iVVRUVPxjtgBR3pjuqmWYeAAAAABJRU5ErkJggg==>

[image25]: <data:image/png;base64,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>

[image26]: <data:image/png;base64,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>

[image27]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADEAAAAYCAYAAABTPxXiAAACIElEQVR4Xu2WvWsVQRTFT8RKET9AIhZBwUKEJFgYIjYhhHTiB0mEYJEihV0ICFrof2BEEEEJpBMFi4j+AaKSMlUSUqYT0ogGDEGj5p7MzNu7d++8fZbC/uDwds49szM7u7P7gIaGBo8R0XVrdsh5a2Q4LPob9dnUPF4iZDdEb0RbsX1AhzSLKAbwdLCIVtC5XXXsMYZQu2oLhg+i39ZEsRCvbYGw8Eg0K7ojmo76JppQOQ/2/YiQ/STqL5dLfEH+AhNpIXI8Q6hv2oLXaVi0Yk0Hr28OZv9Y09DuTpITyGRuWQNOKEOnOcLsE+PxEUlcRGaChk4y+CE6Zs0MPFmXaDkeb4iOlBIFrHMlyVnRT9FD0fFWon6Cp1Gf2cfbVDl4sm3VHoiehRea/Cuix/GYHjd8om6CD1Ads8IQ/Mcrxz1rIAxy33gzCPthXHRX+XbBOHa7i+DkWT9j/BLtTtApa6ieJ60wxTrvTI550VtrRtj/tjU1p1AdvA7mbxpvNfoa3c69Ri+oY69OduLvUYT9UWEO+c45mJ8yHm/5L9XuRfm8C6advi3XUCyIN49zKF4aT0UnVa3Fd/idE+lx0LwzbcKMXiV+YXW/JdXuUT75ivCBfY6wfzQvRKOiSVTn0YK3KluEfxHrolcIf00uIbw2KY3tdzm2+4xPUpay3xSOpesu/DPHvw7/yhBCv/eiG+XSPoOiQ8brRs0GbWhoaPi/2QNqLp0z+MzlLQAAAABJRU5ErkJggg==>

[image28]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADcAAAAYCAYAAABeIWWlAAACX0lEQVR4Xu2WuWsWURTFbyQBFwzBaBAMEhJEbVIINjYWEmJhQAgiIggWYi+WNiK4FO4LgmiRRrAIpJJECwNxwX9AK8WtiglEXBHRe/Luc+6c7735PhHBYn5w+Oade743c2d9IjU1Nf8Dq9ggNqgG2Kxgko0MP53WU+2T6qvTF/M+qGZVO4tomk2qedU5LhijEnZ8RnXNtodLiUb6JeRa5aPk870Sat3kD5q/QP4ifaoO20Yo1dxGCbVvzvth3hHneXCFcYZzB5sC2XdsGvckP1e84ge44Mk11yXFBJHPNj7mPA9qu+23FfA4IHuICwbv3xNrS7jgyTWXIk64jAvKY9Vmaa25eNdckupss+aescn8aXMn2ZRwCz+x7armcMVRO6V6Zdu57DYJtRvkbzX/CvlJEDzPpmNIioPooVrEH2CuuQlp9DF+TV7kvhT7Ze1wuUoQvshmBmTvkjcj4a0byTUH72jCO0heJDaSAv5tNlMgeJnNDMelvNM2KTcGUs3htmcPpDywXELtOReMXRLqa7jAIHSVTWW76oGU30ZbpNzcHjfOCcxJ+ZMC9kq+OXxXUdvHBeOm5P9bAiF8oJl4cHecF69K1cQXpLGO7+MJ8nBVbtl2Ks+eBzWsUpqC4HU2pdjBOudNmzflPCb1eh8zRfokZLCaAU+L0iK5E4jH4Luka78ZkbAqwK0SJ8IyyL+5Vpp/VtWpWmvjRy7jwev9vRTzYT3o54OH5xPNv1WNq06rXqraLfPCclGY743lo4fmllr+rzmseijh7DZ9gJuwX7XajbF2XeHGNTU1NTX/nF97uMe9Q5MFkQAAAABJRU5ErkJggg==>

[image29]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADkAAAAYCAYAAABA6FUWAAACfUlEQVR4Xu2WS6hPURTGl8dFt6SbZ25RSFEooZSZiNkVUUqZy0QMmDIwxUSSGBADBihSUh7dkZJC3hOSV4lCymN9d6917zqfdY6r/il1fvXV2d+3z/qffc5+/EVaWlr+J2axYUxXLWazQ9xVvVM9VT1XvVKtt+y8tZ+YXqs2WwbeWv7SrrtDJvNVPxPtjp2UieafUK1TfVHNrfToDONl6BnwQiNjQjaCslHm31FNpUzmSRk53g7e4JJqPIAPsJd8eMvI6wQ+kIymbAsbDr7GDTaJfskLw7vPZgeoG8gpqc/2sxGZI38eZF3hzO+iNuCpBbJ+TlYXNd7UZOAjG5HZUga5SfVV9ULVU+lRX5j9s1Jqfbf2PdU1KXV/mHdO9V71SMq948yPcF2ATWVkTYZ1OJa8CjOl3LTG2gutvXKwR14YsI+1vSj4eCjgaxpfYrl5AN6F0HbwQpDh2QB2+pN2/cEyZ4LqaminYNqsIs9/xOHBONHHbrxadcg8zBBnhnkHggfgXSEPPJOS+YuOv32b2t/C9V9xS0qh7dYeziCbvIeJt8O8bJqdlpLtkfL1I4ctAw+keW0Pkj3UTfP8rMz6gMxH+0jiZf3Yc/ZJyY6rjlYj2Sklm6a6RFkt2Y/h34QXAtieuQ+Adznx/L7oYRqzd8yu8VIjW6XkWH9Mn+TP3Ag6j068z4mHfzrsTUo8Bl62Y/tU4+1/hdQPxP+hbeOgiSlSbloqZTfE4HAE+M7o7JXSb7K1D6rODMUDrJV8I8geFh7O6E/y+7rETEC+gHyAD5LVGxZYR49VGzgI4Ci4KOWM20UZwIGdbQQYDINaG9kM4Myt4zobLS0tLS3/kl/PhsqfJ0f+lQAAAABJRU5ErkJggg==>

[image30]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAC0AAAAYCAYAAABurXSEAAACH0lEQVR4Xu2WPUiWURiGnxL6gaKhoaWIbBGqIRoKF50SakxQHNItaiuIlsAgaKhJ0KVBh5papKFAXAIRf0CEhojIoR+itB+tJfp/bp/n+D3f7XntVZriveDme899P8dzvuN5z/lEKir+f06wEdjHhrKVDeeU6rPqreq1f35QHfT8veqV6rlqzmvaPCsNOn5RTap+qxbr42XgJ/30z691FasZFavbxYFyUyyb5aAM6NiU8SD2JlSfxCZTBvQZY9NJYzRwUIbUeXPwsOrwbgSPv0QZ0KeVTSe3MKXB3vtG3lOxPzgQvPUO0CnFfY6JZfMcBHhL7aT2KtIqHCEPL94Tf+4OWY5nUjzph2LZBQ6UvVIb/5J7j1V7VioyHBbrgC0SgYc3PoGX8WVoM2ngtZQD/nXVGdWSt6/WVWRAESbE9FK7Xay2i/zE3yZWlEWwxQbZZKZUR9lcg6LB74r5lzlwkN1hk+hjI8eQajq0t6mu+HM6bw/U4mWKJp383HG2RSxr5CDwQHWaTeaiapi886oOf06XSXwxcUPCexO8RNGXAdekOAMzqkPk/aC2nFT9Etsa42KrnU6I7V7T64rcF6vZQf5x93EJ5fguxZPG+O9U/WLbtFnsFBqJRSCtSk4RXO23VJtULWJ5T8hxOeHMj/0XQo7fF1ixlH0U2/uJs6pH/nxbanUvUsFGwUmB/8g9qb9B/wXnqL1ftZu8ioqKig3yB+ZIpjdKG6PTAAAAAElFTkSuQmCC>

[image31]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACaklEQVR4Xu2Wy6tOURjGH8p14hIGkhSSMDEyPGXokgwkkmNgoJS/wGVg4JJIMRITBsyUKExkIIMzPmXguCsSCrnk8j69a9nvfqy9v6/vnIHB/tXTt9fzvHvtvda39tob6Ojo+B+Zo0ZikRrGdDWMH6bfSeckK/HZ9Nb0KumN6UXKVphem8ZMj0xPUz4QK03vTcc1SOSbpn6m3y+1iopj8HyxBi2w/oqaiXzdeRr0wxLTlHTMTtoG+AA+CXckU/IN9cswmuvXwbP8r46LXgPsF9Y+UbOFx2jun5PJbI8GgzARA1wGr90t/iRpR9r+8baMTJX2AmnX6DXAaabRdLyrHv+Fz1G8oeemy/DNYnLwI22DaMu+op6vMt2o4n/pNUDuYplf8N1NiRfMk3AzeWtTOzKM6pwmcQdVLprumTaaTsHrvtUqCrDopJqJw9LeAa/fKj69MdMh06bkXYW/PkrwH+Y5azRIMNuuZgHu7D1hZ5yNfuByyzOc2Rc86n7ImtA+IkfRnGXmmj6q2QQ7O62mcReexfcaNw29Ob6EY5vH60NbWQ2veadB4jvaB7gU5eXbCDs7oyb8eWPGhziTd0tu8Rm2WZsZQfU1c940K2TkOvyc/eJnmDV9DfF51hVywLRFvBrs8KyaxhHTQfFuwetnBE8niJ9hG0Km5BVQeoXMh2elbX85POME7YSfz/suXQObTS9RLQfqk+lZLIKv8xPwzobgdew8Qi/+S3tNl+CviaHg30b1uUfx2vl6F0wfQsY6vRf6M00L03HW7Fg0CBzQQ9M1lGe99LxtQ/mjfDzwfRzh5tbR0dHRMaH8AbMev9hu6UV6AAAAAElFTkSuQmCC>

[image32]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAXCAYAAAARIY8tAAABJklEQVR4Xu2SvUoDQRSFjz9Y2qWx8A0kWNhJiJWIiJBOUkgeQVvLtCkDaVNFfQLB2p9KJG3KBAKijQTBQtFznRm83L27wc5iP/jIcM7snckmQMl/5sEGilv6RW9ojz7TKV3Um4pYRhjgIfmbDckFQte1hWWJzuAfIJmXJ64R+jtbaN7pArKD5GDJhibXbGPOJU7oaVzbTVsxOze5poKCA+TWcvuEt2neN9hFwQGvCD9uwtuU+3DkEqF/tEWTtk3mDTqCnyfSBVZ0eEg7OojkDZK8b0OyhtBt2GIUiyKFavw8Vpnmio7jep2uqi5DDdkhDbof17YTJJM/inCvC486/CEv9IAO6I7pPugebcF/9odN+oTfV/NJJ6rXr+1M5bbLPaCk5O98A4jAVtulFJIxAAAAAElFTkSuQmCC>

[image33]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAXCAYAAAARIY8tAAABSUlEQVR4Xu2TSytFURiGP8TQ5DAxcCmdkcvIzMSEhNKZKZKSn2DiX0iJCQZuP0DK2CUj+QvChImBMiDet/Vtfevba1FGBvupp7PW+67LObt9RCr+G01wHA74wnEBP+E53IRP8BE220WeYwmLVuAq/Ijrb3jwqw/BkYRuwxdkH867rCZhg4Vzn1nOJPSXvmC44zI+LntYi85vTeYZlcyXeNBw22Tr8NrMRySsOTSZp1MyF3SYgnLhS7Qi8NsvmJDMBaRP4kt6ozaQ3azwRWF/4wtyAu8lPJbioOlohcic5jmKfW2pYtZlb5oPupzZrstIl6TXS78WKZjv6XhYPxc195zCOx13w/ai6JH0BsI/27KOG3BKx6n1zPhqkytbEJYLPpTyQc9wBh7AMde9w0m4JOV9MqThFmyFdZ3ztbMwK1z7oStdUFHxd74AEDReAFgguVMAAAAASUVORK5CYII=>

[image34]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACEAAAAXCAYAAACFxybfAAABbklEQVR4Xu2UvyuFYRTHT5IFy1UWGWWx6O42PxNFdDcZiEwyMRhktxoMBmUwkix2pRQlm/8BkQnn3HMO5/3e57m6+/upb73P53ve9z73vvd9iUpK0sxyllEm6EPRhDnON+eDc8y5srX4JOekA7k46LF3hkn9IBak/gulIMUOZ4mzyJkn3fELZwzmrjlPnENOe+icddK5fiwCyc03CNJbdAMuNYckPwDwma0oJ+LCSF0o5ZBWNnGGRWSI04mS9MQBzqsdXxbrOq1sYgWLSO4i4k/D+shcZM9cB/jIvxsdp/xA4R4aMtuVcPvgnEnS/haLiAzco2yCzD+Ck6cm90XEv6OMyMso9y26KX1hcc9hXQ2+J3hHvD/ao7FwDkiHVrFgpkm7NvDidmEtbHNOgncuwvFdOP7ljfQiNSyMEVj3UuOvI+sN+rv3yBTpy1De0pvQ1fkkPXEGC0O6NTtesDX+wR7MeyJ+uz2VYl1SUlLkB/BacUTn+c1tAAAAAElFTkSuQmCC>

[image35]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACsAAAAXCAYAAACS5bYWAAAB/0lEQVR4Xu2WvUtdQRDFB0QtUkgwiBBNF1DEIIJVSLQRixAQFEEIFhZKYqumTQgBhTQiWKgoiIVVxEYQ/AtEwcbOIqAgKATzBX6gZubtzH2zx33YWMn9weHtOTO79973sfuIcnIeBj2sxxg6WlhdGAK1FNapwALwk3XDWmctsi7V38kChcYB1jFrKy5TPYX6JOs16zerKeoIjLJ2WG2sDdZ5XM64Ym1iyDRTuI48bBIproHHJxRfnchanZ/SzDOUyFLre84o1L9gQT4CnPiK1QcZ9giSybvoPfaVQfZRPfZ5JqhETzIEOijdg3NlfO284XtW1afWMwapRI+F3awDHe9HHUSfNUf8gvYOnhbLGZKX6/i9+tR6xjQleupcuOxy8UfOz2iG+AVrdHxSLGdI3gg+tZ7xh0L9mw9faogTZzV7rv4+blZ2B0O+56n1jNQ9FUgVxjSzH0+/egTnyviv80Zq7j+KH8Aj/ck9Gi8ojGu2rb5BPYJz0Rs+69RX+Y7vudyoYi3pWA6gCNmc8QK2dQy7DHsEyVaclwMA+55BhlsdsujGcm8R9hHbr1WQ0wkXEv82kT1y/p1mnu+QjVA4gOSE+uVyQ3rfsD7o+Ba7rAtWJesThaanvoH5qrmdYnOs+WI545DVq+N2CnNeFMuF/x2SmZ64muBrP+JSTk5Ozn8xAK3gJtM5tAAAAABJRU5ErkJggg==>

[image36]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAG0AAAAYCAYAAADwF3MkAAAEZklEQVR4Xu2ZWajvUxTHV2ahcM3TdeNGlAfhQYaDIoVwvRjuRREJeeDBnCFTkkikdK+pJPGCUrx4QcYMEXEND4oHY4aM63P3Xues+/3v/XPk6E6/T63Ob33X/u393/v3++299j5mIyMjIyPrABursK5wsAoNdnLbX8V/4K9qf4j+hNvPbr8k+8ntR7dv3F53O3y69DBPq7C2857bx24XWhlcBk+ZZyW2zO1kK4O9MBcYYFcr996pgUo8VOVDK/q2GmjQun+umO92sYqrkq+sdPiYpOkgxgPbOWmAdqBoLR6w4UHV9jJDsYCX4VAV5xDaZ5xWG16z8qPOSpoO1MviB2jvqthA68scZSV2twYqxN5WUejVPRdsaqX+CzSwOsGUx4/8NWm9Qe/pwSb1L2XeyYHEi1biW2vAedxKbAMNJLZye0nFBluo0GBzFZxrbLiPs6H1+7dUQZjNkjAND4sfmbOx3sNp6eu5fev2ituNbn9aKbMoF0q06oCnrOjUMQQPrPXAA+rgxbi+Xt+xcngFz7m96XaT23du57gttZnflo0vL+BhfG+lr5+7La5lNkplSL6uqPqOVbuv+odFoQoP8ne3T90us/a4rMRvVgq9rwHrD2xLx384+adXrYcOStiz1n5DlV7d3EssvnbYrGoZfBIe4IXTPnH9WfKDDW2yr3ov2W88QHReCjil+rdVH46r2lTSHnG7KPldPrBy8x5J0x8TqL63+LCgoQUkP8Ru14Bzg5VY68sIpmx4LWy1m7WYVYJYv45PGv7ZyQ9iBslom5GF8+Wik9AF+FPik8FntP5BonG2ANlXVOf62uQD00Yv1WcqaNUbaP1KL/a8ldhuosfWA2LKYkCHaLVxv7X7inaLaIDOFkm1gGkTny+ar/NJt1NTfIK7rMyfmXiLPql+b/BU51pPJtB6a47erwzF6egLKlZ69z1qM/oX6brHsdYuE+OT+3pG1TSZ2a7qS0T/Ml2fZO12muxl7Q6G9kz1b62+gsbaA8zdvTLAPurMpEdn3kpaZqmVeLw4CtsAHaCg1SdAi1OZ2LgP8arbg8n/uv5tTY3Lk5b7emXVWS+De912SP4+NlnfIBSmQdUwzSBPTH5oOk9n+NxD081pbLjz+hGwYBNrncwE2laG4zGN8wLmPrFOaRm42kr2BsT3rdeHuD1Ur0kQ9N4YM8h9pX9aVscCtAwwzV+iItxj5YYjqx8nJHtOlyhEYrBN9UkAHpsJr+Dyqh9hJXU+yMo9nGnGVHaAzbSBcdbIVMHax5scOvujIW5WQWC9XO62n9sP1RTa46uZ73aVlXYvTXG+EhKh62wySeA++kr9nJWybdC+Buj8FpK71sMBxuENt12snLdyTRLXhfR1mdtHNjwYfFUczDK16DoYMOWeJtrR4v9XZrNRht3dTnBbX/QMR3Pnu22vgQrnrGTFLbSvJD69vvKynquiwJJxnk0eF64VxLo0sgYRCcHIGgL7oH/7/7yRVQwJwMjIyMj/yN9lCWVN7Ebc+gAAAABJRU5ErkJggg==>

[image37]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACh0lEQVR4Xu2WS6hOYRSGl7uUSEgGSm7JJRnJSBkYIEYnJTkGBopQxkYGSiTFSEQMlImSAUkSOi5lpgyc4xJFQlLurPestfzrf8+3939OKIP91Nu/1/uub//78u1vb5GGhob/jWVsFJimmslm4qvqp+soZSU+ql6rXrheqZ57Nk/1UtWreqR64vmQeaz6oOoRO7C37XE/O8Syi6rvvr2yraPFfrF8Bgc1oP8sm05csMkcDAYMxJViDwrOq06lGvSJ9RwiH/D4TnRLdT9mFbK4q0MmDmZY8j65t9fr6MknuTz5DLw+NmvADCrtB1wRy7ZwMFgwNT+TF3940GucKOolvzts+pVOcLZ7m8nPF5Ap7Seoy8BoqqdSXSR2OpeDxAGxHjz4GTxH+YCeqc6ILRbDk5+pO4m6LGZa5AvE1ohacJcw4D0HROx4VIUPNvnvJfeWep3pltaYKmEFZU6orqvWiM009PFMLILGb2wWQF+esgH8XrFpvda9c2KvjxK4wxiziAMH2QY2C2Bl78hd1WI2ifnSukPMNmm/8jfa4yLRW2KfVGfBJOk82/o5rbqd6vGq3akGE8X+MD9LV9M2XsL5gLBd9a4EC8V63nDgfJH6E5wl5ek7gD1i77rMLtW6VI+Q8p/xCf1I9X1pfc0cU01IGbggNmY7+QGyqq8hPM88Q3aq1pMnq8QOCl8xN1V3VA/Fdj4y9aG+5z23VA9U71TXqOdwqvEZtjplDDyo9AqZIpaVlv05Yhku0Eax8UfcG0D8SUkBFh3OQrjTAep8l7aqToq9JlYk/7K0PvcgTMWnnh0Xu3CRoS+yAP441XTfDuER+qeUnrcu1Vg2/5AxVGNxa2hoaGj4q/wCa6HTLuLkZKMAAAAASUVORK5CYII=>

[image38]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAFEAAAAYCAYAAACC2BGSAAADXUlEQVR4Xu2YS6iNURTHl0eUR4gi5GailLxi5BEDeQ28hULhTkyMxAQpA1IkIq/uwIBiJEoeAyTvEjEg8goJeSTKc/3P3uuc9f3t851zJ+rw/erf/dZ/r++1z95r7++KFBQUFBTUSz82cuigGsVmgmGqHmwS3VVj2GxUvrKRYKDql+qK6l083pvJCOyQ0LZStUb1PNtcoo2EnAuqaarHks5rKL6xQQxRfSSvWUJH+HMvRs9zMOEhnk8e8taR11B8Z4PYLeHFf5IPz3cQjl+5GGBaw29yHncqsLyGpVYn9pHwgi3kpzoR093TLfrHYtw2xozl5dFe1ZFNCXXa04tiwDkANZnpxEa91OrEFF0k3YmnXGzAfxCPx8c4hffnqU46D3V7j+qcaoUlSZgdZ1UzVAdUL1QbJHst/IBbnXdbQj0+5LzDqveqjdHD+yVZXUU/Eh6EF67GUwk3W+A8xMddbMDHPcDcGKeAj5Fqx/YXwmJkWNsRCSPYRvHScobITtWmePwm/rVrtYsxQE1/rZrovC2q8y7OMLSKMBLZg3qG05LgYfDLs1etE+3FMcLyOrGraoBUrg1vdDmj4uGl7TrrVR8qzSXQgXivtarJ0UP+oHJGxdtPHkb8VfJq0trpnNcJZ8izGngrxr1jzKRqJeoYewCer42Ih7sYYJpjyhozpfq1GHipeplLazoRNxjh4n3uGG13XAz6Rr/FeakHtzwPRphNRQPlg/M4BvAWuxh1kPPmJLzUc9RFvZ34RdWfPH9DHFvtM6ZH3696iHmVRd5n8rCYrCLvk+qoiyfIny/tp7qBmL27Ce+G805I2BHURT2diC+K+6rLqkuqm6onkn0IfMHwQ91LeIh3kYe82eQhD5t6A1OWr4Upyx7ilwkP+132UudisIDklxyW9JRQP9iDloXTSi9sN2RxQccD2HYGIIf3bVOjPzLGS2Lswci1e0xSLYzH43xS9CCUlcESnudaJkNkioQcrOIeeNj6eHD+Mwmftp2p7a+ySHVdwmdgNbBoYFo+Um2jNrBZKh2LThjr2jzIwXWaJNQ42yJ5sD2axaaynI0IfuR/AhtheWyXkOP3kAURLF7onNPc4LD/BNXq6P8SdBwWrIcSPuGwzWFQL9GGUoD69TbbXFBQUJDLb4QzBspS1t/EAAAAAElFTkSuQmCC>

[image39]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACbklEQVR4Xu2XOYgUQRiFX6AiKJiIeMKYaWYmJmaKIiqiiEYaiOYGZh6gqZkGCl6pkbCBoZiYGCiIJ4p4H3igCKLi8T+q/u2at9N/t+MOrNAfPLbrvX+qt/+Zrq4GOjo6OqYGY2oMYK9pq5otWK5Gwe9CTcwzfTG9Nr0wvTK9M23J+XXTM9OjrOem9zkbmtNo98+x5qCaAWzKRzTP3bY5zhGk+kXik21I2WcNhuUrmv+5O2jfnJ5pej5uc+HML6gZEM35CSlbpsEw8Od5G/UnI6uQbru2zSmJLoTsRsqXih8RzRllf8VG01k0N8ezUTSHa4TmM2VcshipnmvPIJrON0fGs2U8jk8SNeeeaWE+HkVzNL9qOiReCb9MZgc0yDA7p6axBNW5fmaPS8Xc8YoCruTexbrmrET/U2xUzfGLeYO0VkWf8SwSG6HQP4r0tL2cxwMbvA7pG3DqmqMex6NoTg/9t8l30/liXBLNdxj1mXJKDbIG/Y0hg5qjYzLZzeGj3nOK+6iIfUh1lzTIMHugpnDStF9Nxx/JkYh6qs25rolyTuWi6VY+Xo1Ud6yKJ/AWqWaFBhlmO9Us4O20Xs0mogsoYc1k/nLo++7Wx3eLYyWai0tFXUZuYOJO3RfmkOikJazhk6TkSfb3iO9Ec6vP8XHTLKRde8kMxHPdRH12DemV4wTSr26a6SEaXpnmm16iOukvpH2H8tT0DVUd32fW5ux+9nbkMdmEtLksP8MFl/M4C7Jfwnp+wz/E972Q6wOqdYdzcgH3jDtkv1WdK/nvGVR1j6t4arJdDWODGv8IX1ZLeqZd4nV0dHR0dPwn/AGkbO1R9fIXEwAAAABJRU5ErkJggg==>

[image40]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAADLElEQVR4Xu2XS6hPURTGP88UkWcJuSYepRjJQAwkJAORxwglDDCgGCiuvDKQCQPKKyURiWJ4U54ZICIieT/yyCPv1/qsve9ZZ919zv3f7kDq/Orrf/a31tln73323mf/gYqKiop/y3BRG286eohGebNGRos6edPw26g5+og+ip6LnoieiV6JpoX4ZdEj0b2gx6LXIdYiNoo+i+pFN0S/clGlK7TRB0RLwvXUXEYxbNwH0SXofW/z4Ry1Dk6kHprfz/lkBjT23gdqZYdohfPaomkDfZnQm+VNB3OGJLxUfYT+fm+WUFbXO2hsqA/Uyg/REW8i/8AJrhzhbNjkTUdsvF2unKX06o1H5kH9Qc4vo2xwymI1wSnPCnYbj9PxkykPgOawU+2MX8uDOaW/Oi8+c5vzuUf4Osv2qP7QfO49KZobnG6u3MWV0R1ZJdQYpPecb8hytkBn3ORcRu3Eevx09505I1rjPMseaGylDwQY2+tNZC+b+hm8m6JejRmGgcgP0Ih8uJGnyHJOuFhL4P3cDzy2My9EHYJXNDi2zUXiQHjorxdNF50O5aIB/rukOP0bkFW6OpcB9A7+ovBLcTa1lNnI3paHddYhv0z4jH2mbIntSLEWxTHPTm9EWMFi5/E8QH9pKG+GLiPLOpQ3LgXPHjyTpBiGrD5qYT7chPiSjvtAgLE73nTwS73cm5G4oaWgzw0yXs81sQi/VEX3e/h5vmjK3Pxsww6LrofrsdB6N2ThJryE5oz0gQBjc7xp4HIq3TP7orhzfMsHwzVz5ptYhB0sut/Cc9RR5y1DdpolrMeXb5lrD72UTyahOEauQGeqJbnUWQkb6qHfM1yfRdPPMTkmOmXKD6D3LTDeROjXj6fjc6IL0E4zr6PJ851heauos2iXi/G+ssG5iuLYeehfju3QWddedFd00iZFBkMrOgQ9U8wM5VU2CbrnfIHm80DHTvq/Abeh93LTjcROpBRJzWC+DL7h786PZ6GoN8j2nYfIHzn4RYxLNdIQfvkRinn3s3Aaru9r0H2Eo5liPLRyPnCci7UWvhTPFG+0Ev5ZtdQhvZdWVFRUVFT8B/wBfXcEMlpMJmMAAAAASUVORK5CYII=>

[image41]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAAYCAYAAACvKj4oAAACtElEQVR4Xu2WS6hNYRTHl7eUR7oMKOUVhYm6ukNliBgIJVwDA6VEkZF00+1S6qaYEBHKQCIZkIGUgTIxpNzrESLFQHnksf6ttfZee521j3NjYLB/9e/s9f+v851v7/3t72yihoaG/43FrNHRTBjLmh1N5Tvrl+pkyDI+s96zXqvesV5ptoj1hjXEesJ6rvmI2UcyoaOsH6zhSloymaTvKeuGHmcMkGRzYtAG9F+KpmIXrCsGnYATihPFFYzeteCd1/qM8wybUKf0Un1/D0lmd3VEYKllkzmmHnIwTuvLRQfReNYF1hjnGegdjmYbnlHrHIw7JNmOGHRCN+UnuF+9Pq1Pa72i6KhnAUnv9uCPCrUnm4PRLgO40J6ZvphC+QD96mFZAuvBHf3Jeqh19kzgOfLjvWRdJNks6jawbA5Gu+wLVfMlrJtlLGQDPFIPmwmwHt93WGssX4/v26qft9RbrrWnl6rjZ8IOGjnLusdawzpO0ve10qHgOfITBzYw7pSvdxYdpR+3bXhDrEOstepdIfn7yMAdxneWxUBBtjmaCdgsa8FdsAf9IOuAHu/R3E4wLjHzjV3Og+67rI44hucI1WfGdNanaP6JqyQD2wOMAbIfipPD3fQ1jle5OrKUpOdDDJRvlP+uMZ/y5VthL+tt8DCoX1Ib1MPV8sQTxDE2IQPPsr3NnGJNdRm4TvKd3cE3kNW9DeF5jisEK2598IpJTtN6rtbzig4BHv4uDDwz8LY5D/Wgq/EattplEfvt7C9kBklW2faVhSQZLtAWku+fUK8FvFY9JhkQSxJNmyodApYDsnVa4/huGReev0vYlM6R/E2sdP5tKt+gICzFF5rhzeijy9BnmQF/EmuWHpvsJrWA1yFsuQ9ikIDtGRsSTjiSPW8bWROj+ZdMCDU2t4aGhoaGf8pvEcTdA3RbWzkAAAAASUVORK5CYII=>

[image42]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAD4AAAAYCAYAAACiNE5vAAACSUlEQVR4Xu2WOWhWQRSFD+JSiAtJE5WIIoiVhUWEdAFJJdqYxsaAS6MGUtnZaaGVQVIE0mklmNZaTEo7EURETVRccM2iSVzucebl3dx/Zt57SQqF+eDwz5w7739zZiaTH8hkMpnm3LFGhN9KmkOiWdF3pTnRjOiLaFJ0bmn0P0I/3KTrsAEu9C9b8NxG66KQa3D+ZVtYK6atUcFpuJ2pG3wXXIA+W/CETkPBXcRrq6Zp8AXRCOoHH0V68qnT8A7pZ1dFk+BfRRtRL/g2/5na0R642pAtCF1wtaO2IKy3xkqoG5xH/Ipvp4I/gLucLqEM/WjZiJL7cPU24w96/6HxCU/HM9EH43P8PeMlqRt8UbVjwXkjz6v+OrgJnVCeplgYq4+iDjWu4DHcIhULpmE/do8EqRP8k2iT6seC8+WbA14M1n5aU9gDV3tv/GJRWeNdo4m+52ZE/ALrUcfcY39XXocmoeB8MW97TT/iF9d1uGeO24LnOeJh6Bd3COn1XiOqdnwYy4+h1YQfx7b9n/tCdMN4Bdzp1GSL77dsRavPe+WW8SqpCh7iCcI7zhvfeu2+zUXQxIKR/YjXB0RvjMdxB3z7hy6kWEnwp2g91jzSu1X/KsqJXxB1q1qnr4Vu7YMoQ3N3LedF31R/J8r3HBGdVbUkTYKPiz6jnBgnMOZre0VTon1+HI84LyP+Jtd/569QPk//reil6LXyQwui4ZjDootwz1Nc2NAJidIkeBVbRGeMd9L014LtcDu7Q3mnVLsW3KlMJpPJ/C/8AVjeuSO0X5HVAAAAAElFTkSuQmCC>

[image43]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEcAAAAYCAYAAACoaOA9AAACm0lEQVR4Xu2XTahOURSGVxGRMkHiTiShKMpQblI318/EQAbIgGRiSIbKiJFCrm6XDO7EQDFSTJSYKAmFgfzP/JafCOtt7d1Z3rP3Oed+fUjtp97ut99337P3Weec9X1HpFAoFP5PpqhWsdnAZTYyLFb9DDpF2QfVZ9UXp0+qj6o3qovV1H/DArGN3xLbED6P/DYjDeZ15ajY/AEOArF4Hlysbwm/b3xng1gmdvU8e8Q2hI3lSJ1MEz8kP39QLBvjIIDsHpv9oK04uM2xODbvaTr5l6qZks9TYO4TNgPXxPL5HCiTxLLdHPSDtuLMFVv8LPm54myW6gqn8hyYu5PNQG4t8FbS2XQ2eqGtOClmSH7D3kvlAI8qsvOqC6qHYZwjtRbuzBcJH+CxPyyW+R42HLzO9FKcZ2KLbCMfm8WmI7mNwEeBIneDl2JQquKwDrp5HuwDYM5J5+MbLrdOkl6KgwVGyVsv9YaZ2gjuFvQkz2nVY/Ii18WOM4sD5ZHU1xhXzVbtCBnaQgRjXNgaJzJCo2UP2mr/VoM3E+HCAJ4bm/Q08uHNIy8S75IcyI6xKeZ/TXhLyWtkIncODr7Sjc+4z/EkcgL73WdPygOxmPc5CCwSyzdxIObvTXgTomtx8CuVf6A1LYbHgPMjCW+78y75QDkuluXuYvz+4uNF2F/jPLQEfKm00qU4aHB4vm+qbqhuq55KfQOeOVLP45X2YBy9poyJTfwOBwFkk90Yfe6KyzrRVhz0n7hJ1ns3L4IG+EqqOehpz13+QLVRtS/ka8NfvD5sCHPOiR07HgOvLDg5NFO8V0W/6T0P+VXVCtVr1S7VO9UB1aFqWjNtxfkTrFOtduOpqiVu3C+GVFvceLnY49WZhWwUCoVCofDX+QWaRNrVI+ripQAAAABJRU5ErkJggg==>