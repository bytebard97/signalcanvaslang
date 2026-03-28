<!-- packages/demo/src/components/CodeEditor.vue -->
<script setup lang="ts">
const props = defineProps<{ modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <div class="ce">
    <textarea
      class="ce__ta"
      :value="props.modelValue"
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
  </div>
</template>

<style scoped>
.ce { display: flex; flex-direction: column; height: 100%; }
.ce__ta {
  flex: 1;
  width: 100%;
  background: #0B0E13;
  color: #e5e7eb;
  border: none;
  outline: none;
  resize: none;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
  font-size: 12px;
  line-height: 1.6;
  tab-size: 2;
}
</style>
