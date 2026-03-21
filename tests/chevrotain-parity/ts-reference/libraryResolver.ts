/**
 * Library Resolver — resolves template names with priority search.
 *
 * Search order:
 *   1. Project templates (user's own .patch files)
 *   2. User library paths (~/.signalcanvas/lib/, project lib/)
 *   3. Standard library (built-in device templates)
 *
 * Supports qualified names (namespace.Template) and ambiguity detection.
 */

import type { TemplateDecl } from './types'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface TemplateEntry {
  name: string
  namespace?: string
  sourceFile?: string
  template: TemplateDecl
}

export type ResolveSource = 'project' | 'userlib' | 'stdlib'

export interface ResolveResult {
  source: ResolveSource
  template: TemplateDecl
  namespace?: string
  sourceFile?: string
}

export interface LibraryResolverConfig {
  /** Templates defined in the user's project files */
  projectTemplates: Record<string, TemplateDecl>
  /** Additional library search paths (future use) */
  libraryPaths: string[]
  /** User library templates loaded from search paths */
  userlibTemplates?: Record<string, TemplateDecl>
  /** Standard library templates (built-in) */
  stdlibTemplates?: Record<string, TemplateDecl & { namespace?: string }>
}

// ---------------------------------------------------------------------------
// Resolver
// ---------------------------------------------------------------------------

export class LibraryResolver {
  private projectTemplates: Map<string, TemplateDecl>
  private userlibTemplates: Map<string, TemplateDecl>
  private stdlibByQualified: Map<string, TemplateDecl & { namespace?: string }>
  private stdlibByName: Map<string, Array<{ namespace: string; template: TemplateDecl }>>

  constructor(config: LibraryResolverConfig) {
    this.projectTemplates = new Map(Object.entries(config.projectTemplates))
    this.userlibTemplates = new Map(Object.entries(config.userlibTemplates ?? {}))

    // Build stdlib indexes
    this.stdlibByQualified = new Map()
    this.stdlibByName = new Map()

    for (const [key, tmpl] of Object.entries(config.stdlibTemplates ?? {})) {
      const ns = tmpl.namespace ?? ''
      const qualifiedKey = ns ? `${ns}.${tmpl.name}` : tmpl.name
      this.stdlibByQualified.set(qualifiedKey, tmpl)

      // Also index by bare name for unqualified lookup
      const existing = this.stdlibByName.get(tmpl.name) ?? []
      existing.push({ namespace: ns, template: tmpl })
      this.stdlibByName.set(tmpl.name, existing)

      // Store with original key too if different
      if (key !== qualifiedKey && key !== tmpl.name) {
        this.stdlibByQualified.set(key, tmpl)
      }
    }
  }

  /**
   * Resolve a template name. Supports both bare names ("CL5")
   * and qualified names ("yamaha.CL5").
   */
  resolve(name: string): ResolveResult | undefined {
    const isQualified = name.includes('.')

    // 1. Project templates (highest priority, bare name only)
    if (!isQualified) {
      const proj = this.projectTemplates.get(name)
      if (proj) {
        return { source: 'project', template: proj }
      }
    }

    // 2. User library templates
    if (!isQualified) {
      const userlib = this.userlibTemplates.get(name)
      if (userlib) {
        return { source: 'userlib', template: userlib }
      }
    }

    // 3. Stdlib — qualified lookup
    if (isQualified) {
      const stdlib = this.stdlibByQualified.get(name)
      if (stdlib) {
        return {
          source: 'stdlib',
          template: stdlib,
          namespace: stdlib.namespace,
        }
      }
      return undefined
    }

    // 3. Stdlib — bare name lookup (check for ambiguity)
    const candidates = this.stdlibByName.get(name)
    if (!candidates || candidates.length === 0) return undefined

    if (candidates.length === 1) {
      return {
        source: 'stdlib',
        template: candidates[0].template,
        namespace: candidates[0].namespace,
      }
    }

    // Ambiguous — multiple stdlib templates with same name
    return undefined
  }

  /**
   * Get all matching namespaces for an ambiguous bare name.
   * Returns empty array if name is unambiguous or not found.
   */
  getAmbiguities(name: string): string[] {
    const candidates = this.stdlibByName.get(name)
    if (!candidates || candidates.length <= 1) return []
    return candidates.map((c) => c.namespace)
  }

  /** List all available template names (for CLI search). */
  listAll(): Array<{ name: string; namespace?: string; source: ResolveSource }> {
    const results: Array<{ name: string; namespace?: string; source: ResolveSource }> = []

    for (const [name] of this.projectTemplates) {
      results.push({ name, source: 'project' })
    }
    for (const [name] of this.userlibTemplates) {
      results.push({ name, source: 'userlib' })
    }
    for (const entries of this.stdlibByName.values()) {
      for (const entry of entries) {
        results.push({ name: entry.template.name, namespace: entry.namespace, source: 'stdlib' })
      }
    }

    return results
  }

  /**
   * Search templates by query string (matches name, namespace, or meta fields).
   */
  search(query: string): Array<{ name: string; namespace?: string; source: ResolveSource }> {
    const q = query.toLowerCase()
    return this.listAll().filter((entry) => {
      if (entry.name.toLowerCase().includes(q)) return true
      if (entry.namespace?.toLowerCase().includes(q)) return true
      return false
    })
  }
}
