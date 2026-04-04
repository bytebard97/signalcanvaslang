# **Technical Specifications and Signal Flow Analysis for Broadcast and Live Production Infrastructure**

The evolution of modern media facilities from rigid baseband architectures to flexible, software-defined IP ecosystems has transformed the requirements for signal flow documentation and infrastructure management. As facilities adopt standards such as SMPTE ST 2110 for uncompressed video, AES67 for high-performance audio, and various proprietary protocols like Dante and Clear-Com’s IVC, the complexity of the physical and logical layers increases. This report provides an exhaustive technical analysis of thirty-six pivotal infrastructure devices, spanning network switching, synchronization, intercom, and signal transport. Each device profile is structured to facilitate design rule checking—ensuring electrical and protocol compatibility across the signal chain—and to support the automated routing requirements of the SignalCanvas library.

## **Network Switching Infrastructures for High-Performance Media**

In the context of Audio-over-IP (AoIP) and Video-over-IP (VoIP), the network switch serves as the central matrix. Unlike standard enterprise IT deployments, media networks require specific capabilities to handle high-bandwidth multicast traffic and nanosecond-level synchronization. Key features such as Internet Group Management Protocol (IGMP) snooping, Precision Time Protocol (PTP) support, and high-density Power over Ethernet (PoE) are non-negotiable for stable operations.

### **Cisco SG350-28 Managed Switch**

The Cisco SG350-28 is a cornerstone of the "Small Business" line, offering a cost-effective entry point for managed Gigabit Ethernet in professional audio environments.1 While it lacks the ultra-deep buffers required for uncompressed 4K video, it is a dominant choice for Dante and AES67 audio networks due to its reliable Layer 2 and Layer 3 feature set.1

* **Category:** Network Switch  
* **Protocols:** Ethernet 1GbE, IPv4, IPv6, SNMP, RMON, HTTP/HTTPS, SSH, LACP 1  
* **Special Features:** IGMP Snooping (v1/v2/v3), MLD Snooping, QoS (802.1p), Layer 3 Static Routing (up to 28 routes), DoS Prevention 1

#### **Network Ports**

| Label | Qty | Connector | Speed | PoE | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Ports 1-24 | 24 | RJ45 | 1GbE | \- | Standard access ports 1 |
| G25-G26 | 2 | RJ45/SFP | 1GbE | \- | Combo ports (shared logic) 1 |
| G27-G28 | 2 | SFP | 1GbE | \- | Dedicated fiber uplinks 1 |
| Console | 1 | RJ45 | Serial | \- | RS-232 management 1 |
| USB | 1 | USB-A | \- | \- | Image/Config management 1 |

#### **Power**

* Connector: IEC C14 (Internal) 1  
* Redundant: No  
* Input: 100-240V AC, 50/60 Hz 1

The design rule implication for the SG350-28 centers on its combo ports. Design software must ensure that if port G25 (RJ45) is occupied, the corresponding G25 SFP slot is logically locked to prevent physical layer contention. Its 56 Gbps switching fabric and 16K MAC address table provide sufficient overhead for medium-scale Dante deployments.1

### **Cisco SG550X-48 Managed Switch**

For facilities requiring high-density Gigabit access with 10GbE backbones, the SG550X-48 offers a stackable architecture that allows up to eight switches to be managed as a single logical unit.2 This is critical for large-scale worship venues and broadcast centers where a unified control plane simplifies signal routing.4

* **Category:** Network Switch  
* **Protocols:** Dante, AES67, PTPv2 (Transparent), IPv4/IPv6, SNMP v3 2  
* **Special Features:** True Stacking (up to 8 units), IGMP Snooping, VRRP, RIP, 8 hardware QoS queues 2

#### **Network Ports**

| Label | Qty | Connector | Speed | PoE | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Ports 1-48 | 48 | RJ45 | 1GbE | \- | 10/100/1000Base-T 2 |
| 10G Combo | 2 | RJ45/SFP+ | 10GbE | \- | 10GBase-T / SFP+ combo 5 |
| 10G SFP+ | 2 | SFP+ | 10GbE | \- | Dedicated uplinks/stacking 5 |
| Management | 1 | RJ45 | 1GbE | \- | Dedicated OOB port 2 |
| Console | 1 | RJ45 | Serial | \- | Cisco standard pinout 2 |
| USB | 1 | USB-A | \- | \- | Front panel file transfer 2 |

#### **Power**

* Connector: IEC C14  
* Redundant: Yes (External RPS connector) 2

The SG550X-48 provides 176 Gbps of switching capacity, ensuring non-blocking performance across all 48 ports even when the 10GbE uplinks are saturated.2 The inclusion of a dedicated Out-of-Band (OOB) management port allows for the separation of control traffic from high-bandwidth media streams, a recommended practice in broadcast engineering.2

### **Luminex GigaCore 30i AV-Focused Switch**

The GigaCore 30i is engineered specifically for the AV industry, removing the need for complex command-line configuration by providing pre-defined optimizations for Dante, RAVENNA, AES67, and Milan/AVB.6 Its color-coded "Groups" (VLANs) provide immediate visual feedback for port assignments.7

* **Category:** Network Switch  
* **Protocols:** Dante, AES67, ST 2110, Milan/AVB (IEEE 802.1 BA-2011) 6  
* **Special Features:** RLinkX (Redundancy Protocol), IGMP Querier/Snooping (v1/v2/v3), PTPv2 Hardware Timestamping 6

#### **Network Ports**

| Label | Qty | Connector | Speed | PoE | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Ports 1-24 | 24 | RJ45 | 1GbE | PoE++ (90W) | 500W-1000W budget 6 |
| Ports 25-30 | 6 | SFP+ | 10GbE | \- | Independent 1G/10G cages 6 |
| Serial | 1 | USB-C | Serial | \- | Rear panel RS232 8 |
| LED Extension | 1 | USB-C | \- | \- | For remote 19" LED panel 8 |

#### **Power**

* Connector: Dual IEC C14 (Slide-in PSUs) 6  
* Redundant: Yes (Hot-swappable) 7  
* PoE Class: Class 0-8 (802.3bt compliant) 6

Luminex utilizes the Araneo software to visualize the entire network topology, making it easier to identify bottlenecks or single points of failure. The GigaCore 30i supports jumbo frames up to 12,000 MTU, which is vital for high-resolution video-over-IP applications.6

### **Luminex GigaCore 16Xt Compact Switch**

The GigaCore 16Xt is a modular, compact switch designed for touring and stage boxes. It features ruggedized connectivity via etherCON and supports the same "AV-centric" software stack as the 30i.9

* **Category:** Network Switch  
* **Protocols:** Dante, AVB, AES67, PTPv1/v2 8  
* **Special Features:** RLinkX automatic redundancy, Group-based VLANs, low jitter clocking 9

#### **Network Ports**

| Label | Qty | Connector | Speed | PoE | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Ports 1-12 | 12 | RJ45/etherCON | 1GbE | PoE+ (Optional) | Front-facing rugged ports 9 |
| Ports 13-16 | 4 | SFP | 1GbE | \- | Fiber connectivity 9 |

#### **Power**

* Connector: IEC C14 \+ RPSU Input  
* Redundant: Yes (External RPSU module) 9

The 16Xt is particularly notable for its integration with lighting consoles and sound processors in live event rigs.9 The device's ability to handle PTP v2 is critical for maintaining phase alignment in distributed audio systems.8

### **Netgear M4300-28G (GSM4328PB)**

