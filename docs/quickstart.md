---
layout: default
title: Quick Start
---

# Quick Start

Parse your first `.patch` file in 5 minutes.

## 1. Create a `.patch` file

Save this as `my-venue.patch`:

```
template Stagebox {
  meta {
    manufacturer: "Yamaha"
    model: "Rio3224"
    category: "Stagebox"
  }
  ports {
    Dante_Pri: io(etherCON) [Dante, primary]
    Mic_In[1..32]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri
}

template Console {
  meta {
    manufacturer: "Yamaha"
    model: "CL5"
    category: "Console"
  }
  ports {
    Dante_Pri: io(etherCON) [Dante, primary]
    Dante_Ch[1..72]: in [Dante]
    Mix_Bus[1..24]: out
  }
}

instance Stage_Left is Stagebox {
  location: "Stage Left Wing"
  ip: "192.168.1.31"
}

instance FOH is Console {
  location: "Front of House"
  ip: "192.168.1.10"
}

connect Stage_Left.Dante_Pri -> FOH.Dante_Pri {
  cable: "Cat6a_SL"
  length: "30m"
}

bridge Stage_Left.Mic_In[1..32] -> FOH.Dante_Ch[1..32]

signal Lead_Vocal {
  origin: Stage_Left.Mic_In[1]
  description: "Worship leader vocal"
}
```

## 2. Parse it

### CLI

```bash
patchlang my-venue.patch
```

Output is a JSON AST:
```json
{
  "statements": [
    { "type": "Template", "name": "Stagebox", ... },
    { "type": "Template", "name": "Console", ... },
    { "type": "Instance", "name": "Stage_Left", "templateName": "Stagebox", ... },
    { "type": "Instance", "name": "FOH", "templateName": "Console", ... },
    { "type": "Connect", ... },
    { "type": "Bridge", ... },
    { "type": "Signal", "name": "Lead_Vocal", ... }
  ]
}
```

### JavaScript

```javascript
import { parse } from './pkg-node/patchlang_wasm.js'
import { readFileSync } from 'fs'

const source = readFileSync('my-venue.patch', 'utf-8')
const result = JSON.parse(parse(source))

console.log(`${result.program.statements.length} statements, ${result.errors.length} errors`)

for (const stmt of result.program.statements) {
  console.log(`  ${stmt.type}: ${stmt.name || ''}`)
}
```

### Python

```python
import json
import patchlang_python

with open('my-venue.patch') as f:
    source = f.read()

result = json.loads(patchlang_python.parse(source))
print(f"{len(result['program']['statements'])} statements")
```

### Rust

```rust
let source = std::fs::read_to_string("my-venue.patch").unwrap();
let result = patchlang::parse(&source);

if result.is_valid() {
    for stmt in &result.program.statements {
        println!("{:?}", stmt);
    }
} else {
    for err in &result.errors {
        eprintln!("{}", err);
    }
}
```

## 3. What's in the AST?

The parser produces a `PatchProgram` with a flat list of `statements`. Each statement has a `type` field:

| Type | What it represents |
|------|-------------------|
| `Template` | Reusable device definition (ports, meta, bridges) |
| `Instance` | Physical device placed from a template |
| `Connect` | Physical cable between two ports |
| `Bridge` | Logical signal mapping (no physical cable) |
| `BridgeGroup` | Sequential channel mapping from multiple sources |
| `LinkGroup` | Grouped connections (e.g., quad-link 4K SDI) |
| `Signal` | Named signal path with origin and properties |
| `Flag` | Status indicator or alert |
| `Stream` | Dante/AES67 virtual channel group |
| `Config` | Per-instance channel labels and metadata |
| `Use` | Library import |

## Next Steps

- Read the full [Language Specification](specification.md) for all syntax details
- See [Examples](examples.md) for real-world `.patch` files
- Check [Installation](installation.md) for your platform
