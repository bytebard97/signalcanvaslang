#!/usr/bin/env python3
"""
migrate-patch.py  —  PatchLang source migrator to v0.2.5

Migrations applied:
  P1: io() ports with directional channel protocols split into in()/out() variants.
  P7: WordClock io() is also P1 (WordClock is in CHANNEL_PROTOCOLS).

Port references in connect, bridge, route, bus output, and config label
statements are updated automatically:
  - connect/bridge (top-level):  source.Port → Port_Out, target.Port → Port_In
  - bridge (inside template):    source Port → Port_In, target Port → Port_Out
  - route (inside instance):     source Port → Port_In, target Port → Port_Out
  - bus output (inside instance): Port → Port_Out
  - config label (inside config): Port → Port_In

Usage:
  python migrate-patch.py [--dry-run] [--backup] [--check] <file.patch | dir>

Options:
  --dry-run   Show diffs without writing any files.
  --backup    Write <file>.patch.bak before overwriting.
  --check     Exit 1 if any file would be changed (useful in CI).
"""

import re
import sys
import os
import shutil
import difflib
from pathlib import Path

# ── Protocol classification ───────────────────────────────────────────────────

# Protocols that carry directional signal flows and must not use io().
CHANNEL_PROTOCOLS = {
    "Dante", "MADI", "AES3", "AES67", "SDI", "NDI",
    "OMNEO", "RVON", "WordClock",
}

# Port names (case-insensitive) that are signal-flow edges even without a
# protocol tag. These are physical audio connections on intercom matrices
# that carry audio to/from beltpacks. Headset ports are intentionally
# excluded — those are local user I/O, not system signal flow (see D010).
SPLIT_BY_PORT_NAME = {"partyline"}


def _has_channel_protocol(protocol_list_str: str) -> bool:
    """Return True if the bracketed protocol list contains any channel protocol.

    protocol_list_str is the raw "[Dante, Gigabit]" fragment, or empty string.
    """
    if not protocol_list_str:
        return False
    inner = protocol_list_str.strip()
    if inner.startswith("[") and inner.endswith("]"):
        inner = inner[1:-1]
    return any(tok.strip() in CHANNEL_PROTOCOLS for tok in inner.split(","))


# ── Port declaration parsing ──────────────────────────────────────────────────

# Matches:  <indent><name><opt-range>: io(<connector>) <opt-protocol-list>
_PORT_IO_RE = re.compile(
    r"^(\s*)(\w+)(\[\d+\.\.\d+\])?\s*:\s*io\(([^)]+)\)(\s*\[[^\]]*\])?\s*$"
)


def _parse_io_port(line: str):
    """Parse a port declaration line that uses io().

    Returns (indent, name, range_spec, connector, proto_list) if it's an io()
    port with a channel protocol, otherwise None.
    """
    m = _PORT_IO_RE.match(line)
    if not m:
        return None
    indent, name, range_spec, connector, proto_raw = (
        m.group(1), m.group(2), m.group(3) or "", m.group(4), (m.group(5) or "").strip()
    )
    if not _has_channel_protocol(proto_raw):
        if name.lower() not in SPLIT_BY_PORT_NAME:
            return None
    return indent, name, range_spec, connector, proto_raw


def _split_port_lines(indent, name, range_spec, connector, proto_list) -> list[str]:
    """Emit the two replacement lines for a split io() port."""
    proto = f" {proto_list}" if proto_list else ""
    return [
        f"{indent}{name}_In{range_spec}: in({connector}){proto}\n",
        f"{indent}{name}_Out{range_spec}: out({connector}){proto}\n",
    ]


# ── Port reference substitution ───────────────────────────────────────────────

# Matches the index part of a port reference: [1], [1..8], [auto], etc.
_INDEX_RE = re.compile(r"(\[\w+(?:\.\.\w+)?\])")


