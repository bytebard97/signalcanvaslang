import { describe, it, expect } from 'vitest'
import { buildHierarchyTree } from './useHierarchyTree'

const LEAF_TMPL = {
  type: 'Template', name: 'Mic',
  meta: { manufacturer: 'Shure', model: 'SM58', category: 'Microphone' },
  ports: [], bridges: [], instances: [], connects: [],
}

const MID_TMPL = {
  type: 'Template', name: 'Stage',
  meta: {},
  ports: [], bridges: [],
  instances: [
    { type: 'Instance', name: 'Mic1', templateName: 'Mic', properties: {}, args: {} },
    { type: 'Instance', name: 'Mic2', templateName: 'Mic', properties: {}, args: {} },
  ],
  connects: [],
}

const ROOT_INST_A = { type: 'Instance', name: 'StageLeft', templateName: 'Stage', properties: {}, args: {} }
const ROOT_INST_B = { type: 'Instance', name: 'StageRight', templateName: 'Stage', properties: {}, args: {} }
const ROOT_INST_C = { type: 'Instance', name: 'FOH_Mic', templateName: 'Mic', properties: {}, args: {} }

const STMTS = [LEAF_TMPL, MID_TMPL, ROOT_INST_A, ROOT_INST_B, ROOT_INST_C]

describe('buildHierarchyTree', () => {
  it('builds top-level nodes from root instances', () => {
    const tree = buildHierarchyTree(STMTS)
    expect(tree).toHaveLength(3)
    expect(tree[0].instanceName).toBe('StageLeft')
    expect(tree[1].instanceName).toBe('StageRight')
    expect(tree[2].instanceName).toBe('FOH_Mic')
  })

  it('marks composite nodes as isComposite', () => {
    const tree = buildHierarchyTree(STMTS)
    expect(tree[0].isComposite).toBe(true)   // StageLeft is a Stage (has sub-instances)
    expect(tree[2].isComposite).toBe(false)  // FOH_Mic is a Mic (leaf)
  })

  it('populates meta from the matching template', () => {
    const tree = buildHierarchyTree(STMTS)
    expect(tree[2].manufacturer).toBe('Shure')
    expect(tree[2].model).toBe('SM58')
    expect(tree[2].category).toBe('Microphone')
  })

  it('populates children for composite nodes', () => {
    const tree = buildHierarchyTree(STMTS)
    expect(tree[0].children).toHaveLength(2)
    expect(tree[0].children[0].instanceName).toBe('Mic1')
    expect(tree[0].children[0].isComposite).toBe(false)
  })

  it('returns empty array for empty statements', () => {
    expect(buildHierarchyTree([])).toEqual([])
  })
})
