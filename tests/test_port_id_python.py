"""Conformance tests for deterministic ID generation via Python bindings."""
import json
import patchlang_python

with open('tests/port_id_conformance.json') as f:
    fixture = json.load(f)

passed = 0
failed = 0

# Port ID cases
for case in fixture['port_id_cases']:
    actual = patchlang_python.generate_port_id(
        case['instance_name'],
        case['template_name'],
        case['port_name'],
        case['index'],
    )
    if actual != case['expected']:
        print(f"FAIL [port] {case['id']}: got '{actual}', expected '{case['expected']}'")
        failed += 1
    else:
        print(f"PASS [port] {case['id']}")
        passed += 1

# Route ID cases
for case in fixture['route_id_cases']:
    actual = patchlang_python.generate_route_id(
        case['template_name'],
        case['source_port'],
        case['target_port'],
    )
    if actual != case['expected']:
        print(f"FAIL [route] {case['id']}: got '{actual}', expected '{case['expected']}'")
        failed += 1
    else:
        print(f"PASS [route] {case['id']}")
        passed += 1

# Slot ID cases
for case in fixture['slot_id_cases']:
    actual = patchlang_python.generate_slot_id(
        case['template_name'],
        case['slot_name'],
    )
    if actual != case['expected']:
        print(f"FAIL [slot] {case['id']}: got '{actual}', expected '{case['expected']}'")
        failed += 1
    else:
        print(f"PASS [slot] {case['id']}")
        passed += 1

print(f"\n{passed} passed, {failed} failed out of {passed + failed} cases")
if failed > 0:
    raise SystemExit(1)
print("All Python ID conformance tests passed!")