def _sub_port_ref(port_ref: str, split_ports: set[str], direction: str) -> str:
    """Replace a bare port reference if the port was split.

    port_ref  e.g. "DANTE[1]" or "MADI_3" or "SC_MADI[1..32]"
    split_ports  set of original io() port names for the relevant template
    direction  "in" → append _In, "out" → append _Out
    """
    m = re.match(r"^(\w+)(\[.*\])?$", port_ref)
    if not m:
        return port_ref
    base, idx = m.group(1), m.group(2) or ""
    if base not in split_ports:
        return port_ref
    suffix = "_In" if direction == "in" else "_Out"
    return f"{base}{suffix}{idx}"


def _sub_instance_port_ref(
    ref: str,
    instances: dict[str, str],
    template_split_ports: dict[str, set[str]],
    direction: str,
) -> str:
    """Replace Instance.Port[idx] references for split io() ports.

    ref e.g. "Dante_SW.Port[1]" or "SL_Box.Dante_Pri_Out"
    direction "in" → Port_In, "out" → Port_Out
    """
    m = re.match(r"^(\w+)\.(\w+)(\[.*\])?$", ref)
    if not m:
        return ref
    inst, port, idx = m.group(1), m.group(2), m.group(3) or ""
    tmpl = instances.get(inst)
    if tmpl is None:
        return ref
    split_ports = template_split_ports.get(tmpl, set())
    if port not in split_ports:
        return ref
    suffix = "_In" if direction == "in" else "_Out"
    return f"{inst}.{port}{suffix}{idx}"


# ── Statement transformers ────────────────────────────────────────────────────

# Regex for top-level connect / bridge with Instance.Port references.
# Group 1: keyword, 2: source ref, 3: target ref, 4: trailing (attrs/newline)
_TOP_ARROW_RE = re.compile(
    r"^(\s*(?:connect|bridge)\s+)"
    r"([\w]+\.[\w]+(?:\[[\w.]+\])?)"
    r"(\s*->\s*)"
    r"([\w]+\.[\w]+(?:\[[\w.]+\])?)"
    r"(.*)"
)

# Regex for internal bridge (inside template body) with bare port names.
_TMPL_BRIDGE_RE = re.compile(
    r"^(\s*bridge\s+)"
    r"([\w]+(?:\[[\w.]+\])?)"
    r"(\s*->\s*)"
    r"([\w]+(?:\[[\w.]+\])?)"
    r"(.*)"
)

# Regex for route (inside instance body) with bare port names.
_ROUTE_RE = re.compile(
    r"^(\s*route\s+)"
    r"([\w]+(?:\[[\w.]+\])?)"
    r"(\s*->\s*)"
    r"([\w]+(?:\[[\w.]+\])?)"
    r"(.*)"
)

# Regex for bus output line:  output: Port[idx]
_BUS_OUTPUT_RE = re.compile(
    r"^(\s*output\s*:\s*)([\w]+(?:\[[\w.]+\])?)(.*)"
)

# Regex for config label line:  label Port[idx]: "..."
_CONFIG_LABEL_RE = re.compile(
    r"^(\s*label\s+)([\w]+(?:\[[\w.]+\])?)(.*)"
)


def _transform_top_arrow(
    line: str,
    instances: dict[str, str],
    template_split_ports: dict[str, set[str]],
) -> str:
    """Transform a top-level connect or bridge statement."""
    m = _TOP_ARROW_RE.match(line)
    if not m:
        return line
    prefix, src, arrow, tgt, tail = m.groups()
    src2 = _sub_instance_port_ref(src, instances, template_split_ports, "out")
    tgt2 = _sub_instance_port_ref(tgt, instances, template_split_ports, "in")
    return f"{prefix}{src2}{arrow}{tgt2}{tail}"


