// Quick smoke test for the WASM package
import { parse, validate } from '../pkg-node/patchlang_wasm.js'
import { readFileSync } from 'fs'

// Test 1: Simple parse
const result = JSON.parse(parse('instance FOH is CL5'))
console.assert(result.errors.length === 0, 'Expected no errors')
console.assert(result.program.statements.length === 1, 'Expected 1 statement')
console.assert(result.program.statements[0].type === 'Instance', 'Expected Instance')
console.assert(result.program.statements[0].name === 'FOH', 'Expected FOH')
console.log('PASS: simple instance')

// Test 2: Validate
console.assert(validate('instance FOH is CL5') === true, 'Expected valid')
console.assert(validate('!!! garbage') === false, 'Expected invalid')
console.log('PASS: validate')

// Test 3: Parse real fixture file
const worship = readFileSync('tests/fixtures/examples/worship-venue.patch', 'utf-8')
const worshipResult = JSON.parse(parse(worship))
console.assert(worshipResult.errors.length === 0, 'Expected no errors for worship-venue')
const types = {}
for (const s of worshipResult.program.statements) {
  types[s.type] = (types[s.type] || 0) + 1
}
console.assert(types.Template === 3, `Expected 3 templates, got ${types.Template}`)
console.assert(types.Instance === 4, `Expected 4 instances, got ${types.Instance}`)
console.log('PASS: worship-venue.patch')

// Test 4: Parse Hillsong MTG (1485 lines)
const hillsong = readFileSync('tests/fixtures/examples/hillsong-mtg.patch', 'utf-8')
const hillsongResult = JSON.parse(parse(hillsong))
console.assert(hillsongResult.errors.length === 0, `Expected 0 errors, got ${hillsongResult.errors.length}`)
const hTypes = {}
for (const s of hillsongResult.program.statements) {
  hTypes[s.type] = (hTypes[s.type] || 0) + 1
}
console.assert(hTypes.Template === 24, `Expected 24 templates, got ${hTypes.Template}`)
console.assert(hTypes.Instance === 53, `Expected 53 instances, got ${hTypes.Instance}`)
console.assert(hTypes.Connect === 99, `Expected 99 connects, got ${hTypes.Connect}`)
console.log('PASS: hillsong-mtg.patch (1485 lines, 203 statements)')

console.log('\nAll WASM tests passed!')
