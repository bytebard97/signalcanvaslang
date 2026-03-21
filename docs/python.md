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

# Parse a .patch file
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

Use the Python bindings for server-side `.patch` file validation:

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

### `patchlang_python.parse(source: str) -> str`

Parses PatchLang source and returns a JSON string with the AST and any errors. Same format as the WASM `parse()` function.

### `patchlang_python.validate(source: str) -> bool`

Returns `True` if the source parses without errors.

## Requirements

- Python 3.9+
- Rust toolchain (for building from source)
- maturin (`pipx install maturin`)
