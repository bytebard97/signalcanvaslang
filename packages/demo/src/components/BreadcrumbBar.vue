<!-- packages/demo/src/components/BreadcrumbBar.vue -->
<script setup lang="ts">
defineProps<{
  breadcrumbs: Array<{ label: string; index: number }>
}>()

const emit = defineEmits<{ navigate: [index: number] }>()
</script>

<template>
  <div class="bb">
    <template v-for="(crumb, i) in breadcrumbs" :key="crumb.index">
      <span
        class="bb__crumb"
        :class="i === breadcrumbs.length - 1 ? 'bb__crumb--active' : ''"
        @click="emit('navigate', crumb.index)"
      >{{ crumb.label }}</span>
      <span v-if="i < breadcrumbs.length - 1" class="bb__sep">›</span>
    </template>
  </div>
</template>

<style scoped>
.bb {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  background: #181C22;
  border-bottom: 1px solid rgba(45,61,74,0.3);
  flex-shrink: 0;
  min-height: 34px;
}
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
</style>
