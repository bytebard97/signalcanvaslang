<!-- packages/demo/src/components/AstViewer.vue -->
<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ rawJson: string }>()

function syntaxColorJson(raw: string): string {
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return raw
  }
  return JSON.stringify(parsed, null, 2).replace(
    /("(?:\\.|[^"\\])*")\s*:|("(?:\\.|[^"\\])*")|(true|false|null)|(\d+(?:\.\d+)?)/g,
    (match, key, str, bool, num) => {
      if (key) return `<span class="aj-key">${key}</span>:`
      if (str) return `<span class="aj-string">${str}</span>`
      if (bool) return `<span class="aj-bool">${bool}</span>`
      if (num) return `<span class="aj-number">${num}</span>`
      return match
    },
  )
}

const highlighted = computed(() => {
  if (!props.rawJson) return ''
  return syntaxColorJson(props.rawJson)
})
</script>

<template>
  <div class="av">
    <pre v-if="highlighted" class="av__pre" v-html="highlighted" />
    <div v-else class="av__empty">Compile to see AST</div>
  </div>
</template>

<style scoped>
.av { height: 100%; overflow: auto; }
.av__pre {
  margin: 0;
  padding: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
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
.av__pre :deep(.aj-key) { color: #57f1db; }
.av__pre :deep(.aj-string) { color: #FFAC5A; }
.av__pre :deep(.aj-number) { color: #E0E7FF; }
.av__pre :deep(.aj-bool) { color: #60A5FA; }
</style>
