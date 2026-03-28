<!-- packages/demo/src/App.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { CompileResult } from '@signalcanvas/diagram'
import { useProjectCompiler } from './composables/useProjectCompiler'
import { buildHierarchyTree } from './composables/useHierarchyTree'
import { useNavigation } from './composables/useNavigation'
import { EXAMPLES } from './lib/examples'
import LeftPanel from './components/LeftPanel.vue'
import DiagramCanvas from './components/DiagramCanvas.vue'
import BreadcrumbBar from './components/BreadcrumbBar.vue'
import DiagnosticsPanel from './components/DiagnosticsPanel.vue'

// ── Compiler ─────────────────────────────────────────────────────────────────

const compiler = useProjectCompiler()
const compileResult = ref<CompileResult | null>(null)
const showDiagnostics = ref(false)

onMounted(() => compiler.init())

// ── Source code ───────────────────────────────────────────────────────────────

const source = ref(EXAMPLES[0].kind === 'single' ? EXAMPLES[0].source : '')
const selectedExampleIndex = ref(0)

function loadExample(index: number): void {
  selectedExampleIndex.value = index
  const ex = EXAMPLES[index]
  if (ex.kind === 'single') {
    source.value = ex.source
  } else {
    // Show entry file in editor
    source.value = ex.files[ex.entry] ?? ''
  }
  compileResult.value = null
  navigation.reset()
}

// ── Compile ───────────────────────────────────────────────────────────────────

function compile(): void {
  const ex = EXAMPLES[selectedExampleIndex.value]
  let result: CompileResult
  if (ex.kind === 'multi') {
    result = compiler.compileMulti(ex.files, ex.entry)
  } else {
    result = compiler.compileSingle(source.value)
  }
  compileResult.value = result
  navigation.reset()
  showDiagnostics.value = result.errors.length > 0 || result.diagnostics.some(d => d.severity === 'error')
}

// ── Directory picker ──────────────────────────────────────────────────────────

const supportsFilePicker = typeof (window as any).showDirectoryPicker === 'function'

async function openDirectory(): Promise<void> {
  const dirHandle = await (window as any).showDirectoryPicker()
  const files: Record<string, string> = {}
  for await (const [name, handle] of dirHandle.entries()) {
    if (handle.kind === 'file' && name.endsWith('.patch')) {
      const file = await handle.getFile()
      files[name] = await file.text()
    }
  }
  if (Object.keys(files).length === 0) return
  // Entry: main.patch first, else alphabetically first
  const entry = files['main.patch']
    ? 'main.patch'
    : Object.keys(files).sort()[0]
  source.value = files[entry] ?? ''
  const result = compiler.compileMulti(files, entry)
  compileResult.value = result
  navigation.reset()
  showDiagnostics.value = result.errors.length > 0
}

// ── Hierarchy tree ────────────────────────────────────────────────────────────

const allStatements = computed((): unknown[] => {
  const program = compileResult.value?.program as { statements?: unknown[] } | null
  return program?.statements ?? []
})

const hierarchyNodes = computed(() => buildHierarchyTree(allStatements.value))

// ── Navigation ────────────────────────────────────────────────────────────────

const navigation = useNavigation()

function onDrill(payload: { instanceName: string; templateName: string }): void {
  navigation.drillInto(payload)
}

function onNavigate(index: number): void {
  navigation.navigateTo(index)
}

function onSelectNode(payload: { instanceName: string; templateName: string }): void {
  navigation.drillInto(payload)
}

// ── Diagnostics ───────────────────────────────────────────────────────────────

const hasIssues = computed(() =>
  (compileResult.value?.errors.length ?? 0) > 0 ||
  (compileResult.value?.diagnostics.some(d => d.severity === 'error') ?? false),
)

watch(hasIssues, v => { if (v) showDiagnostics.value = true })
</script>

<template>
  <div class="app">
    <!-- Toolbar -->
    <header class="app__toolbar">
      <span class="app__brand">PatchLang Demo</span>
      <select
        class="app__select"
        :value="selectedExampleIndex"
        @change="loadExample(Number(($event.target as HTMLSelectElement).value))"
      >
        <option v-for="(ex, i) in EXAMPLES" :key="i" :value="i">{{ ex.name }}</option>
      </select>
      <button
        v-if="supportsFilePicker"
        class="app__btn"
        @click="openDirectory"
      >
        Open Directory
      </button>
      <button
        class="app__btn app__btn--primary"
        :disabled="!compiler.isReady.value"
        @click="compile"
      >
        {{ compiler.isReady.value ? 'Compile' : 'Loading WASM…' }}
      </button>
      <span v-if="hasIssues" class="app__error-badge">
        {{ (compileResult?.errors.length ?? 0) }} error(s)
      </span>
    </header>

    <!-- Main layout -->
    <div class="app__body">
      <!-- Left panel -->
      <div class="app__left">
        <LeftPanel
          v-model="source"
          :compile-result="compileResult"
          :hierarchy-nodes="hierarchyNodes"
          :current-template-name="navigation.currentTemplateName.value"
          @select-node="onSelectNode"
        />
      </div>

      <!-- Canvas + breadcrumb -->
      <div class="app__canvas-col">
        <BreadcrumbBar
          :breadcrumbs="navigation.breadcrumbs.value"
          @navigate="onNavigate"
        />
        <div class="app__canvas">
          <DiagramCanvas
            :compile-result="compileResult"
            :current-template-name="navigation.currentTemplateName.value"
            @drill="onDrill"
          />
        </div>
        <!-- Diagnostics panel -->
        <DiagnosticsPanel
          v-if="showDiagnostics"
          :errors="compileResult?.errors ?? []"
          :diagnostics="compileResult?.diagnostics ?? []"
          @close="showDiagnostics = false"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--bg);
}

.app__toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 16px;
  height: 46px;
  background: #181C22;
  border-bottom: 1px solid rgba(45,61,74,0.4);
  flex-shrink: 0;
}
.app__brand {
  font-family: monospace;
  font-size: 13px;
  font-weight: 700;
  color: #57f1db;
  letter-spacing: -0.02em;
  margin-right: 8px;
}
.app__select {
  background: rgba(11,14,19,0.5);
  border: 1px solid rgba(45,61,74,0.4);
  color: #9ca3af;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 6px;
  outline: none;
}
.app__btn {
  padding: 5px 12px;
  font-size: 12px;
  background: rgba(45,61,74,0.3);
  border: 1px solid rgba(45,61,74,0.4);
  color: #9ca3af;
  border-radius: 6px;
  cursor: pointer;
  transition: color 0.1s, border-color 0.1s;
}
.app__btn:hover { color: #e5e7eb; border-color: rgba(87,241,219,0.3); }
.app__btn:disabled { opacity: 0.5; cursor: not-allowed; }
.app__btn--primary { color: #57f1db; border-color: rgba(87,241,219,0.3); }
.app__btn--primary:hover { background: rgba(87,241,219,0.1); }
.app__error-badge {
  font-size: 11px;
  color: #f87171;
  background: rgba(248,113,113,0.1);
  padding: 2px 8px;
  border-radius: 4px;
}

.app__body {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.app__left {
  width: 400px;
  min-width: 280px;
  flex-shrink: 0;
  overflow: hidden;
}
.app__canvas-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.app__canvas {
  flex: 1;
  overflow: hidden;
}
</style>
