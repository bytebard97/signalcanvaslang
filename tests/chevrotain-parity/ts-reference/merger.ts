import type { PatchProgram, Statement, TemplateDecl, InstanceDecl } from './types'

export interface SourceFile {
  filename: string
  ast: PatchProgram
}

export interface MergeResult {
  merged: PatchProgram
  errors: string[]
}

/**
 * Merge multiple parsed PatchLang ASTs into a single program.
 * - Collects all statements from all files
 * - Tags each template with its sourceFile
 * - Detects template name collisions across files
 * - Detects instance name collisions across files
 * - Returns merged AST and any errors
 */
export function mergeAsts(files: SourceFile[]): MergeResult {
  const errors: string[] = []
  const allStatements: Statement[] = []

  // Track template names → source file for collision detection
  const templateSources = new Map<string, string>()
  // Track instance names → source file for collision detection
  const instanceSources = new Map<string, string>()

  for (const file of files) {
    for (const stmt of file.ast.statements) {
      if (stmt.type === 'Template') {
        const tmpl = stmt as TemplateDecl
        const existing = templateSources.get(tmpl.name)
        if (existing) {
          errors.push(
            `Template '${tmpl.name}' defined in both '${existing}' and '${file.filename}'`,
          )
        } else {
          templateSources.set(tmpl.name, file.filename)
        }
        // Tag the template with its source file.
        // sourceFile is defined as optional on TemplateDecl, so we can assign directly.
        tmpl.sourceFile = file.filename
      }

      if (stmt.type === 'Instance') {
        const inst = stmt as InstanceDecl
        const existing = instanceSources.get(inst.name)
        if (existing) {
          errors.push(
            `Instance '${inst.name}' defined in both '${existing}' and '${file.filename}'`,
          )
        } else {
          instanceSources.set(inst.name, file.filename)
        }
      }

      allStatements.push(stmt)
    }
  }

  return {
    merged: { type: 'Program', statements: allStatements },
    errors,
  }
}
