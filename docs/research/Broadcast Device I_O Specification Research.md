# **Technical Architectural Review of Broadcast and Live Production Hardware for SignalCanvas Library Integration**

The current landscape of broadcast engineering is defined by a rigorous transition toward high-density ![][image1]G-SDI baseband systems and the concurrent adoption of SMPTE ST 2110 IP-based infrastructures. As production environments move from traditional studio settings to highly mobile, decentralized, and hyperconverged architectures, the requirement for precise signal flow documentation becomes paramount. The SignalCanvas library aims to codify the physical and electrical characteristics of industry-leading hardware to facilitate automated design rule checking (DRC) and sophisticated routing logic. This transition necessitates a deep understanding of the physical I/O layer, where the mechanical connector meets the electrical protocol and the software-defined license.1

In contemporary system design, a port is no longer merely a physical BNC or SFP cage; it represents a gateway into a sophisticated processing ecosystem. For instance, a single ![][image1]G-SDI port on a Ross Ultrix router can serve as a simple input, a frame-synchronized portal, or part of a multi-link 4K gearbox, depending on the software keys applied to the frame.3 Consequently, the documentation must capture not just the label on the rear panel, but the operational potential of each interface. This report provides an exhaustive technical analysis of thirty-two critical devices across the broadcast spectrum, detailing their physical I/O, audio capabilities, network dependencies, and power requirements to ensure the highest fidelity for the SignalCanvas standard library.1

## **Studio and Field Camera Systems**

The camera system serves as the primary acquisition point in the signal chain. Modern studio cameras have evolved beyond simple video captures; they are now complex network nodes capable of bidirectional data transport for video, audio, telemetry, and control. The primary interface for these systems remains the SMPTE 311M hybrid fiber-optic cable, which provides both high-bandwidth data transmission via glass fibers and DC power via copper conductors.6

### **Sony HDC-3500 and HDC-5500**

The Sony HDC series represents the industry standard for high-end live production. The HDC-3500 is a ![][image2]\-inch 4K CMOS studio camera utilizing global shutter technology, which is essential for live sports to prevent the temporal distortion associated with rolling shutters. Its sibling, the HDC-5500, extends these capabilities with Ultra High Bitrate (UHB) transmission, allowing for high-frame-rate (HFR) acquisition and dual 4K signal lines over a single SMPTE fiber cable.6

The physical I/O of the camera head is modular. The exchangeable side panel, such as the HKC-CN50, allows the camera to be adapted for different transmission environments, including digital triax or wireless RF. However, the standard fiber interface is the most prevalent for fixed studio and OB truck installations.9

## **Sony HDC-3500**

* **Category:** Camera  
* **Video Formats:** 1080i, 1080p, 720p, 2160p (via license)  
* **IP Protocols:** Network Trunk (Proprietary Sony over Fiber)

### **Video I/O (Camera Head)**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI 1 | 1 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| SDI 2 | 1 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| SDI MONI | 1 | BNC 75Ω | output | HD-SDI | 1080i/p |
| TEST OUT | 1 | BNC 75Ω | output | Analog Composite | SD |
| PROMPTER | 1 | BNC 75Ω | input | Genlock/SDI | Analog/HD-SDI |
| CCU | 1 | LEMO SMPTE 304M | bidirectional | Hybrid Fiber | Multiplexed |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| MIC IN 1 | 1 | XLR-3F | input | Mic/Line (+48V) | 1 |
| MIC IN 2 | 1 | XLR-3F | input | Mic/Line (+48V) | 1 |
| INTERCOM 1 | 1 | XLR-5F | bidirectional | Analog | 2 (ENG/PROD) |
| INTERCOM 2 | 1 | XLR-5F | bidirectional | Analog | 2 (ENG/PROD) |
| EARPHONE | 1 | Mini Jack | output | Analog | 1 Stereo |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| PROMPTER/GENLOCK | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | Network Trunk |
| REMOTE | 1 | 8-pin | bidirectional | RCP/MSU Control |
| LENS | 1 | 12-pin | bidirectional | Lens Data/Control |

### **Power**

* Connector: XLR-4M (Local DC) or SMPTE Fiber (CCU)  
* Redundant: No (Switchable between Local and CCU) 6

## **Sony HDC-5500**

* **Category:** Camera  
* **Video Formats:** 1080i, 1080p, 2160p, 720p  
* **IP Protocols:** Network Trunk (1 Gbps)

### **Video I/O (Camera Head)**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI 1 | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| SDI 2 | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| SDI MONI | 1 | BNC 75Ω | output | HD-SDI | 1080i/p |
| TEST OUT | 1 | BNC 75Ω | output | Analog | SD Composite |
| PROMPTER | 1 | BNC 75Ω | input | Genlock/SDI | Analog/HD-SDI |
| CCU | 1 | LEMO SMPTE 304M | bidirectional | UHB Fiber | Multiplexed |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| MIC IN 1 | 1 | XLR-3F | input | Mic/Line (+48V) | 1 |
| MIC IN 2 | 1 | XLR-3F | input | Mic/Line (+48V) | 1 |
| INTERCOM 1 | 1 | XLR-5F | bidirectional | Analog | 2 |
| EARPHONE | 1 | Mini Jack | output | Analog | Stereo |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| GENLOCK | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | 1 Gbps Trunk |
| REMOTE | 1 | 8-pin | bidirectional | RCP/MSU Control |
| EXT I/O | 1 | 6-pin | bidirectional | Tally/Telemetry |

### **Power**

* Connector: XLR-4M / SMPTE Fiber  
* Redundant: No 6

The integration of the camera head with the Camera Control Unit (CCU) is the primary concern for signal flow designers. The HDCU-3500 and HDCU-5500 act as the base station hub, providing the actual video breakout used by production switchers and routers. These units handle the de-multiplexing of the fiber signal into discrete baseband or IP streams.11

### **Sony HDCU-3500 and HDCU-5500**

The Sony HDCU-3500 is a half-rack CCU that provides standard ![][image3]G-SDI outputs, but can be upgraded with the HZCU-UHD35 license to enable ![][image1]G-SDI and ![][image4]K output. It is natively compatible with the HDC-3500 and supports legacy HDC-2500 series cameras. The HDCU-5500 is the full-capability base station for the flagship camera, supporting the Ultra High Bitrate transmission required for ![][image4]K HFR workflows.6

## **Sony HDCU-3500**

* **Category:** Camera Control Unit  
* **Video Formats:** 1080i, 1080p, 2160p, 720p, 480i  
* **IP Protocols:** SMPTE ST 2110 (via HKCU-SFP50 kit)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| CAMERA | 1 | SMPTE Fiber | bidirectional | 311M Fiber | Multiplexed |
| UHD SDI A | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| UHD SDI B | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| UHD SDI C | 1 | BNC 75Ω | bidirectional | 12G-SDI | up to 2160p60 |
| UHD SDI D | 1 | BNC 75Ω | bidirectional | 12G-SDI | up to 2160p60 |
| SDI OUT 1-4 | 4 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| SDI I/O 1-4 | 4 | BNC 75Ω | bidirectional | 3G-SDI | up to 1080p60 |
| SDI RET 1-4 | 4 | BNC 75Ω | input | 3G-SDI | Return Video |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AUDIO OUT CH1 | 1 | XLR-3M | output | Analog Line | 1 |
| AUDIO OUT CH2 | 1 | XLR-3M | output | Analog Line | 1 |
| AES/EBU | 1 | BNC 75Ω | output | AES3 | 2 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| REF OUT | 1 | BNC 75Ω | output | Loop-through |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN-COM | 1 | RJ45 | bidirectional | Management |
| NETWORK TRUNK | 1 | RJ45 | bidirectional | 1 Gbps Camera Data |
| RCP/CNU | 1 | 8-pin | bidirectional | Panel Control |
| TRUNK | 1 | 12-pin | bidirectional | Serial Data |

### **Power**

* Connector: IEC C14  
* Redundant: No 11

## **Sony HDCU-5500**

