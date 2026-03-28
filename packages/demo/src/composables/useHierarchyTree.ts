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
  visiting: Set<string> = new Set(),
): TreeNode[] {
  return instances.map(inst => {
    const tmpl = templateMap.get(inst.templateName)
    const meta = tmpl?.meta ?? {}
    const subInstances = (tmpl?.instances ?? []) as InstanceStmt[]
    const children = tmpl?.name && !visiting.has(tmpl.name)
      ? buildNodes(subInstances, templateMap, new Set([...visiting, tmpl.name]))
      : []
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

  function isInstanceStmt(s: unknown): s is InstanceStmt {
    return (
      typeof s === 'object' && s !== null &&
      (s as any).type === 'Instance' &&
      typeof (s as any).name === 'string' &&
      typeof (s as any).templateName === 'string'
    )
  }

  const rootInstances = statements.filter(isInstanceStmt)

  return buildNodes(rootInstances, templateMap)
}
