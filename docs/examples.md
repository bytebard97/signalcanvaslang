---
layout: default
title: Examples
---

# Examples

Real-world `.patch` files from the test suite.

## Worship Venue

A typical house-of-worship audio system with Yamaha stageboxes, a CL5 mixing console, and a Dante network switch.

```
template Rio3224 {
  meta {
    manufacturer: "Yamaha"
    model: "Rio3224"
    category: "Stagebox"
  }
  ports {
    Dante_Pri: io(etherCON) [Dante, primary]
    Dante_Sec: io(etherCON) [Dante, secondary]
    Mic_In[1..32]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri
}

template CL5 {
  meta {
    manufacturer: "Yamaha"
    model: "CL5"
    category: "Console"
  }
  ports {
    Dante_Pri: io(etherCON) [Dante, primary]
    Dante_Sec: io(etherCON) [Dante, secondary]
    Dante_Ch[1..72]: in [Dante]
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

connect Stage_Left.Dante_Pri -> Dante_Switch.Port[1] {
  cable: "Cat6a_SL_Pri"
  length: "30m"
}

connect Stage_Right.Dante_Pri -> Dante_Switch.Port[3] {
  cable: "Cat6a_SR_Pri"
  length: "25m"
}

connect Dante_Switch.Port[5] -> FOH_Console.Dante_Pri {
  cable: "Cat6a_FOH_Pri"
  length: "3m"
}

bridge Stage_Left.Mic_In[1..32] -> FOH_Console.Dante_Ch[1..32]
bridge Stage_Right.Mic_In[1..16] -> FOH_Console.Dante_Ch[33..48]

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
```

## Broadcast Truck

A broadcast OB van with cameras, video router, encoder, and sync distribution.

See the full file: [broadcast-truck.patch](https://github.com/ByteBard97/SignalCanvasLang/blob/master/tests/fixtures/examples/broadcast-truck.patch)

## Hillsong Production

A real-world 1,485-line production file from a Hillsong campus with 24 templates, 53 instances, 99 connections, 23 config blocks, and 4 streams. Includes AVID Venue consoles, MADI infrastructure, Dante networks, wireless systems, and in-ear monitoring.

See the full file: [hillsong-mtg.patch](https://github.com/ByteBard97/SignalCanvasLang/blob/master/tests/fixtures/examples/hillsong-mtg.patch)

## More Examples

The [tests/fixtures/](https://github.com/ByteBard97/SignalCanvasLang/tree/master/tests/fixtures) directory contains 50+ `.patch` files including device libraries for Yamaha, Shure, Blackmagic, Ross, Riedel, and more.
