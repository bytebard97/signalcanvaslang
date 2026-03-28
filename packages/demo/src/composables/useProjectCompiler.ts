// packages/demo/src/composables/useProjectCompiler.ts
import { ref } from 'vue'
import type { CompileResult } from '@signalcanvas/diagram'

export function useProjectCompiler() {
  const isReady = ref(false)
  const isCompiling = ref(false)
  const lastResult = ref<CompileResult | null>(null)

  let checkFn: ((source: string) => string) | null = null
  let compileProjectFn: ((filesJson: string, entry: string) => string) | null = null

  async function init(): Promise<void> {
    try {
      const wasm = await import('patchlang-wasm')
      await (wasm as any).default({ module_or_path: '/patchlang_wasm_bg.wasm' })
      checkFn = (wasm as any).check ?? (wasm as any).parse ?? null
      compileProjectFn = (wasm as any).compile_project ?? null
      isReady.value = true
    } catch (e) {
      console.error('[useProjectCompiler] failed to load WASM:', e)
    }
  }

  function compileSingle(source: string): CompileResult {
    if (!checkFn) return notLoaded()
    isCompiling.value = true
    return parseRaw(checkFn(source))
  }

  function compileMulti(files: Record<string, string>, entry: string): CompileResult {
    if (!compileProjectFn) return notLoaded()
    isCompiling.value = true
    const filesJson = JSON.stringify(files)
    return parseRaw(compileProjectFn(filesJson, entry))
  }

  function parseRaw(json: string): CompileResult {
    try {
      const raw = JSON.parse(json)
      const errors = raw.errors ?? []
      const diagnostics = raw.diagnostics ?? []
      const result: CompileResult = {
        success: errors.length === 0 && !diagnostics.some((d: { severity: string }) => d.severity === 'error'),
        program: raw.program ?? null,
        errors,
        diagnostics,
        rawJson: json,
      }
      lastResult.value = result
      return result
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : 'JSON parse error'
      const result: CompileResult = {
        success: false,
        program: null,
        errors: [{ message, span: { start: 0, end: 0 } }],
        diagnostics: [],
        rawJson: json,
      }
      lastResult.value = result
      return result
    } finally {
      isCompiling.value = false
    }
  }

  function notLoaded(): CompileResult {
    return {
      success: false,
      program: null,
      errors: [{ message: 'Compiler not loaded', span: { start: 0, end: 0 } }],
      diagnostics: [],
      rawJson: '',
    }
  }

  return { isReady, isCompiling, lastResult, init, compileSingle, compileMulti }
}
