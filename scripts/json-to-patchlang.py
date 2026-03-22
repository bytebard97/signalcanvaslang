#!/usr/bin/env python3
"""
json-to-patchlang.py — Convert legacy SignalCanvas JSON projects to PatchLang.

WHY THIS EXISTS:
    SignalCanvas originally stored projects as monolithic JSON blobs containing
    UUIDs, nested device snapshots, and cross-referenced IDs. PatchLang is now
    the source of truth — a human-readable DSL that is git-diffable, LLM-friendly,
    and far more compact. This script bridges the gap by converting existing JSON
    project files (e.g. Reid's MTG.json, DM7_Plan.json) into .patch + .layout.json
    pairs that the compiler and frontend can consume natively.

    A TypeScript equivalent exists in SignalCanvasFrontend/scripts/migrate-json-to-patchlang.ts,
    but it depends on the frontend's emitter module and type definitions. This Python
    version is fully standalone — no external dependencies, runs anywhere with Python 3.

WHAT IT DOES:
    1. Reads a SignalCanvas JSON project file (placedDevices, connections, etc.)
    2. Deduplicates device snapshots into PatchLang templates
    3. Emits instance blocks with installed cards, channel labels, internal routes,
       and internal buses
    4. Emits connect statements with channel mapping support (1:1, range, explicit)
    5. Writes a .layout.json sidecar preserving canvas positions

USAGE:
    python3 scripts/json-to-patchlang.py input.json > output.patch
    python3 scripts/json-to-patchlang.py input.json -o output.patch

    When -o is used, both output.patch and output.layout.json are written.
"""

import json
import sys
import re
import argparse
from collections import OrderedDict


def sanitize_id(name: str) -> str:
    """Convert a display name to a valid PatchLang identifier."""
    if not name:
        return "Unknown"
    # Replace special chars with underscores
    s = re.sub(r'[^a-zA-Z0-9_]', '_', name)
    # Collapse multiple underscores
    s = re.sub(r'_+', '_', s)
    # Strip leading/trailing underscores
    s = s.strip('_')
    # Must start with letter or underscore
    if s and s[0].isdigit():
        s = '_' + s
    return s or "Unknown"


def direction_to_patchlang(direction: str) -> str:
    """Map JSON direction strings to PatchLang direction keywords."""
    mapping = {
        'in': 'in',
        'input': 'in',
        'out': 'out',
        'output': 'out',
        'io': 'io',
        'bidirectional': 'io',
        'any': 'io',
    }
    return mapping.get(direction, 'io')


def connector_to_id(connector: str) -> str:
    """Sanitize connector name for PatchLang."""
    if not connector:
        return None
    return sanitize_id(connector)


def build_template_key(snapshot: dict) -> str:
    """Build a dedup key for a device snapshot to avoid duplicate templates.

    JSON projects store a full device snapshot per placed device, so the same
    model appears N times. We hash by (manufacturer, model, interfaces, slots)
    to emit each unique template only once in the PatchLang output.
    """
    ifaces = []
    for iface in snapshot.get('interfaces', []):
        ifaces.append((
            iface.get('label', ''),
            iface.get('connector', ''),
            iface.get('direction', ''),
            iface.get('count', 1),
            tuple(t.get('protocol', '') for t in iface.get('transports', [])),
        ))
    slots = []
    for sg in snapshot.get('cardSlotGroups', []):
        slots.append((
            sg.get('slotTypeName', ''),
            sg.get('quantity', 0),
            sg.get('channelCount', 0),
        ))
    return (
        snapshot.get('manufacturer', ''),
        snapshot.get('model', ''),
        tuple(sorted(ifaces)),
        tuple(sorted(slots)),
    )


def build_instance_label(pd: dict, index: int, seen_labels: dict) -> str:
    """Build a unique PatchLang instance identifier from a placed device.

    JSON projects often have duplicate instance labels (e.g. two "Stage Box"
    entries). This deduplicates by appending _2, _3, etc. when collisions occur.
    """
    label = pd.get('instanceLabel') or ''
    if not label:
        model = pd['deviceSnapshot'].get('model', '')
        label = model if model else f"Device_{index}"

    safe = sanitize_id(label)

    # Deduplicate
    if safe in seen_labels:
        seen_labels[safe] += 1
        safe = f"{safe}_{seen_labels[safe]}"
    else:
        seen_labels[safe] = 1

    return safe