* **Category:** Camera Control Unit  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** SMPTE ST 2110 (Standard)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| CAMERA | 1 | SMPTE Fiber | bidirectional | UHB Fiber | UHD / HD |
| 12G-SDI OUT | 4 | BNC 75Ω | output | 12G-SDI | 2160p60 |
| 3G-SDI OUT | 4 | BNC 75Ω | output | 3G-SDI | 1080p60 |
| SDI RET 1-4 | 4 | BNC 75Ω | input | 3G-SDI | Return feeds |
| SFP+ | 2 | SFP+ Cages | bidirectional | ST 2110 / SDI | UHD / HD |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AUDIO OUT 1-2 | 2 | XLR-3M | output | Analog Line | 1 per port |
| INTERCOM | 1 | D-sub 50-pin | bidirectional | Analog/Tally | Multi-channel |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| REF OUT | 1 | BNC 75Ω | output | Loop-through |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | Management |
| REMOTE | 1 | 8-pin | bidirectional | RCP Control |

### **Power**

* Connector: IEC C14  
* Redundant: No 6

### **Robotic and PTZ Camera Innovation**

Robotic cameras, specifically Pan-Tilt-Zoom (PTZ) models, have moved from security applications into the broadcast tier. The Sony FR7 and Panasonic AW-UE160 represent the absolute peak of this category. The FR7 is unique for its full-frame sensor and interchangeable E-mount lenses, while the AW-UE160 is the first PTZ to feature a ST 2110 interface and a high-speed ![][image5]fps HD output, which is invaluable for capturing dynamic movement in sports or live events.13

## **Sony FR7**

* **Category:** PTZ Camera  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** NDI|HX, RTSP, SRT

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI OUT | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| HDMI OUT | 1 | HDMI 2.0 | output | HDMI | up to 2160p60 |
| OPTICAL OUT | 1 | SFP+ Cage | output | Fiber SDI | up to 2160p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AUDIO IN | 1 | XLR-5F | input | Balanced Mic/Line | 2 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| TC IN | 1 | BNC 75Ω | input | LTC Timecode |
| GENLOCK | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | PoE++ / Control |
| OPTICAL | 1 | SFP+ | bidirectional | SFP for IP/Video |

### **Power**

* Connector: Barrel (19.5V DC) or PoE++ (802.3bt)  
* Redundant: No 14

## **Panasonic AW-UE160**

* **Category:** PTZ Camera  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** NDI High Bandwidth, NDI|HX2, SRT, SMPTE ST 2110

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI OUT | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| 3G-SDI OUT 1 | 1 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| 3G-SDI OUT 2 | 1 | BNC 75Ω | output | 3G-SDI | Monitor / Return |
| HDMI OUT | 1 | HDMI 2.0 | output | HDMI | up to 2160p60 |
| SFP+ | 1 | SFP+ Cage | bidirectional | ST 2110 / SDI | up to 2160p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| INPUT 1 | 1 | XLR-3F | input | Balanced (+48V) | 1 |
| INPUT 2 | 1 | XLR-3F | input | Balanced (+48V) | 1 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| G/L IN | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | PoE++ / Control |
| SERIAL | 1 | RJ45 | input | RS-422 Control |
| USB 3.0 | 1 | USB-A | bidirectional | 5G Modem Tethering |

### **Power**

* Connector: XLR-4M (12V DC) or PoE++  
* Redundant: No 13

## **Sony BRC-X1000**

* **Category:** PTZ Camera  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** RTSP, RTMP

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 3G-SDI OUT | 2 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| HDMI OUT | 1 | HDMI | output | HDMI | up to 2160p30 |
| MON OUT | 2 | BNC 75Ω | output | Analog Composite | SD |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AUDIO IN | 2 | Mini-jack | input | Mic/Line | 1 per port |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| SYNC IN | 1 | BNC 75Ω | input | Black Burst |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | Control/Streaming |
| VISCA RS-422 | 2 | RJ45 | bidirectional | Serial Control |

### **Power**

* Connector: Barrel (12V DC)  
* Redundant: No 14

The Grass Valley LDX 100 represents a revolutionary departure from the camera-head/CCU paradigm. It is a "NativeIP" camera that does not require a base station. Instead, the camera head acts as a direct network endpoint, connecting via a ![][image6] Gbps QSFP or SFP28 link. This architecture eliminates rack space in OB trucks and facilitates a truly distributed production environment where a camera can be plugged into any network port in a facility.17

## **Grass Valley LDX 100**

* **Category:** Camera  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** SMPTE ST 2110, AMWA NMOS

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| IP LINK | 1 | QSFP28 | bidirectional | ST 2110 (100G) | up to 2160p180 |
| SDI OUT | 2 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| REF/SYNC | 1 | BNC 75Ω | input | Tri-Level / BB | Analog Sync |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| MIC IN | 2 | XLR-3F | input | Balanced Mic | 1 per port |
| INTERCOM | 1 | XLR-5F | bidirectional | Analog | 2 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level / PTP |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | Management/NMOS |
| REMOTE | 1 | 8-pin | bidirectional | OCP Control |

### **Power**

* Connector: XLR-4M (12V DC)  
* Redundant: No 17

## **Production Switchers and Hyperconverged Processing**

The production switcher is the emotional and technical heart of a live show. Traditionally a discrete "Mix-Effects" (ME) engine, modern switchers like the Ross Carbonite Ultra and Grass Valley K-Frame have absorbed many other roles, including multiviewing, format conversion, and sophisticated internal routing. The Ross Carbonite series, in particular, has mastered the mid-range market by offering a ![][image7]RU chassis that packs the processing power of much larger legacy frames.2

## **Ross Carbonite Ultra**

* **Category:** Video Switcher  
* **Video Formats:** 2160p, 1080p, 1080i, 720p, 480i  
* **IP Protocols:** IP-ready (via Ultrix blades)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| INPUT 1-24 | 24 | HD-BNC | input | 12G/3G-SDI | up to 2160p60 |
| OUTPUT 1-12 | 12 | HD-BNC | output | 12G/3G-SDI | up to 2160p60 |
| MV OUT 1 | 1 | HD-BNC | output | 3G-SDI | Dedicated MV |
| MV OUT 2 | 1 | HD-BNC | output | 3G-SDI | Dedicated MV |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AES OUT | 2 | BNC 75Ω | output | AES3 | 2 per port |
| RAVE (Option) | 1 | DB-25 | bidirectional | Analog/AES | 8/1 per module |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| LTC IN | 1 | BNC 75Ω | input | Linear Timecode |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | DashBoard / Panel |
| TALLY/GPIO | 1 | DB-37 | output | 24 Tally / 24 GPI |
| RS-422 | 1 | RJ45 | bidirectional | Editor Control |

### **Power**

* Connector: IEC C14  
* Redundant: Optional (External Brick) 2

## **Ross Carbonite Black Solo**

* **Category:** Video Switcher  
* **Video Formats:** 1080p, 1080i, 720p, 2160p (Ultra model)  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 6 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| HDMI IN | 3 | HDMI | input | HDMI | up to 1080p60 |
| SDI OUT | 5 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| HDMI OUT | 1 | HDMI | output | HDMI | up to 1080p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AES OUT | 1 | BNC 75Ω | output | AES3 | 2 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | Black Burst |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | Control/Web UI |
| TALLY/GPIO | 1 | DB-25 | output | Tally/GPI |

### **Power**

* Connector: 4-pin XLR (Local DC)  
* Redundant: No 2

Blackmagic Design's ATEM Constellation line has disrupted the high-end market by offering an astonishing density of ![][image1]G-SDI I/O in a ![][image8]RU frame. The ATEM Constellation 8K and its 4 M/E HD counterpart are designed for massive live events, where the ability to have every input standard-converted internally is a critical time-saver.5

## **Blackmagic ATEM Constellation 8K**