Netgear's M4300 series is part of the "Intelligent Edge" family, marketed as "SDVoE-Ready" for uncompressed video-over-IP.10 The GSM4328PB model offers a high PoE budget, making it suitable for powering large arrays of IP intercom panels and PTZ cameras.11

* **Category:** Network Switch  
* **Protocols:** SDVoE, Dante, AES67, PTPv2 (1-step Transparent Clock) 10  
* **Special Features:** Mixed Stacking (1G/10G/40G), Zero Touch Multicast, IGMP Fast Leave 10

#### **Network Ports**

| Label | Qty | Connector | Speed | PoE | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Ports 1-24 | 24 | RJ45 | 1GbE | PoE+ (30W) | Budget up to 720W 11 |
| 10G Copper | 2 | RJ45 | 10GbE | \- | Independent 100M/1G/10G 10 |
| 10G SFP+ | 2 | SFP+ | 10GbE | \- | Independent 1G/10G 10 |
| OOB Port | 1 | RJ45 | 1GbE | \- | Front management port 10 |
| Console | 2 | RJ45/Mini-USB | Serial | \- | Back/Front management 11 |
| Storage | 1 | USB-A | \- | \- | Front panel 11 |

#### **Power**

* Connector: Dual Modular PSU Bays 11  
* Redundant: Yes (Modular hot-swap) 10

The M4300-28G ensures non-stop forwarding (NSF) across the stack, providing hitless failover that is essential for live broadcast environments.11 The switch fabric is rated at 128 Gbps, preventing backplane congestion during high-bandwidth video transfers.11

### **Arista 7010T-48 Data Center Switch**

Arista Networks provides the high-performance switching fabric often found in the core of major broadcast facilities. The 7010T-48 is a low-latency switch with a deep 4MB packet buffer, designed to absorb the micro-bursts characteristic of uncompressed SMPTE 2110 video streams.12

* **Category:** Network Switch  
* **Protocols:** ST 2110, ST 2022-7, PTP (Boundary/Transparent), OSPF, BGP 12  
* **Special Features:** Arista EOS (Extensible OS), CloudVision, VM Tracer, 4MB Dynamic Buffer 12

#### **Network Ports**

| Label | Qty | Connector | Speed | PoE | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 1000Base-T | 48 | RJ45 | 1GbE | \- | 10/100/1000 access 12 |
| Uplinks | 4 | SFP+ | 10GbE | \- | 1/10GbE dual-speed 12 |
| Management | 1 | RJ45 | 1GbE | \- | Dedicated OOB interface 13 |
| Console | 1 | RJ45 | Serial | \- | RS-232 serial 13 |
| USB | 1 | USB-A | \- | \- | Configuration recovery 12 |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes (1+1 internal hot-plug) 13

The 7010T-48 delivers a throughput of 176 Gbps with a forwarding rate of 132 Mpps.13 Its front-to-back or back-to-front airflow options allow it to be integrated into various rack cooling strategies.14

## **Master Synchronization and Reference Generation**

In a digital broadcast facility, synchronization ensures that all video frames align perfectly and all audio samples are captured at the precise moment. This requires a master timebase, typically referenced to an internal high-stability oscillator or an external Global Navigation Satellite System (GNSS) like GPS or GLONASS.

### **AJA GEN10 HD/SD/AES Sync Generator**

The AJA GEN10 is a compact, reliable sync generator capable of providing SD black burst and HD tri-level sync simultaneously.15 It is unique for its ability to generate AES-11 silence or tone, making it highly valuable for multi-format production environments.16

* **Category:** Sync Generator  
* **Protocols:** SD Black Burst, HD Tri-Level, AES-11 15  
* **Special Features:** Independent group control, 19 HD formats supported, 3 ppm accuracy 15

#### **Sync/Reference I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Outputs 1-4 | 4 | BNC 75Ω | Output | SD Black / HD Tri-Level 15 |
| Outputs 5-6 | 2 | BNC 75Ω | Output | SD Black / HD Tri-Level 15 |
| AES-11 | 1 | BNC 75Ω | Output | AES Silence/Tone (48kHz) 17 |

#### **Power**

* Connector: 5-18V DC Locking  
* Redundant: No (Universal power supply included) 15

Design rule validation for the GEN10 must account for its dual-group architecture. Outputs 1-4 are linked to Group 1, while Outputs 5-6 are linked to Group 2; each group can be independently set to HD or SD formats, but individual outputs within a group must share the same standard.15

### **Evertz 5601MSC Master Sync and Clock Generator**

The Evertz 5601MSC is a high-availability master clock designed for the most demanding broadcast operations. It features a high-stability temperature-controlled oscillator with a frequency reference better than 0.01 ppm.19

* **Category:** Master Clock / SPG  
* **Protocols:** Black Burst, Tri-Level, Word Clock, PTP (Option), NTP, LTC 19  
* **Special Features:** GPS/GLONASS reference, VistaLINK control, hot-swappable redundancy 19

#### **Sync/Reference I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Sync Outputs | 6 | BNC 75Ω | Output | Black Burst / Tri-Level 20 |
| Word Clock | 1 | BNC 75Ω | Output | 48kHz Word Clock 20 |
| 10MHz Out | 1 | BNC 75Ω | Output | 10MHz Sine/CW 20 |
| LTC Out | 2 | XLR-3M | Output | Linear Timecode 19 |
| LTC In | 1 | XLR-3F | Input | Linear Timecode 21 |
| GPS In | 1 | BNC 50Ω | Input | GPS/GLONASS Antenna 19 |
| NTP Port | 1 | RJ45 | Bidirectional | PTP / NTP 19 |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes (Hot-swappable) 21

A critical distinction for SignalCanvas documentation is the GPS input, which utilizes a **BNC 50Ω** connector. Design rules must flag any attempt to patch this into a 75Ω video distribution network.19 The 5601MSC can provide RFC-1305 compliant NTP and act as a PTP server, bridging the gap between legacy analog and modern IP timing.19

### **Blackmagic Design Sync Generator**

Blackmagic's offering provides a streamlined solution for providing HD tri-level or SD black burst sync to cameras and switchers in portable rigs.

* **Category:** Sync Generator  
* **Protocols:** SD Black Burst, HD Tri-Level  
* **Special Features:** Compact form factor, 6 outputs

#### **Sync/Reference I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Reference Out | 6 | BNC 75Ω | Output | Black Burst / Tri-Level |

#### **Power**

* Connector: 12V Locking DC  
* Redundant: No

This device is often used as a localized reference for Blackmagic URSA cameras or ATEM switchers, ensuring that all 12G-SDI sources remain in phase for clean switching.

### **Tektronix SPG8000A Master Sync Generator**

The SPG8000A is a precision multiformat video signal generator suitable for master synchronization. It features "Stay GenLock" technology, which prevents synchronization shock by gradually recovering phase if an external reference or GPS signal is lost and then regained.24

* **Category:** Sync Generator  
* **Protocols:** NTSC/PAL Blackburst, HD Tri-Level, 10MHz CW, 1 PPS, PTP 24  
* **Special Features:** GPS/GLONASS Receiver, SNMP Reporting, Web UI, redundant power 24

#### **Sync/Reference I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| GENLOCK IN | 2 | BNC 75Ω | Input | Passive Loop-through 25 |
| BLACK 1-3 | 3 | BNC 75Ω | Output | SD Black / HD Tri-Level 25 |
| LTC I/O | 1 | DB-15 | Bidirectional | 4 Out or 1 In/3 Out 24 |
| Word Clock | 1 | BNC 75Ω | Output | 48kHz (General Purpose) 25 |
| GPS Input | 1 | BNC 50Ω | Input | GPS/GLONASS Antenna 24 |
| Ethernet | 1 | RJ45 | Bidirectional | Management / PTP 24 |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes 24

