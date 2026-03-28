<!-- SignalCanvasLang/packages/diagram/src/PinTag.vue -->
<script setup lang="ts">
defineProps<{
  tags: Array<{ label: string; edgeId: string }>
  side: 'in' | 'out'
  borderColor: string
  highlightedEdgeId?: string | null
}>()
</script>

<template>
  <div
    v-if="tags.length"
    :class="['cdn-pin-tags', side === 'in' ? 'cdn-pin-tags-in' : 'cdn-pin-tags-out']"
  >
    <span
      v-for="tag in tags"
      :key="tag.edgeId"
      :class="[
        'cdn-pin-tag',
        side === 'in' ? 'cdn-pin-tag-in' : 'cdn-pin-tag-out',
        { 'tag-highlighted': highlightedEdgeId === tag.edgeId },
      ]"
      :style="{ borderColor }"
    >{{ tag.label }}</span>
  </div>
</template>

<style scoped>
.cdn-pin-tags {
  position: absolute; top: 0; bottom: 0;
  display: flex; flex-direction: column; justify-content: center;
  gap: 3px; pointer-events: none; z-index: 20;
}
.cdn-pin-tags-in  { right: 100%; align-items: flex-end;   padding-right: 8px; }
.cdn-pin-tags-out { left: 100%;  align-items: flex-start;  padding-left: 8px; }
.cdn-pin-tag {
  font-size: 9px; font-weight: 600; padding: 2px 7px;
  white-space: nowrap; background: #0f172a; line-height: 1.3;
}
.cdn-pin-tag-in {
  color: #7dd3fc; border-top: 1px solid; border-bottom: 1px solid;
  border-left: 1px solid; border-right: none; border-radius: 4px 0 0 4px;
}
.cdn-pin-tag-out {
  color: #5eead4; border-top: 1px solid; border-bottom: 1px solid;
  border-right: 1px solid; border-left: none; border-radius: 0 4px 4px 0;
}
.cdn-pin-tag.tag-highlighted {
  box-shadow: 0 0 0 2px rgba(255,255,255,0.8), 0 0 8px 3px rgba(255,255,255,0.5);
  border-color: #ffffff !important; color: #ffffff;
}
</style>