* **Category:** Video Switcher  
* **Video Formats:** 4320p, 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** N/A (SDI Centric)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 40 | BNC 75Ω | input | 12G-SDI | up to 4320p60 |
| SDI OUT | 24 | BNC 75Ω | output | 12G-SDI | up to 4320p60 |
| MV OUT | 4 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| MADI IN | 1 | BNC 75Ω | input | AES10 (MADI) | 64 |
| MADI OUT | 2 | BNC 75Ω | output | AES10 (MADI) | 64 per port |
| ANALOG IN | 2 | 1/4" TRS | input | Balanced Line | 1 per port |
| ANALOG OUT | 2 | 1/4" TRS | output | Balanced Line | 1 per port |
| TALKBACK | 1 | XLR-5F | bidirectional | Intercom | 2 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| REF OUT | 1 | BNC 75Ω | output | Loop-through |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | Management/Panel |
| RS-422 | 1 | RJ12 | bidirectional | PTZ/GVO Control |
| USB-C | 1 | USB-C | bidirectional | Webcam/Update |

### **Power**

* Connector: 2x IEC C14 (Internal Supplies)  
* Redundant: Yes 22

## **Blackmagic ATEM 4 M/E Constellation HD**

* **Category:** Video Switcher  
* **Video Formats:** 1080p, 1080i, 720p  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 40 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| SDI OUT | 24 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| MV OUT | 4 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| MADI IN | 1 | BNC 75Ω | input | AES10 (MADI) | 64 |
| MADI OUT | 2 | BNC 75Ω | output | AES10 (MADI) | 64 per port |
| ANALOG IN | 2 | 1/4" TRS | input | Balanced Line | 1 per port |
| TALKBACK | 1 | RJ45 | bidirectional | Talkback Bridge | Multi-channel |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | Control |
| USB-C | 1 | USB-C | output | Webcam Out |

### **Power**

* Connector: 2x IEC C14  
* Redundant: Yes 22

The Grass Valley K-Frame XP represents the traditional tier of major broadcast network switching. Available in standard, compact, and entry-level frames, it supports up to ![][image9] inputs and ![][image10] outputs, with a unique ability to mix SDI and IP boards within the same processing environment.18

## **Grass Valley K-Frame XP**

* **Category:** Video Switcher  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** SMPTE ST 2110, JPEG-XS

### **Video I/O (SXP Standard Frame)**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI INPUT | 192 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| SDI OUTPUT | 96 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| IP BOARD | 8 | QSFP28 | bidirectional | ST 2110 (100G) | High-density IP |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 2 | BNC 75Ω | input | Tri-Level / BB |
| PTP | 1 | SFP | input | IEEE 1588 |

### **Power**

* Connector: External Power Frame (Standard)  
* Redundant: Yes 25

## **Video Routers and Hybrid Orchestration**

The video router acts as the traffic controller for the entire facility. The industry has shifted from pure baseband matrices to hybrid and hyperconverged frames that handle SDI, IP, and MADI audio processing internally. The Ross Ultrix FR-12 and Evertz EQX are the definitive examples of this trend. For documented signal flow, it is vital to note that Ultrix SFP cages can support both optical SDI and IP video (ST 2110), making the physical cabling identical but the signal protocol fundamentally different.1

## **Ross Ultrix FR-12**

* **Category:** Video Router  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** SMPTE ST 2110, NDI (via MODX)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| HD BNC (HDX) | 16 | HD-BNC | input | 12G-SDI | up to 2160p60 |
| HD BNC (HDX) | 16 | HD-BNC | output | 12G-SDI | up to 2160p60 |
| SFP CAGE | 16 | SFP+ | bidirectional | SDI / ST 2110 | Variable |
| AUX SFP | 32 | SFP+ | bidirectional | MADI / SDI / IP | AUX Routing |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| MADI (via SFP) | 1 | SFP | bidirectional | AES10 | 64 |
| DANTE (via MODX) | 1 | SFP | bidirectional | Dante | 64 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 2 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 2 | RJ45 | bidirectional | Ultricore Control |

### **Power**

* Connector: External Ultripower 1RU  
* Redundant: Yes 1

## **Evertz EQX**

* **Category:** Video Router  
* **Video Formats:** 12G/3G/HD/SD-SDI, ASI, IP  
* **IP Protocols:** SMPTE ST 2022-6, ST 2110

### **Video I/O (16RU Frame)**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 288 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| SDI OUT | 288 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| IP GATEWAY | 1 | QSFP | bidirectional | 100GbE IP | ST 2110 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 2 | BNC 75Ω | input | Tri-Level / BB |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 2 | RJ45 | bidirectional | MAGNUM Control |
| RS-422 | 4 | D-sub 9 | bidirectional | Serial Control |

### **Power**

* Connector: External 1RU Power Frame  
* Redundant: Yes 27

## **Blackmagic Smart Videohub 40x40**

* **Category:** Video Router  
* **Video Formats:** 2160p, 1080p, 1080i, 720p, SD  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 40 | BNC 75Ω | input | 6G-SDI | up to 2160p30 |
| SDI OUT | 40 | BNC 75Ω | output | 6G-SDI | up to 2160p30 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | IP Control / PoE |
| RS-422 | 1 | RJ12 | bidirectional | Panel/Serial Control |

### **Power**

* Connector: IEC C14  
* Redundant: No 29

## **LED Video Processors**

The LED processor is the final destination for video content in most live event spaces. These devices convert standard video signals (![][image1]G-SDI, HDMI) into proprietary Ethernet-based protocols for driving LED panels. Brompton's SX40 and S8 are notable for their ![][image11]G trunks which can drive massive pixel canvases with extreme color accuracy, whereas Novastar units are the industry standard for general-purpose rental displays.30

## **Novastar NovaPro UHD Jr**

* **Category:** LED Processor  
* **Video Formats:** 2160p, 1080p, 720p  
* **Max Pixel Res:** 10.4 million pixels

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 2 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| HDMI 2.0 | 1 | HDMI 2.0 | input | HDMI | up to 2160p60 |
| DP 1.2 | 1 | DisplayPort | input | DP 1.2 | up to 2160p60 |
| DVI | 4 | DVI-D | input | Single Link | up to 1080p60 |
| MONITOR OUT | 1 | HDMI | output | HDMI | 1080p60 |
| SDI LOOP | 2 | BNC 75Ω | output | 12G-SDI | Reclocked Thru |

### **LED Output**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 16 | Neutrik RJ45 | output | 1G Sending Data |
| OPTICAL | 4 | SFP | output | 10G Fiber Trunk |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| GENLOCK IN | 1 | BNC 75Ω | input | Black Burst |
| GENLOCK LOOP | 1 | BNC 75Ω | output | Loop-through |

### **Power**

* Connector: IEC C14  
* Redundant: No 32

## **Brompton Tessera SX40**

* **Category:** LED Processor  
* **Video Formats:** 2160p, 1080p, 720p  
* **Max Pixel Res:** 9 million pixels

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| 12G-SDI THRU | 1 | BNC 75Ω | output | 12G-SDI | Reclocked Loop |
| HDMI 2.0 IN | 1 | HDMI 2.0 | input | HDMI | up to 2160p60 |

### **LED Output**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| 10G BASE-T | 4 | etherCON | output | Tessera 10G Copper |
| 10G FIBER | 4 | opticalCON DUO | output | Tessera 10G Fiber |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| SYNC IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| SYNC THRU | 1 | BNC 75Ω | output | Loop-through |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| MANAGEMENT | 2 | RJ45 | bidirectional | Tessera Remote |
| DMX IN | 1 | XLR-5M | input | Art-Net / DMX Control |

### **Power**

* Connector: IEC C14 (Switched Auto-ranging)  
* Redundant: No 30

## **Brompton Tessera S8**

* **Category:** LED Processor  
* **Video Formats:** 2160p, 1080p, 720p  
* **Max Pixel Res:** 4.5 million pixels

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| HDMI 2.0 IN | 1 | HDMI 2.0 | input | HDMI | up to 2160p60 |

### **LED Output**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| 1G BASE-T | 8 | etherCON | output | Tessera 1G Data |

### **Power**

* Connector: IEC C14  
* Redundant: No 30

## **Encoders, Decoders, and Streaming Solutions**

As live production extends beyond the local facility, encoders become the bridge to the wider world. The Haivision Makito X4 and Teradek Prism Flex are the industry gold standards for low-latency transmission over unmanaged networks via SRT, while the Kiloview N60 focuses on high-efficiency 4K NDI transport.35

## **Haivision Makito X4**

