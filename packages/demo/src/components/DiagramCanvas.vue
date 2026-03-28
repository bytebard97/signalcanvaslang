<!-- packages/demo/src/components/DiagramCanvas.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { FlowDiagram, transformAstToFlow } from '@signalcanvas/diagram'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'
import '@vue-flow/minimap/dist/style.css'
import type { CompileResult } from '@signalcanvas/diagram'
import { extractLevelFlow, buildCompositeTemplateNames } from '../lib/extractLevelFlow'
import type { NavEntry } from '../composables/useNavigation'

const props = defineProps<{
  compileResult: CompileResult | null
  currentTemplateName: string | null
}>()

const emit = defineEmits<{
  drill: [payload: NavEntry]
}>()

const allStatements = computed((): unknown[] => {
  const program = props.compileResult?.program as { statements?: unknown[] } | null
  return program?.statements ?? []
})

const compositeTemplateNames = computed(() =>
  buildCompositeTemplateNames(allStatements.value),
)

const levelResult = computed(() => {
  if (!props.compileResult?.success) return null
  return extractLevelFlow(props.currentTemplateName, allStatements.value)
})

const flowGraph = computed(() => {
  if (!levelResult.value) return { nodes: [], edges: [] }
  // Cast: LevelResult shape is structurally identical to CompileResult
  const graph = transformAstToFlow(levelResult.value as unknown as CompileResult)
  // Mark drillable nodes
  const nodes = graph.nodes.map(n => ({
    ...n,
    data: {
      ...n.data,
      drillable: compositeTemplateNames.value.has((n.data as any).templateName),
    },
  }))
  return { nodes, edges: graph.edges }
})

function onDrill(payload: { instanceName: string; templateName: string }): void {
  emit('drill', payload)
}
</script>

<template>
  <div class="dc">
    <div v-if="!compileResult" class="dc__placeholder">
      Load a project and press Compile to view the diagram
    </div>
    <div v-else-if="!compileResult.success" class="dc__placeholder">
      Fix compile errors to view the diagram
    </div>
    <FlowDiagram
      v-else
      :nodes="flowGraph.nodes"
      :edges="flowGraph.edges"
      mode="wires"
      @drill="onDrill"
    />
  </div>
</template>

<style scoped>
.dc { width: 100%; height: 100%; position: relative; }
.dc__placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #4b5563;
  font-size: 13px;
}
</style>