def emit_template(name: str, snapshot: dict, manufacturer_cards: dict, out: list):
    """Emit a PatchLang template block."""
    manufacturer = snapshot.get('manufacturer', '')
    model = snapshot.get('model', '')

    out.append(f"template {name} {{")

    # Meta block
    if manufacturer or model:
        out.append("  meta {")
        if manufacturer:
            out.append(f'    manufacturer: "{manufacturer}"')
        if model:
            out.append(f'    model: "{model}"')
        out.append("  }")

    # Ports block
    interfaces = snapshot.get('interfaces', [])
    if interfaces:
        out.append("  ports {")
        for iface in interfaces:
            port_name = sanitize_id(iface.get('label', 'Port'))
            count = iface.get('count', 1)
            direction = direction_to_patchlang(iface.get('direction', 'io'))
            connector = connector_to_id(iface.get('connector', ''))
            transports = iface.get('transports', [])

            # Build port line
            range_spec = f"[1..{count}]" if count > 1 else ""
            connector_spec = f"({connector})" if connector else ""

            # Attributes from transports
            attrs = [t.get('protocol', '') for t in transports if t.get('protocol')]
            attr_spec = f" [{', '.join(attrs)}]" if attrs else ""

            out.append(f"    {port_name}{range_spec}: {direction}{connector_spec}{attr_spec}")
        out.append("  }")

    # Slot definitions from cardSlotGroups
    for sg in snapshot.get('cardSlotGroups', []):
        slot_name = sanitize_id(sg.get('slotTypeName', 'Slot'))
        qty = sg.get('quantity', 1)
        # Find compatible card names
        compatible_ids = sg.get('compatibleCardTypeIds', [])
        card_names = []
        for cid in compatible_ids:
            if cid in manufacturer_cards:
                card_names.append(sanitize_id(manufacturer_cards[cid].get('name', cid[:8])))
        range_spec = f"[1..{qty}]" if qty > 1 else ""
        # Use first compatible card type as the slot type
        card_type = card_names[0] if card_names else "ExpansionCard"
        out.append(f"  slot {slot_name}{range_spec}: {card_type}")

    out.append("}")


def emit_instance(instance_name: str, template_name: str, pd: dict,
                  manufacturer_cards: dict, iface_id_to_port: dict, out: list):
    """Emit a PatchLang instance block."""
    snapshot = pd['deviceSnapshot']
    instance_label = pd.get('instanceLabel', '')

    out.append(f"instance {instance_name} is {template_name} {{")

    # Instance label as a property if different from identifier
    if instance_label and sanitize_id(instance_label) != instance_name:
        out.append(f'  display_name: "{instance_label}"')

    # Installed cards
    for ic in pd.get('installedCards', []):
        if ic.get('cardTypeId') and ic['cardTypeId'] in manufacturer_cards:
            card = manufacturer_cards[ic['cardTypeId']]
            card_name = card.get('name', '')
            slot_id = ic.get('slotId', '')
            # Extract slot index from slotId format: "groupId__N"
            parts = slot_id.rsplit('__', 1)
            slot_idx = parts[-1] if len(parts) > 1 else ''
            if card_name:
                out.append(f'  card_{slot_idx}: "{card_name}"')

    # Channel labels
    for cl in pd.get('channelLabels', []):
        if isinstance(cl, dict) and cl.get('label'):
            ch = cl.get('channel', '')
            iface_id = cl.get('interfaceId', '')
            port_name = iface_id_to_port.get(iface_id, 'Ch')
            label_val = cl['label']
            out.append(f'  label_{port_name}_{ch}: "{label_val}"')

    # Internal routes
    for route in pd.get('internalRoutes', []):
        from_iface = iface_id_to_port.get(route.get('fromInterfaceId', ''), 'Input')
        to_iface = iface_id_to_port.get(route.get('toInterfaceId', ''), 'Output')
        from_ch = route.get('fromChannel', 1)
        to_ch = route.get('toChannel', 1)
        out.append(f"  route {from_iface}[{from_ch}] -> {to_iface}[{to_ch}]")

    # Internal buses
    for bus in pd.get('internalBuses', []):
        bus_name = sanitize_id(bus.get('name', 'Bus'))
        out.append(f"  bus {bus_name} {{")
        for inp in bus.get('inputs', []):
            # Each input has sources with interfaceId + channel
            for src in inp.get('sources', []):
                iface_ref = iface_id_to_port.get(src.get('fromInterfaceId', ''), 'Port')
                ch = src.get('fromChannel', 1)
                out.append(f"    input: {iface_ref}[{ch}]")
        for outp in bus.get('outputs', []):
            # Each output has destinations with interfaceId + channel
            for dest in outp.get('destinations', []):
                iface_ref = iface_id_to_port.get(dest.get('toInterfaceId', ''), 'Port')
                ch = dest.get('toChannel', 1)
                out.append(f"    output: {iface_ref}[{ch}]")
        out.append("  }")

    out.append("}")


