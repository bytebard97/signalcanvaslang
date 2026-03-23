// Conformance tests for deterministic ID generation via WASM.
//
// WASM binding convention: generate_port_id accepts `index` as i32.
// Pass -1 for single (non-ranged) ports (maps to None in Rust).
// Pass a non-negative value for ranged ports.

import { generate_port_id, generate_route_id, generate_slot_id } from '../pkg-node/patchlang_wasm.js'
import { readFileSync } from 'fs'

const fixture = JSON.parse(readFileSync('tests/port_id_conformance.json', 'utf-8'))

let passed = 0
let failed = 0

// Port ID cases
for (const c of fixture.port_id_cases) {
  const wasmIndex = c.index === null ? -1 : c.index
  const actual = generate_port_id(c.instance_name, c.template_name, c.port_name, wasmIndex)
  if (actual !== c.expected) {
    console.error(`FAIL [port] ${c.id}: got '${actual}', expected '${c.expected}'`)
    failed++
  } else {
    console.log(`PASS [port] ${c.id}`)
    passed++
  }
}

// Route ID cases
for (const c of fixture.route_id_cases) {
  const actual = generate_route_id(c.template_name, c.source_port, c.target_port)
  if (actual !== c.expected) {
    console.error(`FAIL [route] ${c.id}: got '${actual}', expected '${c.expected}'`)
    failed++
  } else {
    console.log(`PASS [route] ${c.id}`)
    passed++
  }
}

// Slot ID cases
for (const c of fixture.slot_id_cases) {
  const actual = generate_slot_id(c.template_name, c.slot_name)
  if (actual !== c.expected) {
    console.error(`FAIL [slot] ${c.id}: got '${actual}', expected '${c.expected}'`)
    failed++
  } else {
    console.log(`PASS [slot] ${c.id}`)
    passed++
  }
}

console.log(`\n${passed} passed, ${failed} failed out of ${passed + failed} cases`)
if (failed > 0) {
  process.exit(1)
}
console.log('All WASM ID conformance tests passed!')
