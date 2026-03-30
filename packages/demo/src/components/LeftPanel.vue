<!-- packages/demo/src/components/LeftPanel.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import type { CompileResult } from '@signalcanvas/diagram'
import type { TreeNode } from '../composables/useHierarchyTree'
import CodeEditor from './CodeEditor.vue'
import AstViewer from './AstViewer.vue'
import HierarchyTree from './HierarchyTree.vue'

type Tab = 'code' | 'ast' | 'hierarchy'

const props = defineProps<{
  modelValue: string
  compileResult: CompileResult | null
  hierarchyNodes: TreeNode[]
  currentTemplateName: string | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  selectNode: [payload: { instanceName: string; templateName: string }]
}>()

const activeTab = ref<Tab>('code')
const TABS: Array<{ id: Tab; label: string }> = [
  { id: 'code', label: 'Code' },
  { id: 'ast', label: 'AST' },
  { id: 'hierarchy', label: 'Hierarchy' },
]
</script>

<template>
  <div class="lp">
    <!-- Tab bar -->
    <div class="lp__tabs">
      <button
        v-for="tab in TABS"
        :key="tab.id"
        class="lp__tab"
        :class="activeTab === tab.id ? 'lp__tab--active' : ''"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- Tab content -->
    <div class="lp__body">
      <CodeEditor
        v-show="activeTab === 'code'"
        :model-value="modelValue"
        :errors="compileResult?.errors"
        :diagnostics="compileResult?.diagnostics"
        @update:model-value="emit('update:modelValue', $event)"
      />
      <AstViewer
        v-show="activeTab === 'ast'"
        :raw-json="compileResult?.rawJson ?? ''"
      />
      <HierarchyTree
        v-show="activeTab === 'hierarchy'"
        :nodes="hierarchyNodes"
        :current-template-name="currentTemplateName"
        @select="emit('selectNode', $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.lp {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0B0E13;
  border-right: 1px solid rgba(45,61,74,0.3);
  overflow: hidden;
}
.lp__tabs {
  display: flex;
  flex-shrink: 0;
  border-bottom: 1px solid rgba(45,61,74,0.3);
  background: #181C22;
}
.lp__tab {
  padding: 8px 14px;
  font-size: 12px;
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: color 0.1s, border-color 0.1s;
}
.lp__tab:hover { color: #9ca3af; }
.lp__tab--active { color: #57f1db; border-bottom-color: #57f1db; }
.lp__body { flex: 1; overflow: hidden; }
</style>
