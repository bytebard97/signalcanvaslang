<!-- packages/diagram/src/NetTagNode.vue -->
<script setup lang="ts">
defineProps<{ data: { label: string; side: 'in' | 'out' } }>()
</script>

<template>
  <div :class="['nt', `nt--${data.side}`]">{{ data.label }}</div>
</template>

<style scoped>
.nt {
  font-size: 9px;
  font-weight: 600;
  padding: 2px 8px;
  white-space: nowrap;
  background: #0f172a;
  line-height: 1.4;
  pointer-events: none;
}
.nt--in {
  color: #7dd3fc;
  border-top: 1px solid #57f1db;
  border-bottom: 1px solid #57f1db;
  border-left: 1px solid #57f1db;
  border-right: none;
  border-radius: 4px 0 0 4px;
  transform: translateX(-100%);
}
.nt--out {
  color: #5eead4;
  border-top: 1px solid #57f1db;
  border-bottom: 1px solid #57f1db;
  border-right: 1px solid #57f1db;
  border-left: none;
  border-radius: 0 4px 4px 0;
}
</style>