The SPG8000A supports VITC read from NTSC/PAL genlock inputs and can provide DC antenna power (3.3V or 5V) to active GPS antennas via the 50Ω BNC port.24

## **Audio Word Clock and Frequency Distribution**

For high-resolution audio mastering and recording, the word clock must be distributed with minimal jitter to ensure phase coherence across multiple AD/DA converters.

### **Antelope Audio OCX HD Master Clock**

The OCX HD supports sampling rates up to 768kHz and utilizes Antelope's 4th Generation Acoustically Focused Clocking (AFC) algorithm.26 It is designed to act as the central timing hub for complex studio and live sound environments.27

* **Category:** Master Clock  
* **Protocols:** Word Clock, AES/EBU, S/PDIF, 10MHz 18  
* **Special Features:** Support up to 768kHz, Oven Controlled Crystal Oscillator, Atomic Clock Input 18

#### **Sync/Clock I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| WC Out | 10 | BNC 75Ω | Output | Word Clock (up to 768kHz) 27 |
| AES/EBU Out | 4 | XLR-3M | Output | AES3 (up to 192kHz) 27 |
| S/PDIF Out | 2 | RCA | Output | S/PDIF (up to 192kHz) 27 |
| WC In | 2 | BNC 75Ω | Input | Word Clock 27 |
| Video In | 1 | BNC 75Ω | Input | SD/HD Sync reference 27 |
| Atomic In | 1 | BNC 75Ω | Input | 10MHz (3Vpp) 18 |
| AES/EBU In | 1 | XLR-3F | Input | AES3 27 |
| USB | 1 | USB-B | Bidirectional | Control 27 |

#### **Power**

* Connector: IEC C14 (Universal 95-245V) 27  
* Redundant: No

The OCX HD allows the output of five distinct sample rates simultaneously via multipliers and dividers, which is vital for facilities running multiple rooms at different rates.27

### **Brainstorm Electronics DCD-8 Digiclocks**

The DCD-8 is a "Distripalyzer"—a word clock generator, distributor, and analyzer. It can extract a clean word clock from virtually any digital audio or video signal, making it the "Swiss Army Knife" of audio clocking.28

* **Category:** Master Clock / Distributor  
* **Protocols:** Word Clock, AES/EBU, S/PDIF, ADAT, FireWire, NTSC/PAL/HD Tri-Level 28  
* **Special Features:** Analyzer/Frequency Counter, Format Converter, Dual redundant generators 28

#### **Sync/Clock I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| WC In | 3 | BNC 75Ω | Input | 32kHz \- 192kHz 28 |
| WC Out | 8 | BNC 75Ω | Output | Word Clock 28 |
| AES-EBU In | 3 | XLR/BNC | Input | AES3 28 |
| Optical | 1 | TOSLINK | Bidirectional | ADAT / S/PDIF 28 |
| Video | 1 | BNC 75Ω | Input | SD/HD reference 28 |
| FireWire | 1 | 6-pin 1394 | Bidirectional | Audio/Control 28 |

#### **Power**

* Connector: 4-pin DC (12VDC, 2A) 28  
* Redundant: No (External switching supply) 28

The DCD-8's outputs 7 and 8 feature multipliers (up to x256), specifically designed to support Pro Tools "Super Clock" and DSD requirements.28

### **Mutec MC-3+ Smart Clock**

The Mutec MC-3+ is a high-performance audio clock and re-clocker that uses proprietary "1G-Clock" technology to stabilize digital audio signals and eliminate jitter.

* **Category:** Audio Clock / Re-clocker  
* **Protocols:** Word Clock, AES3, S/PDIF  
* **Special Features:** Bi-directional signal conversion, ultra-low jitter generation

#### **Sync/Clock I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| WC In | 1 | BNC 75Ω | Input | Word Clock |
| WC Out | 6 | BNC 75Ω | Output | Word Clock |
| AES3 In | 1 | XLR-3F | Input | AES/EBU |
| AES3 Out | 1 | XLR-3M | Output | AES/EBU |
| S/PDIF In | 1 | RCA / Opt | Input | S/PDIF |
| S/PDIF Out | 1 | RCA / Opt | Output | S/PDIF |

#### **Power**

* Connector: IEC C14 (Internal)  
* Redundant: No

The MC-3+ is often used to "clean" the output of digital mixers or DAW interfaces before they hit high-end converters, ensuring that the entire digital chain is locked to a pristine timebase.

## **Digital Intercom Matrix Frames**

Intercom matrices are the brain of the communications system, managing thousands of simultaneous audio routes across panels, beltpacks, and telephone interfaces.

### **Clear-Com Eclipse HX Omega**

The Eclipse HX Omega is a 6RU matrix frame with slots for 15 interface cards, providing up to 496 timeslots/ports.29 It is the flagship of the Eclipse line, designed for global broadcast operations.

* **Category:** Intercom Matrix  
* **Intercom Protocol:** Clear-Com IVC, AES67, Dante, MADI, Analog 29  
* **Special Features:** Redundant CPUs, Redundant PSUs, EHX Configuration Software 29

#### **System I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| LAN 1-2 | 2 | RJ45 | Bidirectional | Fast Ethernet (Control) 29 |
| GPI | 1 | DB-25F | Input | 8 Optically Isolated 29 |
| GPO | 1 | DB-25M | Output | 8 Power Relays 29 |
| Alarm | 1 | DB-9F | Output | 1 GPI, 1 GPO 29 |
| 2-Wire I/O | 4 | XLR-3F | Bidirectional | Partyline I/O 29 |
| Serial | 1 | DB-9F | Bidirectional | RS-232 Management 29 |

#### **Card Slots**

* Interface Card Slots: 15 (Supports MVX-16A, E-IPA, E-Dante64, E-MADI64) 29  
* CPU Slots: 2 (Redundant) 29

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes (Load-sharing) 29

The frame's "Housekeeper" status panel provides dedicated LEDs for PSU voltage rails and fan status, critical for maintenance in high-stakes environments.29

### **Clear-Com Eclipse HX Delta**

The Delta is a 3RU frame designed for medium-scale installations. It provides slots for 4 interface cards and 3 smaller interface modules.30

* **Category:** Intercom Matrix  
* **Intercom Protocol:** IVC, AES67, Dante, MADI 31  
* **Special Features:** Redundant CPU support, Compact 3RU frame, 416 timeslots 30

#### **System I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| LAN 1-2 | 2 | RJ45 | Bidirectional | Fast Ethernet 31 |
| GPI | 1 | DB-25F | Input | 8 GPI 31 |
| GPO | 1 | DB-25M | Output | 8 GPO 31 |
| Alarm | 1 | DB-9F | Output | 1 GPI, 1 GPO 31 |
| DC Power | 2 | KPJX-4S-S | Input | 12V DC 30 |

#### **Power**

* Connector: Dual external PSUs (KPPX-4-P) 30  
* Redundant: Yes 31

The Delta is frequently trunked with other Eclipse HX systems via fiber (E-FIB) or IP (E-IPA), allowing for a decentralized matrix architecture.30

### **RTS ADAM-M Modular Matrix**

The ADAM-M is a 3RU modular intercom matrix favored by major television networks. It utilizes RTS's OMNEO (AES67/Dante) protocol for high-quality IP audio.

