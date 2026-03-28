<!-- packages/demo/src/components/AstViewer.vue -->
<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ rawJson: string }>()

const formatted = computed(() => {
  if (!props.rawJson) return ''
  try {
    return JSON.stringify(JSON.parse(props.rawJson), null, 2)
  } catch {
    return props.rawJson
  }
})
</script>

<template>
  <div class="av">
    <pre v-if="formatted" class="av__pre">{{ formatted }}</pre>
    <div v-else class="av__empty">Compile to see AST</div>
  </div>
</template>

<style scoped>
.av { height: 100%; overflow: auto; }
.av__pre {
  margin: 0;
  padding: 12px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  line-height: 1.5;
  color: #9ca3af;
  white-space: pre-wrap;
  word-break: break-all;
}
.av__empty {
  padding: 16px;
  color: #4b5563;
  font-size: 12px;
  text-align: center;
  margin-top: 40px;
}
</style>
