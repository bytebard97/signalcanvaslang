// packages/demo/src/lib/extractLevelFlow.ts

// Matches the shape required by transformAstToFlow (structurally compatible with
// CompileResult from @signalcanvas/diagram).
export interface LevelResult {
  success: boolean
  program: { statements: unknown[] } | null
  errors: Array<{ message: string; span: { start: number; end: number } }>
  diagnostics: Array<{ code: string; message: string; severity: string }>
  rawJson: string
}

interface PortRef {
  instance: string
  port: string
  indexSpec?: unknown[]
}

interface NestedConnect {
  type: 'Connect'
  source: PortRef
  target: PortRef
  properties?: Record<string, string>
}

interface NestedInstance {
  type: 'Instance'
  name: string
  templateName: string
  properties?: Record<string, string>
  args?: Record<string, unknown>
}

interface NestedTemplate {
  type: 'Template'
  name?: string
  meta?: Record<string, string>
  ports?: unknown[]
  bridges?: unknown[]
  instances?: NestedInstance[]
  connects?: NestedConnect[]
}

type AnyStmt = Record<string, unknown> & { type: string }

/**
 * Build a synthetic LevelResult for a single template level.
 *
 * - templateName === null  → root level (all top-level statements unchanged)
 * - templateName === 'Foo' → interior of template 'Foo'
 *
 * Boundary connects (source.instance === '' or target.instance === '')
 * are excluded because there is no node to connect them to at this level.
 *
 * Returns success:false if the named template is not found.
 */
export function extractLevelFlow(
  templateName: string | null,
  allStatements: unknown[],
): LevelResult {
  const stmts = allStatements as AnyStmt[]
  const empty: LevelResult = { success: false, program: null, errors: [], diagnostics: [], rawJson: '' }

  if (templateName === null) {
    return { success: true, program: { statements: stmts }, errors: [], diagnostics: [], rawJson: '' }
  }

  const templateDecls = stmts.filter(s => s.type === 'Template') as NestedTemplate[]
  const target = templateDecls.find(t => t.name === templateName)
  if (!target) return empty

  const instances = target.instances ?? []
  const instanceNames = new Set(instances.map(i => i.name))

  const connects = (target.connects ?? []).filter(
    c =>
      c.source.instance !== '' && instanceNames.has(c.source.instance) &&
      c.target.instance !== '' && instanceNames.has(c.target.instance),
  )

  return {
    success: true,
    program: {
      statements: [
        ...templateDecls,  // all template defs needed for port lookups
        ...instances,
        ...connects,
      ],
    },
    errors: [],
    diagnostics: [],
    rawJson: '',
  }
}

/**
 * Returns a Set of template names that have sub-instances (are composite).
 * Used by DiagramCanvas to mark nodes as drillable.
 */
export function buildCompositeTemplateNames(allStatements: unknown[]): Set<string> {
  const result = new Set<string>()
  for (const stmt of allStatements as AnyStmt[]) {
    if (stmt.type !== 'Template') continue
    const t = stmt as NestedTemplate
    if (t.name && (t.instances?.length ?? 0) > 0) result.add(t.name)
  }
  return result
}
