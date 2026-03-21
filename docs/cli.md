---
layout: default
title: CLI
---

# Command-Line Interface

The `patchlang` CLI parses `.patch` files and outputs JSON ASTs.

## Installation

```bash
git clone https://github.com/ByteBard97/SignalCanvasLang
cd SignalCanvasLang
cargo install --path crates/patchlang-cli
```

## Usage

### Parse a file

```bash
patchlang my-venue.patch
```

Outputs the AST as pretty-printed JSON to stdout. If there are parse errors, they are printed to stderr and the process exits with code 1 — but a partial AST is still output.

### Pipe from stdin

```bash
echo 'instance FOH is CL5' | patchlang
```

### Validate only

Check the exit code — 0 means valid, 1 means errors:

```bash
patchlang my-venue.patch > /dev/null 2>&1 && echo "Valid" || echo "Invalid"
```

## Error Output

Errors are printed to stderr with line and column numbers:

```
error[3:20]: expected ']' to close port range
  hint: port ranges must be closed with ']'
```

The partial AST is still written to stdout so downstream tools can process whatever did parse successfully.
