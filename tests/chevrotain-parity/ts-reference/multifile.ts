import { compile as parseToAst } from './visitor'
import { mergeAsts, type SourceFile, type MergeResult } from './merger'

export interface FileInput {
  filename: string
  source: string
}

/**
 * Parse multiple PatchLang source files and merge their ASTs.
 * Returns the merge result (merged AST + any errors).
 * The caller can then pass merged.statements to the compiler.
 */
export function parseMultiFile(files: FileInput[]): MergeResult {
  const parsed: SourceFile[] = []
  const errors: string[] = []

  for (const file of files) {
    try {
      const ast = parseToAst(file.source)
      parsed.push({ filename: file.filename, ast })
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err)
      errors.push(`Error in ${file.filename}: ${message}`)
    }
  }

  const mergeResult = mergeAsts(parsed)
  return {
    merged: mergeResult.merged,
    errors: [...errors, ...mergeResult.errors],
  }
}