* **Category:** Encoder  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** SRT, RTMP, SMPTE ST 2110

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| 3G-SDI IN | 3 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| SFP+ | 1 | SFP+ Cage | bidirectional | ST 2110 / Fiber | Variable |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 2 | RJ45 | bidirectional | Management/Streaming |

### **Power**

* Connector: 4-pin XLR (Local DC) or Blade Chassis  
* Redundant: Yes (via 1RU chassis) 35

## **Teradek Prism Flex**

* **Category:** Encoder/Decoder  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** SRT, RTMP, NDI|HX

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| SDI OUT | 1 | BNC 75Ω | output | 12G-SDI | Reclocked Loop |
| HDMI IN | 1 | HDMI 2.0 | input | HDMI | up to 2160p60 |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 2 | RJ45 | bidirectional | Bonding / PoE+ |
| USB-C | 2 | USB-C | bidirectional | LTE Modems / Disk |

### **Power**

* Connector: 2-pin Locking Circular  
* Redundant: No 37

## **Kiloview N60**

* **Category:** Encoder/Decoder  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** NDI High Bandwidth, NDI|HX3, SRT

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| 12G-SDI OUT | 1 | BNC 75Ω | output | 12G-SDI | Loop-through |
| HDMI IN | 1 | HDMI 2.0 | input | HDMI | up to 2160p60 |
| HDMI OUT | 1 | HDMI 2.0 | output | HDMI | up to 2160p60 |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | 10GbE / PoE+ |
| USB 3.0 | 1 | USB-A | bidirectional | KVM / Control |

### **Power**

* Connector: DC Barrel / PoE+  
* Redundant: No 32

## **Magewell Ultra Encode 4K Plus**

* **Category:** Encoder  
* **Video Formats:** 2160p, 1080p, 720p  
* **IP Protocols:** NDI|HX3, SRT, RTMP

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| HDMI IN | 1 | HDMI 2.0 | input | HDMI | up to 2160p30 |
| HDMI LOOP | 1 | HDMI 2.0 | output | HDMI | Pass-thru |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | PoE / Streaming |
| USB 3.0 | 1 | USB-A | output | External Recording |

### **Power**

* Connector: DC Barrel / PoE  
* Redundant: No 41

## **Multiviewers and Specialized Monitoring**

The multiviewer allows a single operator to see dozens of sources at once. While internal multiviewers are common in switchers, dedicated hardware like the Decimator DMON-16SL and Blackmagic MultiView 16 remain essential for larger facility-wide monitoring where independent scaling and label control are required.29

## **Ross Tessera MV (openGear)**

* **Category:** Multiviewer  
* **Video Formats:** 1080p, 1080i, 720p  
* **IP Protocols:** N/A (Standard)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 4 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| SDI OUT | 1 | BNC 75Ω | output | 3G-SDI | Multiviewer |
| HDMI OUT | 1 | HDMI | output | HDMI | Multiviewer |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | DashBoard Control |

### **Power**

* Connector: openGear Frame  
* Redundant: Yes (via Frame) 44

## **Decimator DMON-16SL**

* **Category:** Multiviewer  
* **Video Formats:** 1080p, 1080i, 720p, SD  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 16 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| SDI OUT | 1 | BNC 75Ω | output | 3G-SDI | Multiviewer |
| HDMI OUT | 1 | HDMI | output | HDMI | Multiviewer |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| USB | 1 | Mini-USB | bidirectional | Software Control |

### **Power**

* Connector: Threaded DC (5V-32V)  
* Redundant: No 42

## **Blackmagic MultiView 16**

* **Category:** Multiviewer  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 16 | BNC 75Ω | input | 6G-SDI | up to 2160p30 |
| SDI LOOP | 16 | BNC 75Ω | output | 6G-SDI | Reclocked Thru |
| HD-SDI OUT | 2 | BNC 75Ω | output | HD-SDI | Multiviewer |
| 6G-SDI OUT | 2 | BNC 75Ω | output | 6G-SDI | Multiviewer |
| HDMI OUT | 1 | HDMI | output | HDMI | Multiviewer |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | Control / Updates |
| RS-422 | 1 | RJ12 | bidirectional | Protocol Control |

### **Power**

* Connector: IEC C14  
* Redundant: No 29

## **Recording and High-Speed Playback**

Modern recorders have evolved into sophisticated file servers. The Blackmagic HyperDeck and AJA Ki Pro series are no longer "VTRs" but network-attached storage (NAS) devices that can record ![][image12]K video or four simultaneous HD streams. For SignalCanvas, it is critical to distinguish between the ![][image11]G Ethernet used for media transport and the RS-422 used for legacy deck control.46

## **Blackmagic HyperDeck Studio 4K Pro**

* **Category:** Recorder  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** FTP, HyperDeck Protocol

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| 12G-SDI OUT | 2 | BNC 75Ω | output | 12G-SDI | Main / Fill-Key |
| SDI MONITOR | 1 | BNC 75Ω | output | 3G-SDI | Overlays |
| HDMI IN/OUT | 2 | HDMI 2.0 | bidirectional | HDMI | up to 2160p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| PHONES | 1 | 1/4" TRS | output | Analog | 1 Stereo |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| REF OUT | 1 | BNC 75Ω | output | Loop-through |
| TC IN | 1 | XLR-3F | input | LTC Timecode |
| TC OUT | 1 | XLR-3M | output | LTC Timecode |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | 10GbE / FTP |
| RS-422 IN | 1 | D-sub 9 | input | Deck Control |
| RS-422 OUT | 1 | D-sub 9 | output | Deck Control |
| USB-C | 1 | USB-C | bidirectional | Ext Disk / Update |

### **Power**

* Connector: IEC C14 and XLR-4M  
* Redundant: Yes (AC \+ DC) 47

## **Blackmagic HyperDeck Extreme 8K HDR**

* **Category:** Recorder  
* **Video Formats:** 4320p, 2160p, 1080p, 1080i  
* **IP Protocols:** 10GbE FTP, SMB

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 4320p60 |
| SDI OUT | 3 | BNC 75Ω | output | 12G-SDI | Main/Mon/Fill |
| HDMI 2.0 I/O | 2 | HDMI 2.0 | bidirectional | HDMI | up to 2160p60 |
| ANALOG IN | 3 | BNC 75Ω | input | Component YUV | HD/SD |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| ANALOG IN | 4 | XLR-3F | input | Balanced Line | 4 |
| ANALOG OUT | 2 | RCA | output | Unbalanced Line | 2 |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| ETHERNET | 1 | RJ45 | bidirectional | 10GbE Media |
| RS-422 | 2 | D-sub 9 | bidirectional | Deck Control |

### **Power**

* Connector: IEC C14 and XLR-4M  
* Redundant: Yes 46

## **AJA Ki Pro Ultra 12G**

* **Category:** Recorder  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** REST Interface

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| 12G-SDI IN | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| 3G-SDI IN | 3 | BNC 75Ω | input | 3G-SDI | Multi-channel HD |
| 12G-SDI OUT | 1 | BNC 75Ω | output | 12G-SDI | up to 2160p60 |
| 3G-SDI OUT | 3 | BNC 75Ω | output | 3G-SDI | Quad-link |
| HDMI I/O | 2 | HDMI 2.0 | bidirectional | HDMI | up to 2160p60 |
| SFP+ | 1 | SFP Cage | bidirectional | Fiber SDI | Optional |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| ANALOG I/O | 1 | DB-25 | bidirectional | Tascam Standard | 8 |
| AES/EBU I/O | 1 | DB-25 | bidirectional | Digital Audio | 8 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| LTC IN | 1 | BNC 75Ω | input | Timecode |

### **Power**

* Connector: 2x XLR-4M (Dual DC)  
* Redundant: Yes 48

## **Converters, Frame Synchronizers, and Standards Processors**

Converters are the "utility belt" of the broadcast world. They manage the transition between HDMI and SDI, upscale or downscale signals to match house standards, and provide frame synchronization to non-genlocked sources. The AJA FS4 and Blackmagic Teranex AV are standard equipment in any flypack or studio, providing the final layer of compatibility before a signal is mixed or broadcast.54

## **AJA FS4**

