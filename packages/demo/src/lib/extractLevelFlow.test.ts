import { describe, it, expect } from 'vitest'
import { extractLevelFlow, buildCompositeTemplateNames } from './extractLevelFlow'

const LEAF = {
  type: 'Template', name: 'Leaf',
  meta: { manufacturer: 'Acme', model: 'X1', category: 'Device' },
  ports: [], bridges: [], instances: [], connects: [],
}

const SUBSYSTEM = {
  type: 'Template', name: 'Subsystem',
  meta: {},
  ports: [], bridges: [],
  instances: [
    { type: 'Instance', name: 'Dev1', templateName: 'Leaf', properties: {}, args: {} },
    { type: 'Instance', name: 'Dev2', templateName: 'Leaf', properties: {}, args: {} },
  ],
  connects: [
    // instance-to-instance — should be kept
    { type: 'Connect', source: { instance: 'Dev1', port: 'Out' }, target: { instance: 'Dev2', port: 'In' }, properties: {} },
    // boundary connect (empty instance) — should be dropped
    { type: 'Connect', source: { instance: '', port: 'BoundaryIn' }, target: { instance: 'Dev1', port: 'In' }, properties: {} },
  ],
}

const ROOT_INSTANCE_A = { type: 'Instance', name: 'Sub1', templateName: 'Subsystem', properties: {}, args: {} }
const ROOT_INSTANCE_B = { type: 'Instance', name: 'Dev3', templateName: 'Leaf', properties: {}, args: {} }
const ROOT_CONNECT = {
  type: 'Connect',
  source: { instance: 'Sub1', port: 'Out' },
  target: { instance: 'Dev3', port: 'In' },
  properties: {},
}

const ALL_STATEMENTS = [LEAF, SUBSYSTEM, ROOT_INSTANCE_A, ROOT_INSTANCE_B, ROOT_CONNECT]

describe('extractLevelFlow', () => {
  it('root level returns all top-level statements as-is', () => {
    const result = extractLevelFlow(null, ALL_STATEMENTS)
    expect(result.success).toBe(true)
    const stmts = (result.program as any).statements
    expect(stmts).toHaveLength(5)
    expect(stmts.filter((s: any) => s.type === 'Instance')).toHaveLength(2)
    expect(stmts.filter((s: any) => s.type === 'Connect')).toHaveLength(1)
  })

  it('drill level returns template body instances and filtered connects', () => {
    const result = extractLevelFlow('Subsystem', ALL_STATEMENTS)
    expect(result.success).toBe(true)
    const stmts = (result.program as any).statements
    // 2 template decls + 2 instances + 1 instance-to-instance connect (boundary dropped)
    expect(stmts.filter((s: any) => s.type === 'Template')).toHaveLength(2)
    expect(stmts.filter((s: any) => s.type === 'Instance')).toHaveLength(2)
    expect(stmts.filter((s: any) => s.type === 'Connect')).toHaveLength(1)
    const connect = stmts.find((s: any) => s.type === 'Connect')
    expect(connect.source.instance).toBe('Dev1')
    expect(connect.target.instance).toBe('Dev2')
  })

  it('returns success:false for unknown template', () => {
    const result = extractLevelFlow('NonExistent', ALL_STATEMENTS)
    expect(result.success).toBe(false)
    expect(result.program).toBeNull()
  })

  it('leaf template with no instances returns empty nodes/connects', () => {
    const result = extractLevelFlow('Leaf', ALL_STATEMENTS)
    expect(result.success).toBe(true)
    const stmts = (result.program as any).statements
    expect(stmts.filter((s: any) => s.type === 'Instance')).toHaveLength(0)
    expect(stmts.filter((s: any) => s.type === 'Connect')).toHaveLength(0)
  })
})

describe('buildCompositeTemplateNames', () => {
  it('returns names of templates that have sub-instances', () => {
    const composites = buildCompositeTemplateNames(ALL_STATEMENTS)
    expect(composites.has('Subsystem')).toBe(true)
    expect(composites.has('Leaf')).toBe(false)
  })
})
