---
layout: default
title: Python Integration
---

# Python Integration

PatchLang provides Python bindings via [PyO3](https://pyo3.rs/) and [maturin](https://www.maturin.rs/), producing a native Python extension module.

## Building

```bash
./scripts/build-python.sh
```

This creates a virtual environment (`.venv/`) and installs the `patchlang_python` package in development mode.

## Usage

```python
import json
import patchlang_python

with open('my-venue.patch') as f:
    source = f.read()

result = json.loads(patchlang_python.parse(source))

if result['errors']:
    for err in result['errors']:
        print(f"Error: {err['message']}")
else:
    statements = result['program']['statements']
    print(f"Parsed {len(statements)} statements")

# Quick validation
is_valid = patchlang_python.validate(source)
```

## Django Integration

```python
import json
from django.core.exceptions import ValidationError
import patchlang_python

def validate_patch_content(content: str) -> dict:
    """Parse and validate .patch content on save."""
    result = json.loads(patchlang_python.parse(content))

    if result['errors']:
        err = result['errors'][0]
        raise ValidationError(f"Invalid PatchLang: {err['message']}")

    return result['program']

def extract_project_summary(content: str) -> dict:
    """Extract device counts and metadata for search/billing."""
    program = validate_patch_content(content)

    templates = [s for s in program['statements'] if s['type'] == 'Template']
    instances = [s for s in program['statements'] if s['type'] == 'Instance']
    connects  = [s for s in program['statements'] if s['type'] == 'Connect']

    return {
        'template_count': len(templates),
        'instance_count': len(instances),
        'connection_count': len(connects),
        'device_names': [i['name'] for i in instances],
    }
```

## API

> Most functions return a **JSON string** — call `json.loads()` on the result.
> Exceptions: `validate` returns `bool`; `resolve_uses` returns a native `list[str]`.

---

### `parse(source: str) -> str`

Parse PatchLang source. Returns JSON `{ "program": {...} | null, "errors": [...] }`.

```python
result = json.loads(patchlang_python.parse(source))
if not result['errors']:
    print(result['program']['statements'])
```

---

### `validate(source: str) -> bool`

Returns `True` if the source parses with no errors. Does not run DRC.

```python
if not patchlang_python.validate(source):
    raise ValueError("invalid .patch file")
```

---

### `check(source: str) -> str`

Parse + Design Rule Check. Returns JSON `{ "program": {...} | null, "errors": [...], "diagnostics": [...] }`.
Use this instead of `parse` when you also want DRC warnings (e.g. unconnected ports).

```python
result = json.loads(patchlang_python.check(source))
for d in result['diagnostics']:
    print(f"[{d['severity']}] {d['message']}")
```

---

### `resolve_uses(source: str) -> list[str]`

Quick-parse and return the namespace strings from all `use` statements. Returns a native Python `list[str]` — no `json.loads()` needed.

```python
namespaces = patchlang_python.resolve_uses(source)
# e.g. ["vendor/dante", "vendor/madi"]
```

---

### `compile_project(files: dict[str, str], entry: str) -> str`

Multi-file compilation with namespace resolution and merged DRC.

- `files` — mapping of file path → source string for every file in the project
- `entry` — the path of the entry file (must be a key in `files`)

Returns JSON `{ "program": {...} | null, "errors": [...], "diagnostics": [...], "files": [...], "templateFiles": {...}, "useGraph": {...} }`.

```python
files = {
    "main.patch": open("main.patch").read(),
    "vendor/dante.patch": open("vendor/dante.patch").read(),
}
result = json.loads(patchlang_python.compile_project(files, "main.patch"))
```

---

### `generate_port_id(instance_name: str, template_name: str, port_name: str, index: int | None = None) -> str`

Generate a deterministic port ID string. Returns a plain `str` — no `json.loads()` needed.

- `index` — omit (or pass `None`) for single (non-ranged) ports; pass an integer for ranged ports.

```python
port_id = patchlang_python.generate_port_id("Desk", "DiGiCo_SD12", "AES_Out", 3)
```

---

### `generate_route_id(template_name: str, source_port: str, target_port: str) -> str`

Generate a deterministic route ID string. Returns a plain `str`.

```python
route_id = patchlang_python.generate_route_id("DiGiCo_SD12", "AES_Out_3", "MADI_In_1")
```

---

### `generate_slot_id(template_name: str, slot_name: str) -> str`

Generate a deterministic slot ID string. Returns a plain `str`.

```python
slot_id = patchlang_python.generate_slot_id("DiGiCo_SD12", "Option_Slot_A")
```

---

### `parse_manifest(json: str) -> str`

Parse a `project.json` manifest string. Returns JSON `{ "manifest": {...} | null, "errors": [...] }`.

```python
with open("project.json") as f:
    raw = f.read()
result = json.loads(patchlang_python.parse_manifest(raw))
```

---

### `validate_layout(json: str) -> str`

Validate a `.layout.json` string against the layout schema. Returns JSON `{ "valid": bool, "errors": [...] }`.

```python
with open("MTG.layout.json") as f:
    raw = f.read()
result = json.loads(patchlang_python.validate_layout(raw))
if not result['valid']:
    print(result['errors'])
```

---

### `validate_project_consistency(patch: str, layout: str) -> str`

Cross-validate instance names between a `.patch` source string and a `.layout.json` string.
Returns JSON `{ "valid": bool, "errors": [...], "warnings": [...] }`.

```python
patch_src  = open("MTG.patch").read()
layout_src = open("MTG.layout.json").read()
result = json.loads(patchlang_python.validate_project_consistency(patch_src, layout_src))
if result['warnings']:
    for w in result['warnings']:
        print(f"Warning: {w}")
```

---

## Requirements

- Python 3.9+
- Rust toolchain (for building from source)
- maturin (`pipx install maturin`)