def emit_connection(conn: dict, instance_id_to_name: dict,
                    iface_id_to_port: dict, out: list):
    """Emit a PatchLang connect statement."""
    from_inst = instance_id_to_name.get(conn.get('fromInstanceId', ''), None)
    to_inst = instance_id_to_name.get(conn.get('toInstanceId', ''), None)

    if not from_inst or not to_inst:
        out.append(f"# SKIPPED: connection references unknown instance")
        return

    from_port = iface_id_to_port.get(conn.get('fromInterfaceId', ''), 'Port')
    to_port = iface_id_to_port.get(conn.get('toInterfaceId', ''), 'Port')

    mappings = conn.get('channelMappings', [])

    if mappings:
        # Check if it's a simple 1:1 sequential mapping
        is_sequential = all(
            m.get('fromChannel') == m.get('toChannel')
            for m in mappings
        )
        channels = sorted(m.get('fromChannel', 0) for m in mappings)
        is_contiguous = (
            len(channels) > 0 and
            channels == list(range(channels[0], channels[0] + len(channels)))
        )

        if len(mappings) == 1:
            ch = mappings[0].get('fromChannel', 1)
            to_ch = mappings[0].get('toChannel', 1)
            if ch == to_ch:
                out.append(f"connect {from_inst}.{from_port}[{ch}] -> {to_inst}.{to_port}[{ch}]")
            else:
                out.append(f"connect {from_inst}.{from_port}[{ch}] -> {to_inst}.{to_port}[{to_ch}]")
        elif is_sequential and is_contiguous:
            first = channels[0]
            last = channels[-1]
            out.append(f"connect {from_inst}.{from_port}[{first}..{last}] -> {to_inst}.{to_port}[{first}..{last}]")
        else:
            # Explicit mapping
            out.append(f"connect {from_inst}.{from_port} -> {to_inst}.{to_port} {{")
            pairs = []
            for m in mappings:
                pairs.append(f"{m.get('fromChannel', '?')}->{m.get('toChannel', '?')}")
            out.append(f'  mapping: "{", ".join(pairs)}"')
            out.append("}")
    else:
        out.append(f"connect {from_inst}.{from_port} -> {to_inst}.{to_port}")


def convert(data: dict) -> str:
    """Convert a full JSON project to PatchLang."""
    result, _ = convert_with_ids(data)
    return result


