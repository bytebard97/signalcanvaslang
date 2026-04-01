#!/usr/bin/env python3
"""
migrate-device-type-to-kind.py — Rename device_type to kind across SignalCanvas

Migrations applied:
  1. .patch files:  device_type: "xxx" → kind: "xxx"
  2. .rs files:     KNOWN_DEVICE_TYPES → KNOWN_KINDS, device_type → kind in
                    string literals, variable names, comments
  3. .ts/.vue files: device_type string literals in PatchLang emitters/loaders
                     (meta.device_type → meta.kind, emitted "device_type:" → "kind:")
  4. .md files:     device_type → kind in code examples and references
                    (preserves intentional references in D011 decision rationale)
  5. .py files:     device_type → kind in test data and examples

Usage:
  python migrate-device-type-to-kind.py [--dry-run] [--backup] [--check] <dir>

Options:
  --dry-run   Show diffs without writing any files.
  --backup    Write <file>.bak before overwriting.
  --check     Exit 1 if any file would be changed (useful in CI).

Example:
  python migrate-device-type-to-kind.py --dry-run /Users/ceres/Desktop/SignalCanvas
"""

import re
import sys
import shutil
import difflib
from pathlib import Path

# ── Files to skip (contain intentional historical device_type references) ─────

SKIP_FILES = {
    # D011 decision rationale intentionally discusses the rename
    "decisions.md",
    # Debate context mentions the rename
    "debate-context.md",
    # This script itself
    "migrate-device-type-to-kind.py",
}

# Files where we only update .patch-style content (device_type: "xxx" → kind: "xxx")
# but NOT internal TypeScript property names (deviceType stays as-is)
PATCH_STRING_ONLY_EXTENSIONS = {".ts", ".vue", ".js"}


# ── .patch file migration ────────────────────────────────────────────────────

def migrate_patch_content(source: str) -> str:
    """Replace device_type: with kind: in PatchLang source."""
    # Match device_type in meta blocks: device_type: "value" or device_type: "value"
    return re.sub(
        r'(\s*)device_type(\s*:\s*")',
        r'\1kind\2',
        source,
    )


# ── Rust file migration ─────────────────────────────────────────────────────

def migrate_rust(source: str) -> str:
    """Rename device_type → kind in Rust source files."""
    s = source

    # Constants
    s = s.replace("KNOWN_DEVICE_TYPES", "KNOWN_KINDS")

    # String literals in code (meta key checks, error messages)
    s = s.replace('"device_type"', '"kind"')

    # Variable names
    s = s.replace("device_type_value", "kind_value")

    # Comments and doc strings
    s = re.sub(
        r"(//[!/]?\s*.*?)device_type",
        lambda m: m.group(0).replace("device_type", "kind"),
        s,
    )

    # Error messages containing "device_type" as user-facing text
    s = s.replace("Unknown device_type", "Unknown kind")
    s = s.replace("Set device_type to", "Set kind to")
    s = s.replace("Add device_type:", "Add kind:")
    s = s.replace("no device_type is declared", "no kind is declared")
    s = s.replace("device_type is", "kind is")

    # Test function names
    s = s.replace("unknown_device_type", "unknown_kind")

    # device_type in PatchLang test fixture strings
    s = migrate_patch_content(s)

    return s


# ── TypeScript/Vue migration ────────────────────────────────────────────────

def migrate_typescript(source: str) -> str:
    """Replace device_type PatchLang string literals in TS/Vue files.

    This targets the PatchLang output strings (emitters writing device_type:)
    and PatchLang input reads (loaders reading meta.device_type).
    It does NOT rename the internal TypeScript deviceType property.
    """
    s = source

    # Emitter output: lines like `device_type: "card"` or `device_type: "${...}"`
    # These are PatchLang string fragments being generated
    s = re.sub(r'device_type: "', 'kind: "', s)
    s = re.sub(r"device_type: \"", 'kind: "', s)
    s = re.sub(r'device_type: "\$\{', 'kind: "${', s)

    # Template literal: `device_type: "${device.deviceType}"`
    s = s.replace('device_type: "\\${', 'kind: "\\${')

    # Loader reads: meta.device_type (accessing parsed PatchLang AST)
    s = s.replace("meta.device_type", "meta.kind")
    s = s.replace('meta["device_type"]', 'meta["kind"]')

    # Comments referencing device_type in PatchLang context
    s = re.sub(
        r"(//\s*.*?)\bdevice_type\b",
        lambda m: m.group(0).replace("device_type", "kind"),
        s,
    )

    # String content in test descriptions
    s = re.sub(
        r'(".*?)device_type(.*?")',
        lambda m: m.group(0).replace("device_type", "kind"),
        s,
    )

    return s


