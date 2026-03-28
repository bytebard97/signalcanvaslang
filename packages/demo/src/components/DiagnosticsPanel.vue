<!-- packages/demo/src/components/DiagnosticsPanel.vue -->
<script setup lang="ts">
import type { ParseError, Diagnostic } from '@signalcanvas/diagram'

defineProps<{
  errors: ParseError[]
  diagnostics: Diagnostic[]
}>()

const emit = defineEmits<{ close: [] }>()

const SEVER_COLOR: Record<string, string> = {
  error: '#f87171',
  warning: '#fbbf24',
  info: '#60a5fa',
}
</script>

<template>
  <div class="dp">
    <div class="dp__header">
      <span class="dp__title">Diagnostics</span>
      <button class="dp__close" @click="emit('close')">✕</button>
    </div>
    <div class="dp__body">
      <div
        v-for="(err, i) in errors"
        :key="'e' + i"
        class="dp__row dp__row--error"
      >
        <span class="dp__badge" style="color: #f87171">error</span>
        <span class="dp__msg">{{ err.message }}</span>
        <span class="dp__loc">{{ err.span.start }}–{{ err.span.end }}</span>
      </div>
      <div
        v-for="(d, i) in diagnostics"
        :key="'d' + i"
        class="dp__row"
      >
        <span class="dp__badge" :style="{ color: SEVER_COLOR[d.severity] ?? '#9ca3af' }">{{ d.severity }}</span>
        <span class="dp__msg">{{ d.message }}</span>
        <span v-if="d.code" class="dp__code">{{ d.code }}</span>
      </div>
      <div v-if="errors.length === 0 && diagnostics.length === 0" class="dp__empty">
        No diagnostics
      </div>
    </div>
  </div>
</template>

<style scoped>
.dp { display: flex; flex-direction: column; background: #181C22; border-top: 1px solid rgba(45,61,74,0.4); max-height: 200px; }
.dp__header { display: flex; align-items: center; justify-content: space-between; padding: 6px 12px; border-bottom: 1px solid rgba(45,61,74,0.3); }
.dp__title { font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: #9ca3af; font-weight: 600; }
.dp__close { background: none; border: none; color: #4b5563; cursor: pointer; font-size: 12px; padding: 2px 4px; }
.dp__close:hover { color: #9ca3af; }
.dp__body { overflow-y: auto; flex: 1; }
.dp__row { display: flex; align-items: baseline; gap: 8px; padding: 5px 12px; border-bottom: 1px solid rgba(45,61,74,0.15); font-size: 11px; }
.dp__row--error { background: rgba(248,113,113,0.04); }
.dp__badge { font-weight: 700; font-size: 10px; text-transform: uppercase; letter-spacing: 0.05em; min-width: 48px; }
.dp__msg { color: #e5e7eb; flex: 1; }
.dp__loc, .dp__code { color: #6b7280; font-family: monospace; flex-shrink: 0; }
.dp__empty { padding: 12px; color: #4b5563; text-align: center; font-size: 11px; }
</style>