* **Category:** Intercom Matrix  
* **Intercom Protocol:** RTS/OMNEO, RVON (IP), Analog  
* **Special Features:** 3Gb/s backplane, support for up to 256 ports in 3RU, redundant controllers.

#### **System I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Controller I/O | 2 | RJ45 | Bidirectional | Management/OMNEO |
| DB-9 | 1 | DB-9 | Bidirectional | Serial RS-232/485 |
| GPI/O | 1 | DB-25 | Bidirectional | Logic Triggers |

#### **Card Slots**

* Support for AIO (Analog), OMNEO (IP), and MADI cards.

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes

The ADAM-M is a staple in mobile units (OB trucks) where port density and reliability are the primary engineering constraints.

### **Riedel Artist-1024 Intercom Matrix**

The Artist-1024 is a 2RU digital matrix node that redefines high-density intercom. It supports up to 1024 ports and utilizes a decentralized fiber ring topology.33

* **Category:** Intercom Matrix  
* **Intercom Protocol:** AES67, Dante, MADI, Artist Fiber, ST 2110-30/31 33  
* **Special Features:** Software-defined Universal Interface Cards (UIC), 1024 non-blocking ports, E-ink display 33

#### **System I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| UIC Slots | 8 | SFP+ | Bidirectional | UIC-128/UIC-128-II 34 |
| NIC Slots | 2 | SFP+ | Bidirectional | Artist Fiber 34 |
| Management | 2 | RJ45 | Bidirectional | Fast Ethernet 34 |
| GPIO | 1 | DB-25 | Bidirectional | 8 in, 8 out 33 |

#### **Power**

* Connector: Dual IEC C14 (PSU-1024) 34  
* Redundant: Yes (Hot-swappable, load-sharing) 34

Riedel's UIC cards can be reconfigured via software to support different protocols (e.g., swapping from MADI to AES67) without physical module changes, providing unmatched flexibility.35

### **Riedel Artist-128 Frame**

The Artist-128 is a 6RU mainframe for larger decentralized networks, supporting up to 16 client cards and 2 controller modules.36

* **Category:** Intercom Matrix  
* **Intercom Protocol:** Artist Fiber, AVB, AES67, VoIP, MADI 35  
* **Special Features:** Distributed masterless design, hot-swappable cards, redundant controllers 37

#### **System I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Client Slots | 16 | Proprietary | Bidirectional | Mixed Analog/Digital 37 |
| Controller Bays | 2 | Proprietary | Bidirectional | CPU-128F / NIC 37 |
| Power In | 2 | IEC C14 | Input | 90-264V AC 37 |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes 37

The Artist-128 node stores its entire configuration locally, ensuring that even if the fiber ring is broken, the local node remains operational.37

## **Intercom User Panels and Stations**

Intercom panels are the primary interface for directors and producers. Modern panels are almost entirely IP-based, utilizing PoE for power and AES67/Dante for audio transport.

### **Clear-Com V-Series V12 Lever Keypanel**

The V12 is a 12-key user station available in Lever, Rotary, or Pushbutton variants.30 It connects to the Eclipse HX matrix via CAT5 or IP.29

* **Category:** Intercom Panel  
* **Intercom Protocol:** Analog 4-wire, IVC (IP), AES67 (Iris version) 29  
* **Special Features:** High-resolution OLED displays, shift keys for up to 24 keys, front-panel headset ports 30

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Headset | 1 | XLR-4M | Bidirectional | Mic/Ear 29 |
| LAN | 1 | RJ45 | Bidirectional | IP Intercom 29 |
| Matrix | 1 | RJ45 | Bidirectional | Analog 4-wire 30 |
| Aux Audio | 1 | DB-9 | Bidirectional | Local I/O 29 |

#### **Power**

* Connector: 4-pin DC (12V) or PoE  
* Redundant: No

The V-Series Iris variant supports AES67 natively, allowing it to be connected to any standard AoIP network switch and routed directly to an E-IPA matrix card.29

### **RTS KP-5032 OMNEO Intercom Panel**

The KP-5032 is a 32-key digital intercom station that uses the OMNEO protocol for Dante-compatible IP audio transport.

* **Category:** Intercom Panel  
* **Intercom Protocol:** RTS/OMNEO, RVON  
* **Special Features:** High-resolution TFT displays, multi-directional keys, full-duplex operation.

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Headset | 1 | XLR-4F / 5F | Bidirectional | Mic/Ear |
| LAN | 2 | RJ45 / etherCON | Bidirectional | OMNEO IP |
| Mic In | 1 | XLR-3F | Input | Gooseneck Mic |
| Aux In/Out | 1 | DB-9 | Bidirectional | Analog Audio |

#### **Power**

* Connector: IEC C14 or PoE  
* Redundant: No

The dual network ports on the KP-5032 allow for daisy-chaining or redundant network paths, a critical feature for large venues with limited cable infrastructure.

### **Riedel RSP-1232HL SmartPanel**

The 1200 series SmartPanel is Riedel's "App-on-Hardware" interface, combining intercom, control, and audio monitoring in a 2RU panel.33

* **Category:** Intercom Panel  
* **Intercom Protocol:** ST 2110-30/31, AES67, Dante 33  
* **Special Features:** 32 Lever Keys, dual touchscreens, app-driven ecosystem 33

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| LAN 1-2 | 2 | RJ45 / SFP+ | Bidirectional | IP Media 33 |
| Headset | 2 | XLR-5F | Bidirectional | Dual User support 33 |
| GPIO | 1 | DB-15 | Bidirectional | 3 In, 3 Out 33 |

#### **Power**

* Connector: IEC C14 (Dual internal PSUs) 33  
* Redundant: Yes

The RSP-1232HL serves as a "control point" for the entire MediorNet/Artist ecosystem, allowing for the remote control of routers and switchers directly from the intercom surface.33

### **Green-GO MCXD Multi-Channel Desktop Station**

Green-GO systems utilize a unique peer-to-peer IP architecture that does not require a central matrix or frame.10 The MCXD is a 32-channel desktop station.

* **Category:** Intercom Panel  
* **Intercom Protocol:** Green-GO IP  
* **Special Features:** 32 channels, color OLED displays, PoE powered 10

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Network | 2 | etherCON | Bidirectional | 100Mbps IP 10 |
| Headset | 1 | XLR-4M | Bidirectional | Mic/Ear 10 |
| GPIO | 1 | DB-9 | Bidirectional | 2 In, 2 Out 10 |

#### **Power**

* Connector: PoE powered  
* Redundant: Yes (Dual network ports)

Green-GO's simplicity is its strength; the entire system configuration is stored on every device, providing unmatched resiliency in live environments.

## **Intercom Beltpacks**

Beltpacks provide mobility for camera operators and floor managers, requiring rugged connectors and reliable wireless or wired protocols.

### **Clear-Com RS-702 2-Channel Beltpack**

The RS-702 is a traditional 2-channel analog partyline beltpack, powered via the intercom line itself.27

* **Category:** Intercom Beltpack  
* **Intercom Protocol:** Clear-Com Analog 2-Wire  
* **Special Features:** Low power consumption, "Call" light signaling, rugged metal housing 27

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Intercom | 2 | XLR-3 (M/F) | Bidirectional | 2-Wire Audio/Power 27 |
| Headset | 1 | XLR-4M | Bidirectional | Mic/Ear 27 |

#### **Power**

* Power: 24-30V DC (intercom line powered)

The RS-702 uses a standard XLR-3 connector for daisy-chaining multiple beltpacks together on a single cable run, a hallmark of traditional partyline systems.

### **Riedel Bolero Wireless Beltpack**