* **Category:** Frame Synchronizer / Converter  
* **Video Formats:** 2160p, 1080p, 1080i, 720p  
* **IP Protocols:** N/A (IP options via SFP)

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 4 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| SDI OUT | 4 | BNC 75Ω | output | 3G-SDI | up to 1080p60 |
| SFP I/O | 2 | SFP Cages | bidirectional | 12G Fiber / BNC | up to 2160p60 |
| MONITOR OUT | 1 | BNC 75Ω | output | 3G-SDI | Overlay Monitor |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| AES I/O | 1 | DB-25 | bidirectional | Digital Audio | 8 Pairs |
| MADI IN | 1 | BNC 75Ω | input | AES10 | 64 |
| MADI OUT | 1 | BNC 75Ω | output | AES10 | 64 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |
| REF OUT | 1 | BNC 75Ω | output | Loop-through |

### **Network/Control**

| Label | Qty | Connector | Direction | Purpose |
| :---- | :---- | :---- | :---- | :---- |
| LAN | 1 | RJ45 | bidirectional | Web UI / SNMP |
| GPI/O | 1 | DB-25 | bidirectional | Optical Isolated |

### **Power**

* Connector: 2x IEC C14  
* Redundant: Yes 56

## **Blackmagic Teranex AV**

* **Category:** Standards Converter  
* **Video Formats:** 2160p, 1080p, 1080i, 720p, 2K  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN A | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| SDI IN B | 1 | BNC 75Ω | input | 12G-SDI | up to 2160p60 |
| SDI OUT A/B | 2 | BNC 75Ω | output | 12G-SDI | Standards Conv |
| QUAD OUT | 4 | BNC 75Ω | output | 3G-SDI | Quad-link UHD |
| HDMI IN | 1 | HDMI 2.0 | input | HDMI | up to 2160p60 |
| HDMI OUT | 1 | HDMI 2.0 | output | HDMI | up to 2160p60 |

### **Audio I/O**

| Label | Qty | Connector | Direction | Signal | Channels |
| :---- | :---- | :---- | :---- | :---- | :---- |
| ANALOG IN | 2 | XLR-3F | input | Balanced Line/AES | 2 per port |
| ANALOG OUT | 2 | RCA | input | Consumer Level | 2 |

### **Sync/Reference**

| Label | Qty | Connector | Direction | Signal |
| :---- | :---- | :---- | :---- | :---- |
| REF IN | 1 | BNC 75Ω | input | BB / Tri-Level |

### **Power**

* Connector: IEC C14  
* Redundant: No 54

## **Decimator MD-HX**

* **Category:** Converter  
* **Video Formats:** 1080p, 1080i, 720p, SD  
* **IP Protocols:** N/A

### **Video I/O**

| Label | Qty | Connector | Direction | Signal | Formats |
| :---- | :---- | :---- | :---- | :---- | :---- |
| SDI IN | 1 | BNC 75Ω | input | 3G-SDI | up to 1080p60 |
| SDI OUT 1-4 | 4 | BNC 75Ω | output | 3G-SDI | Loop or Scaled |
| HDMI IN | 1 | HDMI | input | HDMI | up to 1080p60 |
| HDMI OUT | 1 | HDMI | output | HDMI | up to 1080p60 |

### **Power**

* Connector: Threaded DC Barrel (5V-32V)  
* Redundant: No 55

## **Integration Logic and Technical Synthesis**

The data presented in these tables form the mechanical and electrical baseline for the SignalCanvas library. However, the true value for design rule checking lies in understanding the bandwidth and protocol nuances of these ports. For instance, a ![][image1]G-SDI port is fundamentally different from a ![][image3]G-SDI port not just in bit rate, but in its ability to support higher color precision and ![][image4]K HDR metadata. The library must distinguish between these capabilities to prevent the routing of a ![][image1]G source to a ![][image3]G-only destination, which would result in a complete signal loss.27

Furthermore, the "Modular" nature of modern broadcast frames like the Ross Ultrix or Evertz EQX necessitates a hierarchical template structure. A router frame is not a static device; it is a container for blades. Each blade (e.g., the ULTRIX-HDX-IO) defines the physical ports, while the software licenses applied to the frame define if those ports can act as frame synchronizers or gearbox converters. SignalCanvas must therefore implement a multi-stage validation: first, physical compatibility (BNC to BNC); second, electrical compatibility (![][image1]G to ![][image1]G); and third, logical compatibility (License presence for the requested processing).1

Power redundancy also plays a critical role in high-availability environments like broadcast trucks. The presence of dual IEC inlets on Blackmagic ATEM units or the use of external rack-mount power supplies like Ross Ultripower ensures that a single circuit failure does not take down the entire production. These power dependencies must be mapped to specific PDU (Power Distribution Unit) outlets within SignalCanvas to ensure the system architect has a complete picture of the facility's resilience.1

As the industry moves closer to a software-defined future, the distinction between "video" and "data" ports will continue to blur. The inclusion of NMOS support in devices like the Grass Valley LDX 100 and Ross Ultrix allows for automated discovery and connection management, a feature that SignalCanvas can leverage to auto-generate signal paths. By maintaining this exhaustive repository of port-level specifications, SignalCanvas provides the foundational data required to build the next generation of intelligent, self-aware broadcast facilities.18

#### **Works cited**

