<!-- packages/demo/src/components/HierarchyTreeRow.vue -->
<script setup lang="ts">
import type { TreeNode } from '../composables/useHierarchyTree'

const props = defineProps<{
  node: TreeNode
  depth: number
  path: string[]
  hideTypes: boolean
  expanded: Set<string>
  currentTemplateName: string | null
}>()

const emit = defineEmits<{
  select: [payload: { instanceName: string; templateName: string }]
  toggle: [key: string]
}>()

function key(): string {
  return props.path.join('/')
}

function isExpanded(): boolean {
  return props.expanded.has(key())
}

function isCurrent(): boolean {
  return props.currentTemplateName !== null &&
    props.node.templateName === props.currentTemplateName
}

function handleClick(): void {
  if (props.node.isComposite) {
    emit('toggle', key())
  }
  emit('select', { instanceName: props.node.instanceName, templateName: props.node.templateName })
}
</script>

<template>
  <div>
    <!-- Row -->
    <div
      class="htr"
      :class="isCurrent() ? 'htr--current' : ''"
      :style="{ paddingLeft: `${10 + depth * 16}px` }"
      @click="handleClick"
    >
      <div class="htr__name-cell" :class="hideTypes ? 'htr__name-cell--full' : ''">
        <span class="htr__chevron">
          <template v-if="node.isComposite">{{ isExpanded() ? '▾' : '▸' }}</template>
          <template v-else>●</template>
        </span>
        <span class="htr__name">{{ node.instanceName }}</span>
        <span class="htr__type-name">{{ node.templateName }}</span>
      </div>
      <template v-if="!hideTypes">
        <span class="htr__meta">{{ node.manufacturer || '—' }}</span>
        <span class="htr__meta">{{ node.model || '—' }}</span>
        <span class="htr__meta">{{ node.category || '—' }}</span>
      </template>
    </div>

    <!-- Children (recursive) -->
    <template v-if="node.isComposite && isExpanded()">
      <HierarchyTreeRow
        v-for="child in node.children"
        :key="child.instanceName"
        :node="child"
        :depth="depth + 1"
        :path="[...path, child.instanceName]"
        :hide-types="hideTypes"
        :expanded="expanded"
        :current-template-name="currentTemplateName"
        @select="emit('select', $event)"
        @toggle="emit('toggle', $event)"
      />
    </template>
  </div>
</template>

<style scoped>
.htr {
  display: grid;
  grid-template-columns: 1fr 80px 80px 80px;
  align-items: center;
  padding-top: 5px;
  padding-right: 10px;
  padding-bottom: 5px;
  cursor: pointer;
  border-bottom: 1px solid rgba(45,61,74,0.12);
  transition: background 0.1s;
}
.htr:hover { background: rgba(87,241,219,0.05); }
.htr--current { background: rgba(87,241,219,0.08); }
.htr__name-cell { display: flex; align-items: center; gap: 6px; min-width: 0; }
.htr__name-cell--full { grid-column: 1 / -1; }
.htr__chevron { color: #57f1db; font-size: 10px; flex-shrink: 0; width: 12px; text-align: center; }
.htr__name { font-family: monospace; font-size: 11px; color: #e5e7eb; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.htr__type-name { font-size: 10px; color: #4b5563; white-space: nowrap; }
.htr__meta { font-size: 10px; color: #6b7280; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: monospace; }
</style>