# ── Python file migration ───────────────────────────────────────────────────

def migrate_python(source: str) -> str:
    """Replace device_type in Python test data and PatchLang strings."""
    return migrate_patch_content(source)


# ── Markdown migration ───────────────────────────────────────────────────────

def migrate_markdown(source: str) -> str:
    """Replace device_type references in documentation.

    Targets code blocks and inline code. Preserves references that are
    clearly discussing the rename itself (in D011 decision docs).
    """
    s = source

    # Inline code: `device_type: "card"` → `kind: "card"`
    s = re.sub(r'`device_type: "([^"]+)"`', r'`kind: "\1"`', s)

    # Inline code: `device_type` alone → `kind`
    s = re.sub(r"`device_type`", "`kind`", s)

    # Code block content: device_type: "xxx" → kind: "xxx"
    s = migrate_patch_content(s)

    # Meta references: meta { device_type: → meta { kind:
    s = s.replace("meta { device_type:", "meta { kind:")

    # KNOWN_DEVICE_TYPES in docs
    s = s.replace("KNOWN_DEVICE_TYPES", "KNOWN_KINDS")

    # Prose: "the device_type meta key" → "the kind meta key"
    s = s.replace("the device_type meta key", "the kind meta key")
    s = s.replace("the `device_type` field", "the `kind` field")

    return s


# ── File dispatcher ──────────────────────────────────────────────────────────

EXTENSION_HANDLERS = {
    ".patch": migrate_patch_content,
    ".rs": migrate_rust,
    ".ts": migrate_typescript,
    ".vue": migrate_typescript,
    ".js": migrate_typescript,
    ".py": migrate_python,
    ".md": migrate_markdown,
}


def should_skip(path: Path) -> bool:
    """Check if this file should be skipped."""
    if path.name in SKIP_FILES:
        return True
    # Skip node_modules, .git, build artifacts
    parts = path.parts
    skip_dirs = {"node_modules", ".git", "target", "pkg-node", "pkg-web",
                 ".venv", "__pycache__", "dist", ".next", "staticfiles",
                 "collected_static", "build"}
    return bool(skip_dirs & set(parts))


def migrate_file(path: Path, dry_run: bool, backup: bool) -> bool:
    """Migrate a single file. Returns True if changed."""
    if should_skip(path):
        return False

    handler = EXTENSION_HANDLERS.get(path.suffix)
    if handler is None:
        return False

    if not path.is_file():
        return False

    try:
        source = path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, PermissionError, IsADirectoryError):
        return False

    # Skip files with no device_type references
    if "device_type" not in source and "KNOWN_DEVICE_TYPES" not in source:
        return False

    new_source = handler(source)

    if new_source == source:
        return False

    if dry_run:
        diff = difflib.unified_diff(
            source.splitlines(keepends=True),
            new_source.splitlines(keepends=True),
            fromfile=str(path),
            tofile=str(path) + " (migrated)",
        )
        print("".join(diff), end="")
        return True

    if backup:
        shutil.copy2(path, str(path) + ".bak")

    path.write_text(new_source, encoding="utf-8")
    changes = sum(
        1 for line in difflib.unified_diff(
            source.splitlines(), new_source.splitlines()
        )
        if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))
    )
    print(f"  migrated: {path}  ({changes} line(s) changed)")
    return True


# ── CLI ──────────────────────────────────────────────────────────────────────

def main():
    args = sys.argv[1:]
    dry_run = "--dry-run" in args
    backup = "--backup" in args
    check = "--check" in args
    paths = [a for a in args if not a.startswith("--")]

    if not paths:
        print(__doc__)
        sys.exit(0)

    targets: list[Path] = []
    for p in paths:
        path = Path(p)
        if path.is_dir():
            for ext in EXTENSION_HANDLERS:
                targets.extend(sorted(path.rglob(f"*{ext}")))
        elif path.is_file():
            targets.append(path)
        else:
            print(f"error: not found: {p}", file=sys.stderr)
            sys.exit(2)

    # Deduplicate and sort
    targets = sorted(set(targets))

    changed_count = 0
    for target in targets:
        changed = migrate_file(target, dry_run=dry_run or check, backup=backup)
        if changed:
            changed_count += 1
            if check and not dry_run:
                print(f"  would change: {target}")

    print(f"\n{'Would change' if dry_run or check else 'Changed'}: {changed_count} file(s)")

    if check:
        sys.exit(1 if changed_count > 0 else 0)


if __name__ == "__main__":
    main()