Bolero is a high-performance DECT wireless intercom system that can operate as a standalone system or integrated into an Artist matrix.33

* **Category:** Intercom Beltpack  
* **Intercom Protocol:** DECT (1.9GHz / 2.4GHz), IP 33  
* **Special Features:** 6 keys, high-resolution display, "Touch-and-Go" registration 33

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Headset | 1 | XLR-4M / 5F | Bidirectional | High fidelity audio 33 |
| USB-C | 1 | USB-C | Bidirectional | Charging/Updates 33 |

#### **Power**

* Power: Internal Li-ion battery (up to 17 hours) 33

Bolero beltpacks can also be used as wireless walkie-talkies or even as user stations via the integrated microphone and speaker.33

### **Green-GO WBPX Wireless Beltpack**

The WBPX is the wireless equivalent of the Green-GO wired beltpack, operating over standard 2.4GHz or 5GHz Wi-Fi networks.

* **Category:** Intercom Beltpack  
* **Intercom Protocol:** Green-GO IP (Wireless)  
* **Special Features:** 4 channels, high-resolution OLED, PoE charging 10

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Network | 1 | RJ45 | Bidirectional | Charging/Programming 10 |
| Headset | 1 | XLR-4M | Bidirectional | Mic/Ear 10 |

#### **Power**

* Power: Internal battery (up to 12 hours)

The WBPX brings the peer-to-peer Green-GO architecture to mobile users, allowing them to roam across multiple Wi-Fi access points without losing communication.

## **Audio Interfaces and Dante Converters**

As audio moves from analog cables to network packets, high-quality converters are required to bridge the gap between traditional equipment and the digital network.

### **Focusrite RedNet A16R MkII Analog Interface**

The A16R MkII provides 16 channels of balanced analog line-level I/O for Dante networks.38 It is designed for rack installation with dual PSUs and network ports for maximum reliability.38

* **Category:** Audio Interface  
* **Protocols:** Dante, AES67, ST 2110-30 38  
* **Special Features:** 119dB Dynamic Range, remote control via RedNet Control, Internal SRC 38

#### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Analog In | 2 | DB-25 | Input | Balanced Line | 16 total 41 |
| Analog Out | 2 | DB-25 | Output | Balanced Line | 16 total 41 |
| AES3 In | 1 | XLR-3F | Input | AES/EBU | 2 channels 40 |
| AES3 Out | 1 | XLR-3M | Output | AES/EBU | 2 channels 40 |
| Word Clock | 2 | BNC 75Ω | Bidirectional | House Clock 41 |  |
| Dante Pri/Sec | 2 | etherCON | Bidirectional | Redundant Network 41 |  |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes 38

The DB-25 connectors utilize the **AES59 / Tascam Digital** pinout standard, a critical detail for wiring SignalCanvas templates.38

### **Focusrite RedNet D16R MkII AES Interface**

The D16R MkII provides 16 channels of AES3 I/O, allowing digital consoles and processors to be integrated into a Dante network.41

* **Category:** Audio Interface  
* **Protocols:** Dante, AES67, AES3 41  
* **Special Features:** Individual channel gain/trim, internal SRC on all inputs, DARS support 41

#### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AES3 I/O | 2 | DB-25 | Bidirectional | AES3 | 16 total 41 |
| Alt AES3 In | 1 | XLR-3F | Input | AES3 / DARS 41 |  |
| Alt AES3 Out | 1 | XLR-3M | Output | AES3 41 |  |
| S/PDIF In | 1 | RCA | Input | S/PDIF 41 |  |
| S/PDIF Out | 1 | RCA | Output | S/PDIF 41 |  |
| Word Clock | 2 | BNC 75Ω | Bidirectional | Sync 41 |  |
| Dante Pri/Sec | 2 | etherCON | Bidirectional | Redundant IP 41 |  |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes 41

Sample rate conversion on all input channels allows external digital gear to operate at any rate from 32kHz to 216kHz while the Dante network remains locked to house clock.41

### **Audinate Dante AVIO Adapters**

AVIO adapters are single-purpose Dante interfaces designed to bring analog, AES3, or USB devices onto the network.

* **Category:** Audio Interface  
* **Protocols:** Dante  
* **Special Features:** PoE powered, plug-and-play operation.

#### **Models**

1. **Analog In:** 1 or 2 x XLR-3F to Dante.  
2. **Analog Out:** 1 or 2 x XLR-3M to Dante.  
3. **AES3:** 1 x XLR-3M/F to Dante.  
4. **USB:** 1 x USB-A to Dante (2x2 channels).

#### **Power**

* Connector: PoE Class 1  
* Redundant: No

These adapters are essential for "last-mile" connectivity, such as connecting a single powered speaker or a laptop to the facility audio network.

### **DirectOut PRODIGY.MC Modular Converter**

The PRODIGY.MC is a high-density modular converter capable of managing up to 320 inputs and 324 outputs across MADI, Dante, RAVENNA, and Analog formats.42

* **Category:** Audio Converter  
* **Protocols:** MADI, Dante, RAVENNA, AES67, ST 2110-30/31 42  
* **Special Features:** Modular A/B/C architecture, EARS redundancy switching, integrated matrix 42

#### **System Slots**

| Slot | Type | Signal | Channels | Notes |
| :---- | :---- | :---- | :---- | :---- |
| Slot A | Network | Dante/RAV/MADI | 128 | Primary network interface 42 |
| Slot B | MADI | BNC/SC/SFP | 64 | Secondary MADI I/O 42 |
| Slot C (8) | I/O | Analog/AES3 | 8 | Modular converters 42 |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes 42

The PRODIGY.MC supports sample rates up to 192kHz and includes a 5-inch touch display for real-time monitoring and routing.8

## **High-Density MADI Ecosystems**

Multichannel Audio Digital Interface (MADI) remains a vital standard for connecting consoles and routers due to its high channel count (64ch) over simple BNC or fiber cables.

### **DirectOut ANDIAMO 2 XT MADI/AES Converter**

The ANDIAMO 2 XT provides 32 analog and 32 digital (AES3) I/Os in a single unit, converting them to and from MADI.44

* **Category:** MADI Interface  
* **Protocols:** MADI (AES10), AES3, Analog 44  
* **Special Features:** Two MADI ports for redundancy, 4 FS support (up to 192kHz), remote control via USB/MIDI 44

#### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Analog I/O | 8 | DB-25 | Bidirectional | Balanced Line | 32 total 44 |
| AES3 I/O | 4 | DB-25 | Bidirectional | AES3 | 32 total 44 |
| MADI 1 | 1 | SC Optical | Bidirectional | MADI | 64 channels 44 |
| MADI 2 | 1 | SC / BNC | Bidirectional | MADI | 64 channels 44 |
| Word Clock | 2 | BNC 75Ω | Bidirectional | Sync 44 |  |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes (Phase-redundant) 43

The "Extended Routing" feature allows both MADI I/Os to be used independently, effectively doubling the available MADI input capacity or creating two individual MADI feeds.44

### **RME MADI Router**

The RME MADI Router is a patchbay and format converter that can route audio between its 12 MADI ports (4 x Optical, 4 x Coaxial, 4 x MADI TP).

* **Category:** MADI Router  
* **Protocols:** MADI  
* **Special Features:** Low-latency routing, format conversion (Optical to Coax), front-panel matrix control.

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| MADI Opt | 4 | SC Optical | Bidirectional | MADI |
| MADI Coax | 4 | BNC 75Ω | Bidirectional | MADI |
| MADI TP | 4 | RJ45 | Bidirectional | MADI over CAT5 |
| Word Clock | 2 | BNC 75Ω | Bidirectional | Sync |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes

