<!-- packages/demo/src/components/BreadcrumbBar.vue -->
<script setup lang="ts">
defineProps<{
  breadcrumbs: Array<{ label: string; index: number }>
  mode: 'wires' | 'netnames'
}>()

const emit = defineEmits<{
  navigate: [index: number]
  'update:mode': [mode: 'wires' | 'netnames']
}>()
</script>

<template>
  <div class="bb">
    <div class="bb__crumbs">
      <template v-for="(crumb, i) in breadcrumbs" :key="crumb.index">
        <span
          class="bb__crumb"
          :class="i === breadcrumbs.length - 1 ? 'bb__crumb--active' : ''"
          @click="emit('navigate', crumb.index)"
        >{{ crumb.label }}</span>
        <span v-if="i < breadcrumbs.length - 1" class="bb__sep">›</span>
      </template>
    </div>

    <div class="bb__mode">
      <button
        class="bb__mode-btn"
        :class="mode === 'wires' ? 'bb__mode-btn--active' : ''"
        @click="emit('update:mode', 'wires')"
      >Wires</button>
      <button
        class="bb__mode-btn"
        :class="mode === 'netnames' ? 'bb__mode-btn--active' : ''"
        @click="emit('update:mode', 'netnames')"
      >Net Names</button>
    </div>
  </div>
</template>

<style scoped>
.bb {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 14px;
  background: #181C22;
  border-bottom: 1px solid rgba(45,61,74,0.3);
  flex-shrink: 0;
  min-height: 34px;
}
.bb__crumbs { display: flex; align-items: center; gap: 4px; }
.bb__crumb {
  font-family: monospace;
  font-size: 12px;
  color: #9ca3af;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: color 0.1s;
}
.bb__crumb:hover { color: #57f1db; background: rgba(87,241,219,0.06); }
.bb__crumb--active { color: #57f1db; cursor: default; }
.bb__crumb--active:hover { background: none; }
.bb__sep { color: #374151; font-size: 12px; }

.bb__mode {
  display: flex;
  gap: 2px;
  background: rgba(11,14,19,0.5);
  border: 1px solid rgba(45,61,74,0.4);
  border-radius: 6px;
  padding: 2px;
}
.bb__mode-btn {
  font-size: 11px;
  padding: 2px 10px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  background: transparent;
  color: #6b7280;
  transition: color 0.1s, background 0.1s;
  white-space: nowrap;
}
.bb__mode-btn:hover { color: #9ca3af; }
.bb__mode-btn--active { background: rgba(87,241,219,0.1); color: #57f1db; }
</style>
