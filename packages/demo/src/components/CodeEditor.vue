<!-- packages/demo/src/components/CodeEditor.vue -->
<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, lineWrapping } from '@codemirror/view'
import { defaultKeymap } from '@codemirror/commands'
import { indentUnit } from '@codemirror/language'
import { lintGutter, setDiagnostics } from '@codemirror/lint'
import type { Diagnostic as CmDiagnostic } from '@codemirror/lint'
import type { ParseError, Diagnostic } from '@signalcanvas/diagram'
import { patchlangLanguage, patchlangDarkTheme } from '../composables/usePatchlangHighlighting'

const TAB_SIZE = 2
const SPACES_PER_TAB = ' '.repeat(TAB_SIZE)

const props = defineProps<{
  modelValue: string
  errors?: ParseError[]
  diagnostics?: Diagnostic[]
}>()

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const editorContainer = ref<HTMLElement | null>(null)
let editorView: EditorView | null = null

const syncEmitExtension = EditorView.updateListener.of((update) => {
  if (update.docChanged) {
    emit('update:modelValue', update.state.doc.toString())
  }
})

const twoSpaceTabKeymap = keymap.of([
  {
    key: 'Tab',
    run(view): boolean {
      view.dispatch(
        view.state.update({
          changes: { from: view.state.selection.main.from, insert: SPACES_PER_TAB },
          selection: { anchor: view.state.selection.main.from + TAB_SIZE },
          scrollIntoView: true,
        }),
      )
      return true
    },
  },
])

function buildCmDiagnostics(
  errors: ParseError[],
  diagnostics: Diagnostic[],
  docLength: number,
): CmDiagnostic[] {
  const result: CmDiagnostic[] = []

  for (const e of errors) {
    const from = Math.min(e.span.start, docLength)
    const to = Math.min(Math.max(e.span.end, from + 1), docLength)
    result.push({ from, to, severity: 'error', message: e.message })
  }

  for (const d of diagnostics) {
    if (!d.span) continue
    const from = Math.min(d.span.start, docLength)
    const to = Math.min(Math.max(d.span.end, from + 1), docLength)
    const severity = d.severity === 'warning' ? 'warning' : d.severity === 'info' ? 'info' : 'error'
    result.push({ from, to, severity, message: d.message })
  }

  return result
}

function pushDiagnostics(): void {
  if (!editorView) return
  const docLength = editorView.state.doc.length
  const cmDiags = buildCmDiagnostics(props.errors ?? [], props.diagnostics ?? [], docLength)
  editorView.dispatch(setDiagnostics(editorView.state, cmDiags))
}

onMounted(() => {
  if (!editorContainer.value) return

  const startState = EditorState.create({
    doc: props.modelValue,
    extensions: [
      patchlangLanguage(),
      patchlangDarkTheme,
      lineNumbers(),
      lineWrapping,
      lintGutter(),
      indentUnit.of(SPACES_PER_TAB),
      twoSpaceTabKeymap,
      keymap.of(defaultKeymap),
      syncEmitExtension,
    ],
  })

  editorView = new EditorView({ state: startState, parent: editorContainer.value })
  pushDiagnostics()
})

onBeforeUnmount(() => {
  editorView?.destroy()
  editorView = null
})

watch(
  () => props.modelValue,
  (newValue) => {
    if (!editorView) return
    if (editorView.state.doc.toString() === newValue) return
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: newValue },
    })
  },
)

watch(
  () => [props.errors, props.diagnostics],
  () => pushDiagnostics(),
  { deep: true },
)
</script>

<template>
  <div class="ce">
    <div ref="editorContainer" class="ce__editor" />
  </div>
</template>

<style scoped>
.ce { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
.ce__editor { flex: 1; overflow: hidden; }
.ce__editor :deep(.cm-editor) { height: 100%; }
.ce__editor :deep(.cm-scroller) { height: 100%; overflow: auto; }
.ce__editor :deep(::-webkit-scrollbar) { width: 4px; height: 4px; }
.ce__editor :deep(::-webkit-scrollbar-track) { background: transparent; }
.ce__editor :deep(::-webkit-scrollbar-thumb) { background: #32353b; border-radius: 10px; }
.ce__editor :deep(::-webkit-scrollbar-thumb:hover) { background: #57f1db; }
.ce__editor :deep(.cm-lintRange-error) { text-decoration: underline wavy #f87171; }
.ce__editor :deep(.cm-lintRange-warning) { text-decoration: underline wavy #fbbf24; }
.ce__editor :deep(.cm-lint-marker-error) { color: #f87171; }
.ce__editor :deep(.cm-lint-marker-warning) { color: #fbbf24; }
</style>