The MADI Router acts as a central hub, allowing engineers to split MADI signals to multiple recorders or combine various sources into a single fiber trunk.

### **Ferrofish A32 AD/DA Converter**

The Ferrofish A32 is a high-density 32-channel converter that supports MADI, ADAT, and Dante, providing a bridge between legacy studio formats and modern broadcast networks.47

* **Category:** Audio Interface  
* **Protocols:** Dante (Option), MADI, ADAT 47  
* **Special Features:** 32x32 analog I/O, four TFT status screens, integrated routing matrix 47

#### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Analog In/Out | 8 | DB-25 | Bidirectional | Balanced Line | 32 total 50 |
| MADI SFP | 2 | SFP Cages | Bidirectional | MADI | 64 channels 50 |
| ADAT I/O | 8 | TOSLINK | Bidirectional | ADAT | 32 total 50 |
| Word Clock | 2 | BNC 75Ω | Bidirectional | Sync 51 |  |
| MIDI I/O | 2 | 3.5mm Jack | Bidirectional | Control 50 |  |

#### **Power**

* Connector: Dual DC Barrel (Locking)  
* Redundant: Yes (12V, 3A) 51

The A32 includes a sophisticated headphone output that can monitor any mono or stereo pair from the entire 256-channel routing matrix.47

## **Fiber Transport and Media Conversion Nodes**

Fiber optic transport is the backbone of modern facility routing, enabling the transmission of uncompressed 4K video and hundreds of channels of audio over kilometers of cable.

### **Riedel MediorNet MicroN Node**

MicroN is a high-density media distribution node for Riedel’s MediorNet fabric. It acts as a decentralized router with integrated signal processing.

* **Category:** Fiber Transport / Router  
* **Protocols:** 3G-SDI, 10G MediorNet, ST 2110 (Option)  
* **Special Features:** Integrated frame sync, embedding/de-embedding, timecode management.

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| SDI I/O | 12 | BNC 75Ω | Bidirectional | 3G-SDI |
| MediorNet Links | 8 | SFP+ | Bidirectional | 10G Fiber Link |
| Sync I/O | 2 | BNC 75Ω | Bidirectional | BB / Tri-Level |
| LAN | 2 | RJ45 | Bidirectional | Control IP |

#### **Power**

* Connector: Dual IEC C14  
* Redundant: Yes

By clustering multiple MicroN nodes, facilities can build a "virtual router" that has no single point of failure and can be physically distributed across different floors or buildings.

### **Riedel MediorNet FusioN Node**

The FusioN is a compact, SFP-based node designed to be mounted directly behind displays or integrated into racks where space is at a premium.

* **Category:** Media Network Node  
* **Protocols:** ST 2110, ST 2022-6, HDMI (via SFP)  
* **Special Features:** PoE powered, miniature form factor, software-defined functionality.

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Media Ports | 2 | SFP+ | Bidirectional | 10G / 25G IP |
| Control | 1 | RJ45 | Bidirectional | Management |

#### **Power**

* Connector: PoE+ or DC input

The FusioN is the ultimate "edge device," converting uncompressed IP streams into HDMI for monitoring or SDI for local processing.

### **Blackmagic Mini Converter SDI to Fiber**

A simple, unidirectional converter designed to extend the reach of 3G-SDI signals up to 45km using single-mode fiber.52

* **Category:** Fiber Converter  
* **Protocols:** 3G-SDI, SD-SDI, HD-SDI 52  
* **Special Features:** Automatic standard detection, support for embedded audio 52

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| SDI In | 1 | BNC 75Ω | Input | 3G-SDI 52 |
| Fiber I/O | 1 | SFP (LC) | Bidirectional | Optical SDI 52 |
| SDI Out | 1 | BNC 75Ω | Output | 3G-SDI 52 |

#### **Power**

* Connector: 12V DC Locking 53

This converter is widely used in ENG (Electronic News Gathering) and event production for simple point-to-point fiber runs.

### **AJA FiDO-4R-12G Quad 12G-SDI Fiber Receiver**

The FiDO-4R-12G is a high-performance 4-channel 12G-SDI fiber receiver, allowing for cable runs of up to 10km for single-mode fiber.54

* **Category:** Fiber Transport  
* **Protocols:** 12G-SDI, 6G-SDI, 3G-SDI, SMPTE-297 54  
* **Special Features:** 4 independent channels, compact low-profile enclosure, five-year warranty 54

#### **Physical I/O**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| Fiber Input | 2 | LC Duplex | Input | 12G Fiber (SM) 54 |
| SDI Output | 4 | BNC 75Ω | Output | 12G-SDI 55 |

#### **Power**

* Connector: 5-20V DC Locking 55  
* Power Consumption: 5 Watts Max 54

The FiDO series supports the pass-through of all SDI metadata, including HDR, making it ideal for high-end cinematic and broadcast capture workflows.54

## **Technical Synthesis and Conclusion**

The data captured in this report illustrates a clear shift toward high-density, modular, and IP-centric hardware. For SignalCanvas library development, the key differentiator remains the physical connector type and its associated electrical parameters. Design rule checking must prioritize impedance matching—such as the 50Ω vs 75Ω BNC distinction in GPS inputs—and PoE budget allocation to ensure system stability. Furthermore, the modular nature of intercom matrices like the Clear-Com Eclipse HX and Riedel Artist-1024 requires a logical framework that accounts for card-specific port capacities and backplane timeslots. By utilizing these exhaustive specifications, engineers can automate the validation of signal chains, ensuring that every connection is physically possible and electrically sound, thereby reducing commissioning time and minimizing on-site troubleshooting.

#### **Works cited**