def convert_with_ids(data: dict) -> tuple[str, dict]:
    """Main conversion entry point. Returns (patch_text, instance_id_to_name).

    The conversion pipeline:
      1. Index manufacturer cards by UUID (needed to resolve slot/card references)
      2. First pass over placedDevices: build interface-ID-to-port-name lookup,
         deduplicate templates, assign unique instance names
      3. Emit templates (one per unique device model/interface combo)
      4. Emit instances grouped by groupBox membership, then ungrouped remainder
      5. Emit connect statements with channel mapping simplification
    """
    out = []
    out.append("# Converted from SignalCanvas JSON project")
    out.append(f"# Version: {data.get('version', '?')}")
    out.append("")

    # Build manufacturer card lookup
    manufacturer_cards = {}
    for mc in data.get('manufacturerCards', []):
        manufacturer_cards[mc['id']] = mc

    # Deduplicate templates by structure
    template_key_to_name = {}
    template_names_used = set()

    # Build interface ID -> port name lookup (across all placed devices)
    iface_id_to_port = {}

    # First pass: collect all templates and build lookups
    placed_devices = data.get('placedDevices', [])
    instance_id_to_name = {}
    seen_labels = {}
    device_template_map = []  # (instance_name, template_name, pd)

    for i, pd in enumerate(placed_devices):
        snapshot = pd['deviceSnapshot']
        instance_id = pd['instanceId']

        # Map interface IDs to port names for this device
        for iface in snapshot.get('interfaces', []):
            iface_id_to_port[iface['id']] = sanitize_id(iface.get('label', 'Port'))

        # Also map card-expanded interface IDs
        for ic in pd.get('installedCards', []):
            if ic.get('cardTypeId') and ic['cardTypeId'] in manufacturer_cards:
                card = manufacturer_cards[ic['cardTypeId']]
                slot_id = ic.get('slotId', '')
                for card_iface in card.get('interfaces', []):
                    # Card interfaces get compound IDs: slotId__cardInterfaceId
                    compound_id = f"{slot_id}__{card_iface['id']}"
                    iface_id_to_port[compound_id] = sanitize_id(card_iface.get('label', 'Card_Port'))

        # Build template
        tkey = build_template_key(snapshot)
        if tkey not in template_key_to_name:
            base_name = sanitize_id(snapshot.get('model', '') or f"Device_{i}")
            tname = base_name
            counter = 2
            while tname in template_names_used:
                tname = f"{base_name}_{counter}"
                counter += 1
            template_key_to_name[tkey] = (tname, snapshot)
            template_names_used.add(tname)

        template_name = template_key_to_name[tkey][0]

        # Build instance name
        instance_name = build_instance_label(pd, i, seen_labels)
        instance_id_to_name[instance_id] = instance_name
        device_template_map.append((instance_name, template_name, pd))

    # Also handle "inputs"/"outputs" pseudo-interface IDs used in connections
    # These reference the first input/output interface on the target device
    for pd in placed_devices:
        snapshot = pd['deviceSnapshot']
        first_in = None
        first_out = None
        for iface in snapshot.get('interfaces', []):
            d = iface.get('direction', '')
            if d in ('in', 'input') and not first_in:
                first_in = sanitize_id(iface.get('label', 'Input'))
            elif d in ('out', 'output') and not first_out:
                first_out = sanitize_id(iface.get('label', 'Output'))
            elif d in ('io', 'bidirectional'):
                if not first_in:
                    first_in = sanitize_id(iface.get('label', 'Port'))
                if not first_out:
                    first_out = sanitize_id(iface.get('label', 'Port'))
    # Map the generic "inputs"/"outputs" refs
    iface_id_to_port['inputs'] = 'Input'
    iface_id_to_port['outputs'] = 'Output'

    # Emit templates
    out.append("# ─── Templates ───")
    out.append("")
    for tkey, (tname, snapshot) in template_key_to_name.items():
        emit_template(tname, snapshot, manufacturer_cards, out)
        out.append("")

    # Emit instances
    out.append("# ─── Instances ───")
    out.append("")

    # Group by groupBox if available
    group_boxes = data.get('groupBoxes', [])
    grouped_instances = set()

    for gb in group_boxes:
        gb_label = gb.get('label', 'Group')
        # Find instances within this group box bounds
        gx = gb.get('position', {}).get('x', 0)
        gy = gb.get('position', {}).get('y', 0)
        gw = gb.get('size', {}).get('width', 0)
        gh = gb.get('size', {}).get('height', 0)

        members = []
        for inst_name, tmpl_name, pd in device_template_map:
            px = pd.get('position', {}).get('x', 0)
            py = pd.get('position', {}).get('y', 0)
            if gx <= px <= gx + gw and gy <= py <= gy + gh:
                members.append((inst_name, tmpl_name, pd))

        if members:
            out.append(f"# Group: {gb_label}")
            for inst_name, tmpl_name, pd in members:
                emit_instance(inst_name, tmpl_name, pd, manufacturer_cards,
                              iface_id_to_port, out)
                out.append("")
                grouped_instances.add(inst_name)

    # Emit ungrouped instances
    for inst_name, tmpl_name, pd in device_template_map:
        if inst_name not in grouped_instances:
            emit_instance(inst_name, tmpl_name, pd, manufacturer_cards,
                          iface_id_to_port, out)
            out.append("")

    # Emit connections
    out.append("# ─── Connections ───")
    out.append("")
    for conn in data.get('connections', []):
        emit_connection(conn, instance_id_to_name, iface_id_to_port, out)

    out.append("")
    return '\n'.join(out), instance_id_to_name


def build_layout_sidecar(data: dict, instance_id_to_name: dict) -> dict:
    """Build a .layout.json sidecar preserving canvas positions.

    PatchLang doesn't encode visual layout — that lives in a companion
    .layout.json file. This extracts (x, y) from each placedDevice and
    maps them to the PatchLang instance names so the frontend can restore
    device positions on the canvas without re-running auto-layout.
    """
    positions = {}
    for pd in data.get('placedDevices', []):
        inst_id = pd.get('instanceId', '')
        inst_name = instance_id_to_name.get(inst_id)
        if not inst_name:
            continue
        pos = pd.get('position', {})
        if 'x' in pos and 'y' in pos:
            positions[inst_name] = {
                'x': round(pos['x']),
                'y': round(pos['y']),
            }
    return {
        'version': 1,
        'positions': positions,
    }


def main():
    parser = argparse.ArgumentParser(description='Convert SignalCanvas JSON to PatchLang')
    parser.add_argument('input', help='Input JSON file')
    parser.add_argument('-o', '--output', help='Output .patch file (default: stdout)')
    args = parser.parse_args()

    with open(args.input, 'r') as f:
        data = json.load(f)

    result, instance_id_to_name = convert_with_ids(data)

    if args.output:
        with open(args.output, 'w') as f:
            f.write(result)
        # Also write layout sidecar
        layout_path = args.output.replace('.patch', '.layout.json')
        layout = build_layout_sidecar(data, instance_id_to_name)
        with open(layout_path, 'w') as f:
            json.dump(layout, f, indent=2)
        print(f"Wrote {args.output}", file=sys.stderr)
        print(f"Wrote {layout_path}", file=sys.stderr)
    else:
        print(result)


if __name__ == '__main__':
    main()