1. Ultrix Specifications \- Ross Video, accessed March 4, 2026, [https://www.rossvideo.com/products/routing-systems/ultrix/ultrix-specifications/](https://www.rossvideo.com/products/routing-systems/ultrix/ultrix-specifications/)  
2. Ultrix Carbonite Specifications \- Ross Video, accessed March 4, 2026, [https://www.rossvideo.com/products/production-switchers/ultrix-carbonite/ultrix-carbonite-specifications/](https://www.rossvideo.com/products/production-switchers/ultrix-carbonite/ultrix-carbonite-specifications/)  
3. ROSS ULTRIX-FR12 Ultrix 12RU frame. Requires minimum 2x Ultripower. Requires Ultricore-BCS. | Standalone Matrices \- DigiNet, accessed March 4, 2026, [https://shop.diginet.pro/en-us/ross-ultrix-fr12-ultrix-12ru-frame-requires-minimu-272594](https://shop.diginet.pro/en-us/ross-ultrix-fr12-ultrix-12ru-frame-requires-minimu-272594)  
4. Ultrix I/O Cards \- Ross Video, accessed March 4, 2026, [https://www.rossvideo.com/products/routing-systems/ultrix-io-cards/](https://www.rossvideo.com/products/routing-systems/ultrix-io-cards/)  
5. ATEM Constellation 8K \- Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/atemconstellation8k](https://www.blackmagicdesign.com/products/atemconstellation8k)  
6. HDC Series HDC-5500, HDC-3500, HDC-3100, HDC ... \- Pro Sony, accessed March 4, 2026, [https://pro.sony/s3/2019/03/29101012/MK20391V2\_h.pdf](https://pro.sony/s3/2019/03/29101012/MK20391V2_h.pdf)  
7. Grass Valley LDX Camera \- EBU, accessed March 4, 2026, [https://tech.ebu.ch/docs/tech/tech3335\_s13.pdf](https://tech.ebu.ch/docs/tech/tech3335_s13.pdf)  
8. Datasheet \- AV-iQ, accessed March 4, 2026, [https://cdn-docs.av-iq.com/dataSheet/LDX%205640\_Datasheet.pdf](https://cdn-docs.av-iq.com/dataSheet/LDX%205640_Datasheet.pdf)  
9. HDC Series HDC-5500, HDC-3500, HDC-3100, HDC-3170, HDC-P50, HDC \- Pro Sony, accessed March 4, 2026, [https://pro.sony/s3/2018/09/04094441/MK20431V3\_h.pdf](https://pro.sony/s3/2018/09/04094441/MK20431V3_h.pdf)  
10. Sony HDC-5500, HDC-3500, HDC-3100, HDC-3170, HDC-P50 Broschüre \- Teltec.de, accessed March 4, 2026, [https://teltec.de/media/pdf/49/20/8c/sony-hdc-5500-3500-3100-3170-p50-broschuere.pdf](https://teltec.de/media/pdf/49/20/8c/sony-hdc-5500-3500-3100-3170-p50-broschuere.pdf)  
11. HDCU-3500 \- Pro Sony, accessed March 4, 2026, [https://pro.sony/ue\_US/pdf/products/camera-control-unit/hdcu-3500](https://pro.sony/ue_US/pdf/products/camera-control-unit/hdcu-3500)  
12. HDC Series HDC-5500, HDC-3500, HDC-3100, HDC-3170, HDC-P50, accessed March 4, 2026, [https://pro.sony/s3/2018/09/22113232/MK20416V1\_h.pdf](https://pro.sony/s3/2018/09/22113232/MK20416V1_h.pdf)  
13. AW-UE160 \- Panasonic Connect, accessed March 4, 2026, [https://latam.connect.panasonic.com/mx/es/download-specifications/136](https://latam.connect.panasonic.com/mx/es/download-specifications/136)  
14. Compare BirdDog XL Ultra vs Sony FR7 vs Panasonic AW-UE160 vs Sony BRC-X1000, accessed March 4, 2026, [https://www.bhphotovideo.com/c/compare/BirdDog\_XL+Ultra\_vs\_Sony\_FR7\_vs\_Panasonic\_AW-UE160\_vs\_Sony\_BRC-X1000/BHitems/1918609-REG\_1724296-REG\_1860452-REG\_1303346-REG](https://www.bhphotovideo.com/c/compare/BirdDog_XL+Ultra_vs_Sony_FR7_vs_Panasonic_AW-UE160_vs_Sony_BRC-X1000/BHitems/1918609-REG_1724296-REG_1860452-REG_1303346-REG)  
15. Specifications | AW-UE160W/K | PTZ Camera Systems | Broadcast ..., accessed March 4, 2026, [https://pro-av.panasonic.net/en/products/aw-ue160/spec.html](https://pro-av.panasonic.net/en/products/aw-ue160/spec.html)  
16. Compare Panasonic AW-UE160 vs Sony BRC-X1000 | B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/compare/Panasonic\_AW-UE160\_vs\_Sony\_BRC-X1000/BHitems/1731534-REG\_1303346-REG](https://www.bhphotovideo.com/c/compare/Panasonic_AW-UE160_vs_Sony_BRC-X1000/BHitems/1731534-REG_1303346-REG)  
17. Grass Valley LDX 100 IP UHD Live Production Camera \- ES Broadcast, accessed March 4, 2026, [https://esbroadcast.com/product/grass-valley-ldx-100-ip-uhd-live-production-camera/](https://esbroadcast.com/product/grass-valley-ldx-100-ip-uhd-live-production-camera/)  
18. Grass Valley 2020 news \- LDX 100, K-Frame XP, AMP \- Hannu Pro (TV, playout and broadcast solutions), accessed March 4, 2026, [https://www.hannu-pro.com/info\_GV2020\_en.html](https://www.hannu-pro.com/info_GV2020_en.html)  
19. LDX 100 CAMERA SERIES \- Grass Valley, accessed March 4, 2026, [https://wwwapps.grassvalley.com/docs/DataSheets/cameras/ldx\_100/LDX\_100\_Series\_DS-PUB-3-0900E-EN.pdf](https://wwwapps.grassvalley.com/docs/DataSheets/cameras/ldx_100/LDX_100_Series_DS-PUB-3-0900E-EN.pdf)  
20. Carbonite Ultra Specifications \- Ross Video, accessed March 4, 2026, [https://www.rossvideo.com/products/production-switchers/carbonite-ultra/carbonite-ultra-specifications/](https://www.rossvideo.com/products/production-switchers/carbonite-ultra/carbonite-ultra-specifications/)  
21. Carbonite Ultra Solo | — Ross Video, accessed March 4, 2026, [https://www.rossvideo.com/products/production-switchers/carbonite-ultra-solo/](https://www.rossvideo.com/products/production-switchers/carbonite-ultra-solo/)  
22. ATEM Constellation 8K – Tech Specs | Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/atemconstellation8k/techspecs](https://www.blackmagicdesign.com/products/atemconstellation8k/techspecs)  
23. ATEM Constellation 8K – Tech Specs | Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/atemconstellation8k/techspecs/W-ATC-03](https://www.blackmagicdesign.com/products/atemconstellation8k/techspecs/W-ATC-03)  
24. ATEM Constellation 8K \- Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/atemconstellation8k/design](https://www.blackmagicdesign.com/products/atemconstellation8k/design)  
25. K-Frame Production Switchers | High-Performance Video Switching \- Grass Valley, accessed March 4, 2026, [https://www.grassvalley.com/products/production-switchers-hardware/k-frame-production-switchers/](https://www.grassvalley.com/products/production-switchers-hardware/k-frame-production-switchers/)  
26. GV K-Frame XP \- Hannu Pro, accessed March 4, 2026, [https://www.hannu-pro.com/all\_imagesxtra/all\_news/GV\_K-Frame\_XP-EN.pdf](https://www.hannu-pro.com/all_imagesxtra/all_news/GV_K-Frame_XP-EN.pdf)  
27. EQX16 \- 288x288 \- Enterprise Hybrid Video / Audio / IP Router in 16RU \- Evertz, accessed March 4, 2026, [https://evertz.com/products/EQX16](https://evertz.com/products/EQX16)  
28. EQX Router | Solutions by Platform \- Evertz, accessed March 4, 2026, [https://evertz.com/solutions/eqx/](https://evertz.com/solutions/eqx/)  
29. MultiView – Tech Specs \- Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/multiview/techspecs](https://www.blackmagicdesign.com/products/multiview/techspecs)  
30. Tessera LED Processors \- Brompton Technology, accessed March 4, 2026, [https://www.bromptontech.com/online-help/Content/Tessera%20User%20Manual/02.%20Introduction%20Topics/02%20-%20General%20Overview.htm](https://www.bromptontech.com/online-help/Content/Tessera%20User%20Manual/02.%20Introduction%20Topics/02%20-%20General%20Overview.htm)  
31. TESSERA SX40 LED PROCESSOR \- Brompton Technology, accessed March 4, 2026, [https://www.bromptontech.com/wp-content/uploads/2019/05/Brompton-SX40-Data-Sheet-May-2019.pdf](https://www.bromptontech.com/wp-content/uploads/2019/05/Brompton-SX40-Data-Sheet-May-2019.pdf)  
32. NOVASTAR NOVAPRO UHD Jr \- ADJ, accessed March 4, 2026, [https://www.adj.com/products/novapro-uhd-jr](https://www.adj.com/products/novapro-uhd-jr)  
33. TESSERA SX40 LED PROCESSOR \- Brompton Technology, accessed March 4, 2026, [https://www.bromptontech.com/wp-content/uploads/2024/09/Brompton-SX40-Data-Sheet-Sep2024-EN.pdf](https://www.bromptontech.com/wp-content/uploads/2024/09/Brompton-SX40-Data-Sheet-Sep2024-EN.pdf)  
34. NovaPro UHD Jr \- Global leading LED display control solution, accessed March 4, 2026, [https://www.novastar.tech/product/detail.html?catid=3\&id=36](https://www.novastar.tech/product/detail.html?catid=3&id=36)  
35. Makito X4 Series \- Milexia Group, accessed March 4, 2026, [https://milexia.com/products/wp-content/uploads/sites/7/2022/12/haivision-makito-x4-series-datasheets.pdf](https://milexia.com/products/wp-content/uploads/sites/7/2022/12/haivision-makito-x4-series-datasheets.pdf)  
36. Makito X4 Ultra-Low Latency Video Decoder \- Haivision, accessed March 4, 2026, [https://www.haivision.com/products/makito-x4-video-decoder/](https://www.haivision.com/products/makito-x4-video-decoder/)  
37. Prism Flex Mk I (2021-2024) \- Teradek User Guide, accessed March 4, 2026, [https://guide.teradek.com/a/1536446-prism-flex-mk-i-2021-2024](https://guide.teradek.com/a/1536446-prism-flex-mk-i-2021-2024)  
38. Prism Flex Mk II: Portable 4K HEVC Video Encoder/Decoder \- Teradek User Guide, accessed March 4, 2026, [https://guide.teradek.com/a/1862550-prism-flex-mk-ii-portable-4k-hevc-video-encoder-decoder](https://guide.teradek.com/a/1862550-prism-flex-mk-ii-portable-4k-hevc-video-encoder-decoder)  
39. Makito X4 Ultra-Low Latency Video Encoder \- Haivision, accessed March 4, 2026, [https://www.haivision.com/products/makito-x4-video-encoder/](https://www.haivision.com/products/makito-x4-video-encoder/)  
40. Prism Flex 4K HDR Streaming for Live Production \- Teradek, accessed March 4, 2026, [https://teradek.com/pages/prism-flex](https://teradek.com/pages/prism-flex)  
41. 4K Live Media Streaming Encoders \- Magewell, accessed March 4, 2026, [https://www.magewell.com/ultra-encode](https://www.magewell.com/ultra-encode)  
42. Decimator Design 12G-CROSS Cross Converter With Sync Rentals \- Rentex, accessed March 4, 2026, [https://www.rentex.com/rental-products/decimator-design-12g-cross/](https://www.rentex.com/rental-products/decimator-design-12g-cross/)  
43. Blackmagic MultiView 16 \- CPL, accessed March 4, 2026, [https://www.cpl.tech/wp-content/uploads/2018/11/Blackmagic-Multiview-16-Spec-Sheet.pdf](https://www.cpl.tech/wp-content/uploads/2018/11/Blackmagic-Multiview-16-Spec-Sheet.pdf)  
44. Video Multiviewer / Quad-Split (Model: OG-MicroQ) \- openGear.tv, accessed March 4, 2026, [https://www.opengear.tv/card/og-microq/](https://www.opengear.tv/card/og-microq/)  
45. 9x1, 9x2, 16x2 3G-SDI Video Multiviewers (Models: OG-Mi, OG-Mi+, OG-Mi\#) \- openGear.tv, accessed March 4, 2026, [https://www.opengear.tv/card/og-mi/](https://www.opengear.tv/card/og-mi/)  
46. Blackmagic Design HyperDeck Extreme 4K HDR \- Sweetwater, accessed March 4, 2026, [https://www.sweetwater.com/store/detail/HyperDeckEx4k--blackmagic-design-hyperdeck-extreme-4k-hdr](https://www.sweetwater.com/store/detail/HyperDeckEx4k--blackmagic-design-hyperdeck-extreme-4k-hdr)  
47. HyperDeck Studio – Tech Specs | Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/hyperdeckstudio/techspecs](https://www.blackmagicdesign.com/products/hyperdeckstudio/techspecs)  
48. AJA KIPRO ULTRA 12G 4K/UHD/2K/HD recorder/player with 12G I/O and multi-channel encoding support \- DigiNet, accessed March 4, 2026, [https://shop.diginet.pro/en-us/aja-kipro-ultra-12g-4k-3-uhd-3-2k-3-hd-recorder-3-142762](https://shop.diginet.pro/en-us/aja-kipro-ultra-12g-4k-3-uhd-3-2k-3-hd-recorder-3-142762)  
49. AJA Ki Pro Ultra 12G DCI/UHD/HD Recorder and Player (SDI, HDMI) \- B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/product/1567409-REG/aja\_ki\_pro\_ult\_12g\_ki\_pro\_ultra\_12g.html](https://www.bhphotovideo.com/c/product/1567409-REG/aja_ki_pro_ult_12g_ki_pro_ultra_12g.html)  
50. Blackmagic Design HyperDeck Studio 4K Pro \- B\&H Photo, accessed March 4, 2026, [https://www.bhphotovideo.com/c/product/1657121-REG/blackmagic\_design\_hyperdeck\_studio\_4k\_pro.html](https://www.bhphotovideo.com/c/product/1657121-REG/blackmagic_design_hyperdeck_studio_4k_pro.html)  
51. HyperDeck Extreme 8K HDR \- Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/hyperdeckextreme/techspecs](https://www.blackmagicdesign.com/products/hyperdeckextreme/techspecs)  
52. HyperDeck Extreme | Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/hyperdeckextreme](https://www.blackmagicdesign.com/products/hyperdeckextreme)  
53. AJA Ki Pro Ultra 12G Digital Recorder and Player | Sweetwater, accessed March 4, 2026, [https://www.sweetwater.com/store/detail/KiProUlt--aja-ki-pro-ultra-12g-digital-recorder-and-player](https://www.sweetwater.com/store/detail/KiProUlt--aja-ki-pro-ultra-12g-digital-recorder-and-player)  
54. Teranex Standards Converters – Tech Specs \- Blackmagic Design, accessed March 4, 2026, [https://www.blackmagicdesign.com/products/teranex/techspecs](https://www.blackmagicdesign.com/products/teranex/techspecs)  
55. DECIMATOR MD-HX Miniature HDMI/SDI Cross Converter \- Filmtools, accessed March 4, 2026, [https://www.filmtools.com/decimator-md-hx-miniature-hdmi-sdi-cross-converter-with-scaling-frame-rate-conversion.html](https://www.filmtools.com/decimator-md-hx-miniature-hdmi-sdi-cross-converter-with-scaling-frame-rate-conversion.html)  
56. FS4 \- AJA, accessed March 4, 2026, [https://www.aja.com/products/fs4/spec-sheet.pdf](https://www.aja.com/products/fs4/spec-sheet.pdf)  
57. AJA FS4 4-Channel 2K/HD/SD or 1-Channel 4K/UltraHD Frame Synchronizer and Up/Down Cross-Converter \- Markertek, accessed March 4, 2026, [https://www.markertek.com/product/aja-fs4/aja-fs4-4-channel-2k-hd-sd-or-1-channel-4k-ultrahd-frame-synchronizer-and-up-down-cross-converter](https://www.markertek.com/product/aja-fs4/aja-fs4-4-channel-2k-hd-sd-or-1-channel-4k-ultrahd-frame-synchronizer-and-up-down-cross-converter)  
58. MD-HX HDMI / SDI Cross Converter \- Decimator Design, accessed March 4, 2026, [https://decimator.com/Products/MiniConverters/MD-HX/MD-HX.html](https://decimator.com/Products/MiniConverters/MD-HX/MD-HX.html)

[image1]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABMAAAAYCAYAAAAYl8YPAAAAz0lEQVR4Xu2TOw5BQRiFDxJLkFiBRiIqtUJiExagUNoDi1CqLMEm1AoikhuFWufxj5k7dxzzIFHJ/ZJT3O+c/M3kAiW/YsiC6EtaLBl15Ca5c2FoQndLFLvVy0JYSE6SOfQgdIz91rgZeUvoWBfa7x1XNc63fxIqe/B3PmeJlW0WiO/jJTGC3m64yPnmWHKbHAgV6M2EC+aTY6ofsPSROqZetUPuSt+W2LGGZMcSgX0d4WM1FB1H/VqWseQsOUoOJpnk4mymeD+SZ+3sSv6CByRlS1GyUD69AAAAAElFTkSuQmCC>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB0AAAAXCAYAAAD3CERpAAABWElEQVR4Xu2TzytEURTHD8ofYCELS0uRvWRFFsJOYm87FrKzVJY2Fn7lT1AWspYkykZRJAkrkmKl+J7uO3V835k3Q2Nqyqe+Nfdz7j3v3Tf3ivzTAKyw+AnNyDjSzoUCmlhkaK9+pIULnmfkGtlHPrNxNWywAI/IAbIgqdf793JCC32B01Tig8avktYNOBf2iqS5MfKeEjJF7lTSuhHnov7yhLyQ0x3oxEnynlyjgGFJ8+64EGFv18qFDD0o9ywDwl2WQyees3RsIz0sHfawQy6UY1Hyn5up9u1vJc3t4ILnBjljScwhMywLsF0PcUHZQ3bcWK/RqBsbRbtcR+bJ2UN3ycsSskpuE+kkpwfogZzRLfHBMbfs5YSkK3KCHCHHyJXkFytbSC9Lh665CFyul8koTOQ8a5LmDCJtyFs21t+/ZprFX9PFoh5csqgHlf7PmqNXZ5Zlw/AFKLddv1bgjhIAAAAASUVORK5CYII=>

[image3]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAWCAYAAAD5Jg1dAAAAmElEQVR4XmNgGFqAG4ifA/F/IP4CxE6o0hAgDcRvkPjdDBANq5HEwOAVVCIUSQzEB2EUYATEd4CYA0kMq0J00MAAUTQZTRwOeIHYngGiaAqaHAooAuJWBojCKjQ5nADmRlCw4QV/GSAKDyILYvMhTOFLmIAIVABdIUysAFlwIlTQFcp/CuVrw1UgARYgng/Et4G4C01ueAEA3hMoKhdCWIgAAAAASUVORK5CYII=>

[image4]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAWCAYAAAD5Jg1dAAAAZ0lEQVR4XmNgGLpAEIj/owtiAyBFRCn8wkCEwn9AfIGBgMJXQMzLQEBhIBDPgLLxKkSWwKkQXRCrwjNAfBCINwDxaij+wABRCOODgT4QW6BhWDjC+DgBwQAXA+LnDAiFIPwURcWIAgAptCQbfKICXQAAAABJRU5ErkJggg==>

[image5]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB0AAAAXCAYAAAD3CERpAAABWklEQVR4Xu2ULUsEURSGjx/4AwxajBaD3S0Gv6LFoGIwG4yL4E8Rk6IGg5gEwSSIIvgBKlYxiE1EDH6g59175s6Z15ndNS47D7xwz3PP3TNwZ0ekpF2YYkGMaoZY5jCt6WDJTGi+ND+8YfRp3jWbkvYdZjoC+xL2RjSPmofsdqDfrdGcN7RL/voxc4PO3ZjzHOW4DEVDFyX4GfJwl1TfuhrMmu8kHykaCuD5juDubd1j9UG6XaNifpV8pN5QBk+O3mWr8U6g3okdAbx08NvkI/8Zyvc3b/WGcwB3Dn9OPtLs0F4JfVvOzZkrGnpHPtLM0BXNN0tlQMLZXfLD5tfIRxoNHde8kdtza5w9djWYNL9EPtJo6DMLZd2tcfbV1aBqvpB6Q080F5pTzZnmSsIXB5+7BNwnn3/RfJKr/XmfJP20JU+LH0xYcHucbtcHPjTXtsZ/M6+npKQF+QX5+GpdQSejyQAAAABJRU5ErkJggg==>

[image6]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB0AAAAXCAYAAAD3CERpAAABTUlEQVR4Xu2Uvy5EURCHhwgvYKNRahR6tkYllHgEodBotN5CtpKNzUYUKlGoUKkk/paiEJ2IhCBh5pzZM3MnM3pxv2SSOd9M7u8mu+cC1PwXZq0wNLEmrXSYx+qz0jKN9YX1bQcKmq1hLXDfqo4TB5Bn9GIPWPfVcWZE9bQchXreukvHHTuuQhTaAN/fYK2rM+1cqTOxyL7f+EIUugW+P8J6434Q8s6hjBNT7DeNL0Shp+D7Loin/wT1HRknxtnvGl+IQu/A920Qv8z9jowTY+zPjS9Eobfgex26xH0Uem18IQqlq+H5fRA/yv2ejBMT7LeNL0Shq+D7M6wndaadE3UmZtivGF+IQgnPv2LNqTPtvKgzscE+5LdQ+loNG2d36fe07hnr07h0eR9BPoG9t6VPmGYI8k7vkl9gvcu48AF5RtDdpOcNyLim5q/yA+w1aq0L6XHOAAAAAElFTkSuQmCC>

[image7]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAWCAYAAAD5Jg1dAAAAWUlEQVR4XmNgGJpAFIj70AVhIA2IXwLxfyjGqRBkCgzgVYgMBljhBHRBbACkcBK6IDYAUjgFXRAbACmcii6IDYAUTkcXhAFfIH4GxL8YELHzDYgfIysaeQAAT/4YqdrM2u0AAAAASUVORK5CYII=>

[image8]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAWCAYAAAD5Jg1dAAAAfUlEQVR4XmNgGFpACIg/AfF/IP4KxAooslBgwQBRBANfGCAaWpDEwAAkCFKMLgbCBAXPQMWWIguyATEvsgADQnM2mjgGwGYLBjjMAFHkgi6BDCSA+D26IDpgZSDCOhAAKeJG4psCcR0SHwz+ATEzmtg+INZHFoD5EBsewQAAhoghjGAKuWIAAAAASUVORK5CYII=>

[image9]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAB0AAAAXCAYAAAD3CERpAAABN0lEQVR4Xu2TP0vFQBDEFwUtLUWxtRBsbEWwEQtBxEq/gb2VnaVYvsrObyEIai2CCBZ2WllY+QdbBZ15ySWbyV5eLeYHA7ezk1tyyZn1/BfW1RDmoQ01hTFoRc2INegb+tGGg71jaK9c7zfbQ96gJ+jVigzrFtNuzVBuKP3xwNN6KfA01yAXOLTYp8cT8rXmkrclfkX0ELmw2Kd372oe6YerSfpku+JX5IYOLPbpfakppD0ntJHIDV212M/lEztW9B+04enahG806+pF684T9vS4W4za5MrqjabK9WUjUcPejJoRo4YqzC6rCc6hSVfzGm26ukHX0Gsr3s4TZY+gE/FOoTnxKrqG0j9w9VnpebatuCK30A10Bz1aOzf8w16svk/UJ/TsQ+AdWijXaaBeg/R8pJ6eP84vME1nkoIuAtcAAAAASUVORK5CYII=>

[image10]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABMAAAAYCAYAAAAYl8YPAAABIUlEQVR4Xu2TMUuCURSGbwniHwhaG3QRHAQHHZxCob2xHyC22a8IgpYGNxMcFaGpiBqD+g01hmuIuBR13s/3wLnH69YkvvDi9zz3fPdTuV8Iu/xHWtKGl4mUpMdeavrSX+m5dCB9i1bjYG4s7fK6aBdHlHvGgS8Ma+ALjueGM4HazBJumnDge4UyhR96pMsbB/4yvJZ6SG92R9cmN8lP0lPpg/SEa1FSm73T9ciX5KH0gO6FLkpqM3XXZHyTTXMfzoVX6SKszs6ExaCeuRvyM1mjD9h3PoseRJwzDOXIHfIVWaObHVqpN2n8T8I6+NY4xM+FpRfks4T7TLjoXvxXP4ZrYXVofaohvhGvEfjIuOxg6hO++bkpeHftXCVe3mU78gdwS1y6YatYxQAAAABJRU5ErkJggg==>

[image11]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABMAAAAYCAYAAAAYl8YPAAAA+UlEQVR4Xu3TIWtCURgG4BcEYcU4xCXHyqJhRf0BJoPRXzDL/sfGMK36BwST1aDJIAxmNYiIYrBtrAhu3z3fx7j35Xwmy8AX3nCec3jL5QKXnCsNBkpN+sDISUaO0h++SOVT+ihtQd+9ZK+BnnQnfYY+8MZmyN7l7HybskxOjSX+FLED2V+8sTbUq+RL82i8sT7Uy+RT82i8sTnUS+Rj8yJ5iDe2gfo1+ci8Th7ijQ2hfkM+MS+Qh3hjr1C/J383j8Ybu4N6k/zLPBpvLEnibxFbkIXkcXqsi+xdxc5XKUNHupeupSvrVvqdfmQZQL/sB3TI/ZUu+c/5BR7aTUg1KNZbAAAAAElFTkSuQmCC>

[image12]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAoAAAAWCAYAAAD5Jg1dAAAAnElEQVR4XmNgGHrgJBD/h2IHVCkEAElKI/E/A/E8JD4YTAfiQnRBBohmFPATiyATFjGGSqggCKdDxb4DcR1cBRJ4zIBQDMIBqNKo4A8DQuFfBoj1KOAwEL9E4i9iQGjQQRIHC6DrloCKb4AJGEAFsAGQu+8jC+BS+BuIK5AFVBkgilcDMQcQh0H51ciKkEEzEF8C4nYgZkGTGz4AAJtGKVkekTfpAAAAAElFTkSuQmCC>