1. SG350-28 28-Port Gigabit Switch Specs | PDF | Computer Network | Ethernet \- Scribd, accessed March 4, 2026, [https://www.scribd.com/document/524223288/sg350-28-datasheet](https://www.scribd.com/document/524223288/sg350-28-datasheet)  
2. SG550X-48 Datasheet \- Router-Switch.com, accessed March 4, 2026, [https://www.router-switch.com/pdf2html/pdf/sg550x-48-datasheet.pdf](https://www.router-switch.com/pdf2html/pdf/sg550x-48-datasheet.pdf)  
3. Cisco 550X Series SG550X-48MP | Overview, Specs, Details | SHI, accessed March 4, 2026, [https://www.shi.com/product/33523818/Cisco-550X-Series-SG550X-48MP](https://www.shi.com/product/33523818/Cisco-550X-Series-SG550X-48MP)  
4. sg350x 48 \- cisco \- device.report, accessed March 4, 2026, [https://device.report/cisco/sg350x-48](https://device.report/cisco/sg350x-48)  
5. Cisco SG550X-48 48-Port Gigabit Stackable Managed Switch \- SecureITStore.com, accessed March 4, 2026, [https://www.secureitstore.com/sg550x-48.asp](https://www.secureitstore.com/sg550x-48.asp)  
6. PRODUCT SPECIFICATIONS \- PRO MUSIC, s.r.o., accessed March 4, 2026, [https://www.promusic.cz/sites/default/files/product/field\_files/product-specification-sheet-gigacore-30i-v1.0.6.pdf](https://www.promusic.cz/sites/default/files/product/field_files/product-specification-sheet-gigacore-30i-v1.0.6.pdf)  
7. GigaCore 30i \- Luminex, accessed March 4, 2026, [https://www.luminex.be/products/gigacore/gigacore-30i/](https://www.luminex.be/products/gigacore/gigacore-30i/)  
8. USER MANUAL \- Luminex, accessed March 4, 2026, [https://www.luminex.be/wp-content/uploads/doccenter/User-Manual\_GigaCore-30i\_rev-1.0.1.pdf](https://www.luminex.be/wp-content/uploads/doccenter/User-Manual_GigaCore-30i_rev-1.0.1.pdf)  
9. User Manual GigaCore 16XT \- TSL Lighting, accessed March 4, 2026, [https://www.tsllighting.com/wp-content/uploads/2023/07/Luminex-GigaCore-16Xt-POE-User-Manual.pdf](https://www.tsllighting.com/wp-content/uploads/2023/07/Luminex-GigaCore-16Xt-POE-User-Manual.pdf)  
10. M4300 Intelligent Edge Series \- Netgear, accessed March 4, 2026, [https://www.netgear.com/jp/media/M4300\_brochure\_tcm171-74796.pdf](https://www.netgear.com/jp/media/M4300_brochure_tcm171-74796.pdf)  
11. Fully Managed Switches M4300-28G \- GSM4328PB | NETGEAR, accessed March 4, 2026, [https://www.netgear.com/business/wired/switches/fully-managed/m4300-28g-poe-plus-1000w-psu/](https://www.netgear.com/business/wired/switches/fully-managed/m4300-28g-poe-plus-1000w-psu/)  
12. 7010T Gigabit Ethernet Data Center Switches \- Data Sheet \- Arista, accessed March 4, 2026, [https://www.arista.com/assets/data/pdf/Datasheets/7010T-48\_Datasheet.pdf](https://www.arista.com/assets/data/pdf/Datasheets/7010T-48_Datasheet.pdf)  
13. Arista 7010T-48 \- switch \- 48 ports \- managed \- rack-mountable \- CDW, accessed March 4, 2026, [https://www.cdw.com/product/arista-7010t-48-switch-48-ports-managed-rack-mountable/3428637](https://www.cdw.com/product/arista-7010t-48-switch-48-ports-managed-rack-mountable/3428637)  
14. Arista 7010T-48 \- Switch | Overview, Specs, Details | SHI, accessed March 4, 2026, [https://eu.shi.com/product/33021114/Arista-7010T-48-Switch](https://eu.shi.com/product/33021114/Arista-7010T-48-Switch)  
15. AJA GEN10 HD/SD/AES Sync Generator with Universal Power Supply \- B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/product/899136-REG/AJA\_gen10\_HD\_SD\_Synchronizer\_Generator\_With.html](https://www.bhphotovideo.com/c/product/899136-REG/AJA_gen10_HD_SD_Synchronizer_Generator_With.html)  
16. AJA GEN 10 HD/SD SYNC GENERATOR, SIMULTANEOUS BLACKBURST & TRI-LEVEL, OUTPUTS ASSIGNABLE \- Broadcast Depot, accessed March 4, 2026, [https://7bd.com/television/aja-gen-10-hd-sd-sync-generator-simultaneous-blackburst-tri-level-outputs-assignable.html](https://7bd.com/television/aja-gen-10-hd-sd-sync-generator-simultaneous-blackburst-tri-level-outputs-assignable.html)  
17. AJA GEN10 HD/SD Sync Generator with Universal Power Supply \- eBay, accessed March 4, 2026, [https://www.ebay.com/itm/374646832120](https://www.ebay.com/itm/374646832120)  
18. OCX HD | Master Clock | Antelope Audio | Antelope Audio, accessed March 4, 2026, [https://en.antelopeaudio.com/products/ocx-hd/](https://en.antelopeaudio.com/products/ocx-hd/)  
19. Evertz 5601MSC Master Sync Pulse Gen/Clock Rental \- PRG Gear, accessed March 4, 2026, [https://prggear.com/product/evertz-5601msc-master-sync-pulse-genclock/](https://prggear.com/product/evertz-5601msc-master-sync-pulse-genclock/)  
20. 5600MSC, 5601MSC \- AV-iQ, accessed March 4, 2026, [http://cdn-docs.av-iq.com/dataSheet/5600MSC.pdf](http://cdn-docs.av-iq.com/dataSheet/5600MSC.pdf)  
21. Master SPG/Master Clock System Including 6 Bi-level/Tri-level Sync Outputs | Evertz \- AV-iQ, accessed March 4, 2026, [https://www.av-iq.com/avcat/ctl1642/index.cfm?manufacturer=evertz\&product=5601msc](https://www.av-iq.com/avcat/ctl1642/index.cfm?manufacturer=evertz&product=5601msc)  
22. Evertz 5601MSC Master SPG Clock System \- Gear Rental Company, accessed March 4, 2026, [https://www.gearrentalcompany.com/product/evertz-5601msc-master-spg-clock-system/](https://www.gearrentalcompany.com/product/evertz-5601msc-master-spg-clock-system/)  
23. Evertz 5601MSC Master SPG Master Clock System OPTIONS \+3GTG NTP \- BS Broadcast, accessed March 4, 2026, [https://www.bsbroadcast.com/evertz-5601msc-master-spg-master-clock-system-optiones-3gtg-ntp.html](https://www.bsbroadcast.com/evertz-5601msc-master-spg-master-clock-system-optiones-3gtg-ntp.html)  
24. Tektronix SPG8000 \- Master Sync / Master Clock Reference Generator \- TEquipment, accessed March 4, 2026, [https://www.tequipment.net/Tektronix/SPG8000/Video-Signal-Generator/](https://www.tequipment.net/Tektronix/SPG8000/Video-Signal-Generator/)  
25. Tektronix® SPG8000A \- Grass Valley, accessed March 4, 2026, [https://wwwapps.grassvalley.com/docs/DataSheets/modular/tektronix/spg8000a/GVB-1-0586D-EN-DS\_Tektronix\_SPG8000A.pdf](https://wwwapps.grassvalley.com/docs/DataSheets/modular/tektronix/spg8000a/GVB-1-0586D-EN-DS_Tektronix_SPG8000A.pdf)  
26. Antelope OCX HD, Universal Master Clock \- REFLEXION-ARTS, accessed March 4, 2026, [https://www.reflexion-arts.com/en/tienda/antelope-ocx-hd/](https://www.reflexion-arts.com/en/tienda/antelope-ocx-hd/)  
27. Antelope Audio OCX HD 768 kHz HD Master Clock, accessed March 4, 2026, [https://www.proaudiosolutions.com/Antelope-Audio-OCXHD-p/ocx-hd.htm](https://www.proaudiosolutions.com/Antelope-Audio-OCXHD-p/ocx-hd.htm)  
28. Brainstorm DCD-8 Master Clock Distripalyzer \- Discontinued, accessed March 4, 2026, [https://brainstormtime.com/products/dcd-8/](https://brainstormtime.com/products/dcd-8/)  
29. Eclipse HX-Omega \- Clear-Com, accessed March 4, 2026, [https://clearcom.com/DownloadCenter/datasheets/EclipseHX/EclipseHX-Omega\_Datasheet.pdf](https://clearcom.com/DownloadCenter/datasheets/EclipseHX/EclipseHX-Omega_Datasheet.pdf)  
30. Eclipse HX-Delta \- Clear-Com, accessed March 4, 2026, [https://clearcom.com/DownloadCenter/datasheets/EclipseHX/EclipseHX-Delta\_Datasheet.pdf](https://clearcom.com/DownloadCenter/datasheets/EclipseHX/EclipseHX-Delta_Datasheet.pdf)  
31. Eclipse HX-Delta \- Clear-Com, accessed March 4, 2026, [https://www.clearcom.com/Products/Products-By-Name/Station-IC/eclipse-hx-delta](https://www.clearcom.com/Products/Products-By-Name/Station-IC/eclipse-hx-delta)  
32. Eclipse HX-Delta Lite Matrix Frame, accessed March 4, 2026, [https://base.ren-ting.com/pictures/prod\_pdf/1010121.pdf](https://base.ren-ting.com/pictures/prod_pdf/1010121.pdf)  
33. ARTIST Brochure\_05\_2024.indd \- Riedel Communications, accessed March 4, 2026, [https://www.riedel.net/fileadmin/user\_upload/800-downloads/02-Brochures/EN/ARTIST\_-\_The\_Intercom\_Brochure.pdf](https://www.riedel.net/fileadmin/user_upload/800-downloads/02-Brochures/EN/ARTIST_-_The_Intercom_Brochure.pdf)  
34. Riedel ARTIST-1024 19 Inch 2RU Artist Intercom Frame \- holds up to 10x UIC \- Universal Interface Cards \- Markertek, accessed March 4, 2026, [https://www.markertek.com/product/artist-1024/riedel-artist-1024-19-inch-2ru-artist-intercom-frame-holds-up-to-10x-uic-universal-interface-cards](https://www.markertek.com/product/artist-1024/riedel-artist-1024-19-inch-2ru-artist-intercom-frame-holds-up-to-10x-uic-universal-interface-cards)  
35. Riedel Artist 1024 \- RGEAR.com, accessed March 4, 2026, [https://www.rgear.com/riedel-artist-1024](https://www.rgear.com/riedel-artist-1024)  
36. RIEDEL » Hardware, accessed March 4, 2026, [https://www.riedel.net/en/products-solutions/intercom/artist-matrix-intercom/hardware](https://www.riedel.net/en/products-solutions/intercom/artist-matrix-intercom/hardware)  
37. Riedel ARTIST-128 Intercom Matrix Node \- ES Broadcast, accessed March 4, 2026, [https://esbroadcast.com/product/riedel-artist-128-intercom-matrix-node/](https://esbroadcast.com/product/riedel-artist-128-intercom-matrix-node/)  
38. Focusrite RedNet A16R MkII Rackmount 16x16 Dante Analog Audio Interface \- B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/product/1600657-REG/focusrite\_ar16rmkll\_16x16\_dante\_analog\_interface.html](https://www.bhphotovideo.com/c/product/1600657-REG/focusrite_ar16rmkll_16x16_dante_analog_interface.html)  
39. Focusrite RedNet A16R MkII 16x16 Dante Audio Interface \- Sweetwater, accessed March 4, 2026, [https://www.sweetwater.com/store/detail/RedNetA16RMk2--focusrite-rednet-a16r-mkii-16x16-dante-audio-interface](https://www.sweetwater.com/store/detail/RedNetA16RMk2--focusrite-rednet-a16r-mkii-16x16-dante-audio-interface)  
40. RedNet A16R MkII \- Focusrite, accessed March 4, 2026, [https://us.focusrite.com/products/rednet-a16r-mkii](https://us.focusrite.com/products/rednet-a16r-mkii)  
41. RedNet D16R MkII | Focusrite, accessed March 4, 2026, [https://us.focusrite.com/products/rednet-d16r-mkii](https://us.focusrite.com/products/rednet-d16r-mkii)  
42. PRODIGY.MC \- Modular Audio Converter \- DirectOut Technologies, accessed March 4, 2026, [https://www.directout.eu/product/prodigy-mc/](https://www.directout.eu/product/prodigy-mc/)  
43. ANDIAMO \- High-End AD/DA Converter \- DirectOut Technologies, accessed March 4, 2026, [https://www.directout.eu/product/andiamo/](https://www.directout.eu/product/andiamo/)  
44. andiamo 2.xt (src) \- SeeSound, accessed March 4, 2026, [https://seesound.es/productos/pdfs/Datasheet\_ANDIAMO2\_XT-7702-7854-6779-6781-7705-7821-5837-6803.pdf](https://seesound.es/productos/pdfs/Datasheet_ANDIAMO2_XT-7702-7854-6779-6781-7705-7821-5837-6803.pdf)  
45. D.O.TEC ANDIAMO.XT SRC, accessed March 4, 2026, [https://www.hhb.co.uk/wp-content/uploads/datasheet\_20111121142733\_38079.pdf](https://www.hhb.co.uk/wp-content/uploads/datasheet_20111121142733_38079.pdf)  
46. DirectOut ANDIAMO 2.XT & ANDIAMO 2.XT SRC (Discontinued) \- Synthax Audio UK, accessed March 4, 2026, [https://www.synthax.co.uk/directout/directout-discontinued/directout-andiamo-2xt-src/](https://www.synthax.co.uk/directout/directout-discontinued/directout-andiamo-2xt-src/)  
47. Manual | Synthax, accessed March 4, 2026, [https://synthax.hk/attachment/646/download/ferrofish-a32\_manual\_en\_v1-2.pdf](https://synthax.hk/attachment/646/download/ferrofish-a32_manual_en_v1-2.pdf)  
48. Owner's Manual \- Ferrofish, accessed March 4, 2026, [https://www.ferrofish.com/public/downloads/products/a32/A32\_manual\_ENG\_V1.3i.pdf](https://www.ferrofish.com/public/downloads/products/a32/A32_manual_ENG_V1.3i.pdf)  
49. Owner's Manual \- Ferrofish, accessed March 4, 2026, [https://www.ferrofish.com/public/downloads/products/a32/A32\_DANTE\_manual\_V1.3e.pdf](https://www.ferrofish.com/public/downloads/products/a32/A32_DANTE_manual_V1.3e.pdf)  
50. A32pro A32pro Dante \- Ferrofish, accessed March 4, 2026, [https://www.ferrofish.com/public/downloads/products/a32pro/ferrofish\_a32pro\_manual\_english\_v3.1.pdf](https://www.ferrofish.com/public/downloads/products/a32pro/ferrofish_a32pro_manual_english_v3.1.pdf)  
51. A32pro | Ferrofish USA, accessed March 4, 2026, [https://ferrofish-usa.com/a32pro\_analog\_to\_madi\_adat\_converter/](https://ferrofish-usa.com/a32pro_analog_to_madi_adat_converter/)  
52. Blackmagic Fiber Converters, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/blackmagicfiberconverters](https://www.blackmagicdesign.com/products/blackmagicfiberconverters)  
53. Professional Video Converter Accessories \- B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/buy/Accessories-for-Converters/ci/6246/N/4028759643](https://www.bhphotovideo.com/c/buy/Accessories-for-Converters/ci/6246/N/4028759643)  
54. FiDO-4R \- 4-Channel Single-Mode LC Fiber to 3G-SDI Receiver \- AJA Video, accessed March 4, 2026, [https://www.aja.com/products/fido-4r](https://www.aja.com/products/fido-4r)  
55. FiDO-4R-MM \- 4-Channel Multi-Mode LC Fiberto 3G-SDI Receiver \- AJA, accessed March 4, 2026, [https://www.aja.com/products/fido-4r-mm](https://www.aja.com/products/fido-4r-mm)  
56. FiDO-4T-MM \- 4-Channel 3G-SDI to Multi-Mode LC Fiber Transmitter \- AJA Video, accessed March 4, 2026, [https://www.aja.com/products/fido-4t-mm](https://www.aja.com/products/fido-4t-mm)