def _transform_tmpl_bridge(line: str, split_ports: set[str]) -> str:
    """Transform a bridge statement inside a template body."""
    m = _TMPL_BRIDGE_RE.match(line)
    if not m:
        return line
    prefix, src, arrow, tgt, tail = m.groups()
    # Inside a template, bridge reads from an input port and writes to an output port.
    src2 = _sub_port_ref(src, split_ports, "in")
    tgt2 = _sub_port_ref(tgt, split_ports, "out")
    return f"{prefix}{src2}{arrow}{tgt2}{tail}"


def _transform_route(line: str, split_ports: set[str]) -> str:
    """Transform a route statement inside an instance body."""
    m = _ROUTE_RE.match(line)
    if not m:
        return line
    prefix, src, arrow, tgt, tail = m.groups()
    # Route: signal enters via an input port, leaves via an output port.
    src2 = _sub_port_ref(src, split_ports, "in")
    tgt2 = _sub_port_ref(tgt, split_ports, "out")
    return f"{prefix}{src2}{arrow}{tgt2}{tail}"


def _transform_bus_output(line: str, split_ports: set[str]) -> str:
    """Transform a bus output reference inside an instance body."""
    m = _BUS_OUTPUT_RE.match(line)
    if not m:
        return line
    prefix, port_ref, tail = m.groups()
    port2 = _sub_port_ref(port_ref, split_ports, "out")
    return f"{prefix}{port2}{tail}"


def _transform_config_label(
    line: str,
    instances: dict[str, str],
    template_split_ports: dict[str, set[str]],
    config_instance: str,
) -> str:
    """Transform a label line inside a config block."""
    m = _CONFIG_LABEL_RE.match(line)
    if not m:
        return line
    prefix, port_ref, tail = m.groups()
    tmpl = instances.get(config_instance, "")
    split_ports = template_split_ports.get(tmpl, set())
    port2 = _sub_port_ref(port_ref, split_ports, "in")
    return f"{prefix}{port2}{tail}"


# ── First pass: collect template and instance info ────────────────────────────

_TEMPLATE_OPEN_RE = re.compile(r"^\s*template\s+(\w+)\s*\{")
_INSTANCE_OPEN_RE = re.compile(r"^\s*instance\s+(\w+)\s+is\s+(\w+)\s*\{")
_PORTS_OPEN_RE = re.compile(r"^\s*ports\s*\{")
_CONFIG_OPEN_RE = re.compile(r"^\s*config\s+(\w+)\s*\{")
_BRACE_OPEN_RE = re.compile(r"\{")
_BRACE_CLOSE_RE = re.compile(r"\}")


def _collect_info(lines: list[str]):
    """Scan the file to collect template io() ports and instance→template map.

    Returns:
        template_split_ports: {template_name: {port_name, ...}}
        instances: {instance_name: template_name}
    """
    template_split_ports: dict[str, set[str]] = {}
    instances: dict[str, str] = {}

    depth = 0
    current_template: str | None = None
    current_instance: str | None = None
    template_depth: int | None = None
    instance_depth: int | None = None
    in_ports = False
    ports_depth: int | None = None

    for raw_line in lines:
        line = raw_line.rstrip("\n")

        # Count braces on this line (ignore string contents).
        opens = _count_opens(line)
        closes = _count_closes(line)

        # Detect block openers BEFORE incrementing depth.
        if opens > 0:
            if current_template is None and _TEMPLATE_OPEN_RE.match(line):
                m = _TEMPLATE_OPEN_RE.match(line)
                current_template = m.group(1)
                template_depth = depth
                template_split_ports.setdefault(current_template, set())

            elif current_instance is None and _INSTANCE_OPEN_RE.match(line):
                m = _INSTANCE_OPEN_RE.match(line)
                inst_name, tmpl_name = m.group(1), m.group(2)
                instances[inst_name] = tmpl_name
                current_instance = inst_name
                instance_depth = depth

            elif current_template is not None and not in_ports and _PORTS_OPEN_RE.match(line):
                in_ports = True
                ports_depth = depth

        # Parse port declarations when inside a ports block.
        if in_ports and opens == 0 and closes == 0:
            parsed = _parse_io_port(line)
            if parsed and current_template:
                _, name, *_ = parsed
                template_split_ports[current_template].add(name)

        depth += opens

        # Detect block closers AFTER incrementing depth, checking at depth-closes.
        if closes > 0:
            depth -= closes
            if in_ports and ports_depth is not None and depth <= ports_depth:
                in_ports = False
                ports_depth = None
            if current_template is not None and template_depth is not None and depth <= template_depth:
                current_template = None
                template_depth = None
            if current_instance is not None and instance_depth is not None and depth <= instance_depth:
                current_instance = None
                instance_depth = None

    # Remove templates with no split ports.
    return {k: v for k, v in template_split_ports.items() if v}, instances


