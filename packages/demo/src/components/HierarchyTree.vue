<!-- packages/demo/src/components/HierarchyTree.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import type { TreeNode } from '../composables/useHierarchyTree'
import HierarchyTreeRow from './HierarchyTreeRow.vue'

const props = defineProps<{
  nodes: TreeNode[]
  currentTemplateName: string | null
}>()

const emit = defineEmits<{
  select: [payload: { instanceName: string; templateName: string }]
}>()

const hideTypes = ref(false)
const expanded = ref<Set<string>>(new Set())

function toggleExpand(key: string): void {
  if (expanded.value.has(key)) {
    expanded.value.delete(key)
  } else {
    expanded.value.add(key)
  }
}
</script>

<template>
  <div class="ht">
    <!-- Header with "Hide types" toggle -->
    <div class="ht__header">
      <span class="ht__header-label">Hierarchy</span>
      <button
        class="ht__toggle"
        :class="hideTypes ? 'ht__toggle--active' : ''"
        @click="hideTypes = !hideTypes"
        title="Toggle manufacturer/model/category columns"
      >
        {{ hideTypes ? 'Show types' : 'Hide types' }}
      </button>
    </div>

    <!-- Column headers -->
    <div class="ht__cols" :class="hideTypes ? 'ht__cols--compact' : ''">
      <span class="ht__col-h ht__col-h--name">Instance</span>
      <template v-if="!hideTypes">
        <span class="ht__col-h">Manufacturer</span>
        <span class="ht__col-h">Model</span>
        <span class="ht__col-h">Category</span>
      </template>
    </div>

    <!-- Tree -->
    <div class="ht__body">
      <template v-if="nodes.length > 0">
        <HierarchyTreeRow
          v-for="node in nodes"
          :key="node.instanceName"
          :node="node"
          :depth="0"
          :path="[node.instanceName]"
          :hide-types="hideTypes"
          :expanded="expanded"
          :current-template-name="currentTemplateName"
          @select="emit('select', $event)"
          @toggle="toggleExpand"
        />
      </template>
      <div v-else class="ht__empty">Compile to see hierarchy</div>
    </div>
  </div>
</template>

<style scoped>
.ht { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
.ht__header { display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; border-bottom: 1px solid rgba(45,61,74,0.3); flex-shrink: 0; }
.ht__header-label { font-size: 10px; text-transform: uppercase; letter-spacing: 0.1em; color: #9ca3af; font-weight: 600; }
.ht__toggle { font-size: 10px; background: rgba(87,241,219,0.08); border: 1px solid rgba(87,241,219,0.15); color: #57f1db; padding: 2px 8px; border-radius: 4px; cursor: pointer; }
.ht__toggle--active { background: rgba(87,241,219,0.15); }
.ht__cols { display: grid; grid-template-columns: 1fr 100px 68px 76px; padding: 4px 10px; border-bottom: 1px solid rgba(45,61,74,0.3); flex-shrink: 0; }
.ht__cols--compact { grid-template-columns: 1fr; }
.ht__col-h { font-size: 10px; color: #4b5563; text-transform: uppercase; letter-spacing: 0.08em; }
.ht__col-h--name { color: #6b7280; }
.ht__body { overflow-y: auto; flex: 1; }
.ht__empty { padding: 16px; color: #4b5563; font-size: 11px; text-align: center; margin-top: 20px; }
</style>
