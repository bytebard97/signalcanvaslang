export interface TreeNode {
  instanceName: string
  templateName: string
  manufacturer: string
  model: string
  category: string
  isComposite: boolean
  children: TreeNode[]
}

interface TemplateStmt {
  type: 'Template'
  name?: string
  meta?: Record<string, string>
  instances?: InstanceStmt[]
}

interface InstanceStmt {
  type: 'Instance'
  name: string
  templateName: string
}

type AnyStmt = Record<string, unknown> & { type: string }

function buildNodes(
  instances: InstanceStmt[],
  templateMap: Map<string, TemplateStmt>,
): TreeNode[] {
  return instances.map(inst => {
    const tmpl = templateMap.get(inst.templateName)
    const meta = tmpl?.meta ?? {}
    const subInstances = (tmpl?.instances ?? []) as InstanceStmt[]
    const children = buildNodes(subInstances, templateMap)
    return {
      instanceName: inst.name,
      templateName: inst.templateName,
      manufacturer: meta.manufacturer ?? '',
      model: meta.model ?? '',
      category: meta.category ?? '',
      isComposite: children.length > 0,
      children,
    }
  })
}

/**
 * Build a TreeNode hierarchy from a program's statements array.
 * Top-level Instance statements become the roots; their template's
 * sub-instances become children, recursively.
 */
export function buildHierarchyTree(statements: unknown[]): TreeNode[] {
  const templateMap = new Map<string, TemplateStmt>()
  for (const stmt of statements as AnyStmt[]) {
    if (stmt.type !== 'Template') continue
    const t = stmt as TemplateStmt
    if (t.name) templateMap.set(t.name, t)
  }

  const rootInstances = (statements as AnyStmt[])
    .filter(s => s.type === 'Instance') as InstanceStmt[]

  return buildNodes(rootInstances, templateMap)
}
