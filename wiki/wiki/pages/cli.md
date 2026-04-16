---
title: CLI Tool
tags: [cli, patchlang, tool]
sources: [docs/cli]
updated: 2026-04-16
---

# CLI Tool

**Source:** `docs/cli.md`
**Type:** Tool reference

## Summary

The `patchlang` CLI parses `.patch` files and outputs JSON ASTs. Useful for CI/CD validation, scripting, and piping into other tools.

## Installation

```bash
git clone https://github.com/ByteBard97/SignalCanvasLang
cd SignalCanvasLang
cargo install --path crates/patchlang-cli
```

## Usage

```bash
# Parse a file → AST JSON to stdout
patchlang my-venue.patch

# Pipe from stdin
echo 'instance FOH is CL5' | patchlang

# Validate only (check exit code)
patchlang my-venue.patch > /dev/null 2>&1 && echo "Valid" || echo "Invalid"
```

Exit code: `0` = valid, `1` = parse errors. If there are parse errors, a partial AST is still written to stdout (parse errors go to stderr).

## Error Output

```
error[3:20]: expected ']' to close port range
  hint: port ranges must be closed with ']'
```

Errors include line and column numbers on stderr.

## Relation to Other Wiki Pages

- [[compiler-architecture]] — the Rust compiler the CLI wraps
- [[wasm-api]] — equivalent API for browser/Node.js environments
- [[python-api]] — equivalent API for Python environments