def _count_opens(line: str) -> int:
    return sum(1 for c in _iter_non_string_chars(line) if c == "{")


def _count_closes(line: str) -> int:
    return sum(1 for c in _iter_non_string_chars(line) if c == "}")


def _iter_non_string_chars(line: str):
    in_str = False
    for ch in line:
        if ch == '"':
            in_str = not in_str
        elif not in_str:
            yield ch


# ── Second pass: line-by-line transformation ──────────────────────────────────

def _transform_lines(
    lines: list[str],
    template_split_ports: dict[str, set[str]],
    instances: dict[str, str],
) -> list[str]:
    """Transform lines using the collected template and instance info."""

    depth = 0
    current_template: str | None = None
    current_instance: str | None = None
    current_instance_tmpl: str | None = None
    template_depth: int | None = None
    instance_depth: int | None = None
    in_ports = False
    ports_depth: int | None = None
    in_bus = False
    bus_depth: int | None = None
    in_config = False
    config_depth: int | None = None
    config_instance: str | None = None

    out: list[str] = []

    for raw_line in lines:
        line = raw_line.rstrip("\n")
        opens = _count_opens(line)
        closes = _count_closes(line)

        # ── Detect block openers ────────────────────────────────────────────
        if opens > 0:
            if current_template is None and _TEMPLATE_OPEN_RE.match(line):
                m = _TEMPLATE_OPEN_RE.match(line)
                current_template = m.group(1)
                template_depth = depth

            elif current_instance is None and _INSTANCE_OPEN_RE.match(line):
                m = _INSTANCE_OPEN_RE.match(line)
                current_instance = m.group(1)
                current_instance_tmpl = instances.get(current_instance)
                instance_depth = depth

            elif current_template is not None and not in_ports and _PORTS_OPEN_RE.match(line):
                in_ports = True
                ports_depth = depth

            elif current_instance is not None and not in_bus:
                if re.match(r"\s*bus\s+\w+\s*\{", line):
                    in_bus = True
                    bus_depth = depth

            if not in_config and _CONFIG_OPEN_RE.match(line):
                m = _CONFIG_OPEN_RE.match(line)
                in_config = True
                config_depth = depth
                config_instance = m.group(1)

        depth += opens

        # ── Transform the line ──────────────────────────────────────────────

        # Port declaration inside a ports block.
        if in_ports and opens == 0 and closes == 0:
            parsed = _parse_io_port(line)
            if parsed:
                indent, name, range_spec, connector, proto_list = parsed
                out.extend(_split_port_lines(indent, name, range_spec, connector, proto_list))
                depth -= closes
                _update_closers(closes, depth,
                                 in_ports, ports_depth,
                                 current_template, template_depth,
                                 current_instance, instance_depth,
                                 in_bus, bus_depth,
                                 in_config, config_depth)
                continue

        # Bridge inside a template (bare port names, not Instance.Port).
        if (current_template is not None
                and current_instance is None
                and _TMPL_BRIDGE_RE.match(line)):
            split_ports = template_split_ports.get(current_template, set())
            line = _transform_tmpl_bridge(line, split_ports)

        # Route inside an instance body.
        elif current_instance is not None and _ROUTE_RE.match(line):
            split_ports = template_split_ports.get(current_instance_tmpl or "", set())
            line = _transform_route(line, split_ports)

        # Bus output inside an instance body.
        elif in_bus and _BUS_OUTPUT_RE.match(line):
            split_ports = template_split_ports.get(current_instance_tmpl or "", set())
            line = _transform_bus_output(line, split_ports)

        # Config label (top-level config block).
        elif in_config and _CONFIG_LABEL_RE.match(line) and config_instance:
            line = _transform_config_label(line, instances, template_split_ports, config_instance)

        # Top-level connect or bridge (Instance.Port references).
        elif (current_template is None
              and current_instance is None
              and not in_config
              and _TOP_ARROW_RE.match(line)):
            line = _transform_top_arrow(line, instances, template_split_ports)

        out.append(line + "\n" if not raw_line.endswith("\n") else line + "\n")

        # ── Update context on close ─────────────────────────────────────────
        if closes > 0:
            depth -= closes
            if in_ports and ports_depth is not None and depth <= ports_depth:
                in_ports = False
                ports_depth = None
            if in_bus and bus_depth is not None and depth <= bus_depth:
                in_bus = False
                bus_depth = None
            if in_config and config_depth is not None and depth <= config_depth:
                in_config = False
                config_depth = None
                config_instance = None
            if current_template is not None and template_depth is not None and depth <= template_depth:
                current_template = None
                template_depth = None
            if current_instance is not None and instance_depth is not None and depth <= instance_depth:
                current_instance = None
                current_instance_tmpl = None
                instance_depth = None

    return out


