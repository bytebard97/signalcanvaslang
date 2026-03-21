---
layout: default
title: Home
---

<p align="center">
  <img src="images/logo.png" alt="SignalCanvas" width="400">
</p>

# PatchLang

A domain-specific language for describing signal flow in broadcast and live production environments. Human-readable, git-diffable, LLM-friendly.

```
template Rio3224 {
  meta { manufacturer: "Yamaha"  model: "Rio3224"  category: "Stagebox" }
  ports {
    Dante_Pri: io(etherCON) [Dante, primary]
    Mic_In[1..32]: in(XLR)
    Line_Out[1..16]: out(XLR)
  }
  bridge Mic_In -> Dante_Pri
}

instance Stage_Left is Rio3224 { location: "Stage Left Wing" }
connect Stage_Left.Dante_Pri -> FOH.Dante_Pri { cable: "Cat6a"  length: "30m" }
bridge Stage_Left.Mic_In[1..32] -> FOH.Dante_Ch[1..32]
```

---

## Documentation

### Getting Started

- **[Installation](installation.md)** -- Rust library, CLI, WASM, and Python bindings
- **[Quick Start](quickstart.md)** -- Parse your first `.patch` file in 5 minutes

### Language Reference

- **[Language Specification](specification.md)** -- Complete EBNF grammar and syntax reference
- **[Examples](examples.md)** -- Real-world `.patch` files for worship venues, broadcast trucks, and more

### Integration

- **[WebAssembly](wasm.md)** -- Use PatchLang in the browser or Node.js
- **[Python](python.md)** -- Use PatchLang in Django or any Python project
- **[CLI](cli.md)** -- Command-line validation and AST output

## Project Links

- **[GitHub Repository](https://github.com/ByteBard97/SignalCanvasLang)**
- **[SignalCanvas](https://github.com/ByteBard97/SignalCanvas)** -- The infinite-canvas signal flow tool that uses PatchLang
- **[Language Spec (SPEC.md)](https://github.com/ByteBard97/SignalCanvasLang/blob/master/SPEC.md)**