def _update_closers(closes, depth,
                    in_ports, ports_depth,
                    current_template, template_depth,
                    current_instance, instance_depth,
                    in_bus, bus_depth,
                    in_config, config_depth):
    """Placeholder — state is managed inline in _transform_lines."""
    pass


# ── File migration ─────────────────────────────────────────────────────────────

def migrate_source(source: str) -> tuple[str, int]:
    """Migrate PatchLang source text.

    Returns (migrated_source, changes_count).
    """
    lines = source.splitlines(keepends=True)
    # Ensure last line has newline.
    if lines and not lines[-1].endswith("\n"):
        lines[-1] += "\n"

    template_split_ports, instances = _collect_info(lines)
    if not template_split_ports:
        return source, 0

    transformed = _transform_lines(lines, template_split_ports, instances)
    new_source = "".join(transformed)
    changes = sum(1 for a, b in zip(
        difflib.ndiff(lines, transformed), [""] * max(len(lines), len(transformed))
    ) if a.startswith(("+ ", "- ")))
    # Simpler change count: compare original vs transformed line counts + diffs.
    diff_lines = list(difflib.unified_diff(lines, transformed, lineterm=""))
    changes = sum(1 for l in diff_lines if l.startswith(("+", "-")) and not l.startswith(("+++", "---")))
    return new_source, changes


def migrate_file(path: Path, dry_run: bool, backup: bool) -> bool:
    """Migrate a single .patch file. Returns True if the file was (or would be) changed."""
    source = path.read_text(encoding="utf-8")
    new_source, changes = migrate_source(source)

    if changes == 0:
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
    print(f"  migrated: {path}  ({changes} line(s) changed)")
    return True


# ── CLI ────────────────────────────────────────────────────────────────────────

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
            targets.extend(sorted(path.rglob("*.patch")))
        elif path.is_file():
            targets.append(path)
        else:
            print(f"error: not found: {p}", file=sys.stderr)
            sys.exit(2)

    any_changed = False
    for target in targets:
        changed = migrate_file(target, dry_run=dry_run or check, backup=backup)
        if changed:
            any_changed = True
            if check and not dry_run:
                print(f"  would change: {target}")

    if check:
        sys.exit(1 if any_changed else 0)


if __name__ == "__main__":
    main()
