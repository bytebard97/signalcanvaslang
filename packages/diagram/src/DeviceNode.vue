<!-- SignalCanvasLang/packages/diagram/src/DeviceNode.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position } from '@vue-flow/core'
import { Router, SlidersHorizontal, Box } from 'lucide-vue-next'
import type { DeviceNodeData } from './types'
import PinTag from './PinTag.vue'

const CATEGORY_ICON_SIZE = 16

const props = defineProps<{
  data: DeviceNodeData
  selected?: boolean
}>()

const categoryIcon = computed(() => {
  switch (props.data.category.toLowerCase()) {
    case 'stagebox': return Router
    case 'console':  return SlidersHorizontal
    default:         return Box
  }
})

function portLabel(port: { id: string; name: string; range?: string }): string {
  return port.range ? `${port.name}${port.range}` : port.name
}

function isPortConnected(portId: string): boolean {
  return props.data.connectedPortIds?.has(portId) ?? false
}

const hasMeta = computed(() =>
  props.data.manufacturer.length > 0 || props.data.model.length > 0,
)

const isNetnames = computed(() => props.data.mode === 'netnames')

function getPortTags(portId: string): Array<{ label: string; edgeId: string }> {
  return props.data.portTags?.[portId] ?? []
}
</script>

<template>
  <div :class="['dn', selected ? 'dn--selected' : '', data.drillable ? 'dn--drillable' : '']">
    <!-- Header -->
    <div class="dn__header">
      <div class="dn__header-left">
        <component :is="categoryIcon" :size="CATEGORY_ICON_SIZE" class="dn__icon" />
        <span class="dn__instance-name">{{ data.instanceName }}</span>
      </div>
      <span class="dn__template-name">{{ data.templateName }}</span>
    </div>

    <!-- Ports body -->
    <div class="dn__ports">
      <!-- Input ports (left column) -->
      <div class="dn__col">
        <div
          v-for="port in data.inputPorts"
          :key="port.id"
          class="dn__port-row"
        >
          <Handle
            :id="port.id"
            type="target"
            :position="Position.Left"
            :style="{
              background: 'transparent', border: 'none',
              width: '8px', height: '8px',
              left: '-21px', top: '50%', transform: 'translateY(-50%)',
            }"
          />
          <div
            class="dn__dot dn__dot--left"
            :class="isPortConnected(port.id) ? 'dn__dot--connected' : ''"
          />
          <div class="dn__pill">
            <span class="dn__port-label">{{ portLabel(port) }}</span>
          </div>
          <PinTag
            v-if="isNetnames && getPortTags(port.id).length > 0"
            :tags="getPortTags(port.id)"
            side="in"
            border-color="#57f1db"
          />
        </div>
      </div>

      <!-- Output ports (right column) -->
      <div class="dn__col dn__col--right">
        <div
          v-for="port in data.outputPorts"
          :key="port.id"
          class="dn__port-row dn__port-row--output"
        >
          <Handle
            :id="port.id"
            type="source"
            :position="Position.Right"
            :style="{
              background: 'transparent', border: 'none',
              width: '8px', height: '8px',
              right: '-21px', top: '50%', transform: 'translateY(-50%)',
            }"
          />
          <div class="dn__pill">
            <span class="dn__port-label">{{ portLabel(port) }}</span>
          </div>
          <div
            class="dn__dot dn__dot--right"
            :class="isPortConnected(port.id) ? 'dn__dot--connected dn__dot--connected-glow' : ''"
          />
          <PinTag
            v-if="isNetnames && getPortTags(port.id).length > 0"
            :tags="getPortTags(port.id)"
            side="out"
            border-color="#57f1db"
          />
        </div>
      </div>
    </div>

    <!-- Meta Inspector (hover reveal) -->
    <div v-if="hasMeta" class="dn__meta">
      <div class="dn__meta-title">Meta Inspector</div>
      <div class="dn__meta-grid">
        <span class="dn__meta-key">Manufacturer:</span>
        <span class="dn__meta-val">{{ data.manufacturer }}</span>
        <span class="dn__meta-key">Model:</span>
        <span class="dn__meta-val">{{ data.model }}</span>
        <span class="dn__meta-key">Category:</span>
        <span class="dn__meta-val">{{ data.category }}</span>
        <span class="dn__meta-key">Location:</span>
        <span class="dn__meta-val">{{ data.location }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* BEM: dn = device-node */

.dn {
  width: 260px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 12px;
  background: #1E2228;
  border: 1px solid rgba(45, 61, 74, 0.3);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  transform: scale(1);
  transition: transform 0.15s;
  position: relative;
  cursor: pointer;
}
.dn:hover { transform: scale(1.02); }
.dn--selected {
  border: 2px solid #57f1db;
  box-shadow: 0 0 20px rgba(87, 241, 219, 0.15);
}

.dn__header {
  background: #181C22;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #2D3D4A;
}
.dn__header-left { display: flex; align-items: center; gap: 8px; min-width: 0; overflow: hidden; }
.dn__icon { flex-shrink: 0; color: #57f1db; }
.dn__instance-name {
  font-family: monospace; font-size: 12px; font-weight: 700;
  color: #57f1db; letter-spacing: -0.025em;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.dn__template-name {
  font-size: 10px; color: #6b7280; font-family: monospace; flex-shrink: 0; margin-left: 8px;
}

.dn__ports {
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  gap: 8px;
  position: relative;
}
.dn__col { display: flex; flex-direction: column; gap: 8px; }
.dn__col--right { align-items: flex-end; }

.dn__port-row { position: relative; display: flex; align-items: center; }
.dn__port-row--output { flex-direction: row-reverse; }

.dn__dot {
  position: absolute;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #4b5563;
}
.dn__dot--left  { left: -21px; top: 50%; transform: translateY(-50%); }
.dn__dot--right { right: -21px; top: 50%; transform: translateY(-50%); }
.dn__dot--connected { background: rgba(45, 212, 191, 0.4); }
.dn__dot--connected-glow { background: #2DD4BF; box-shadow: 0 0 8px #57f1db; }

.dn__pill {
  background: rgba(11, 14, 19, 0.5);
  padding: 4px 6px;
  border-radius: 8px;
}
.dn__port-label {
  font-family: monospace; font-size: 9px; color: #9ca3af; white-space: nowrap;
}

/* Meta inspector — reveals on group:hover via parent .dn:hover */
.dn__meta {
  position: absolute;
  top: -16px; right: -16px;
  transform: translateX(100%);
  width: 224px;
  background: rgba(50, 53, 59, 0.95);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(60, 74, 70, 0.3);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  opacity: 0;
  transition: opacity 0.15s;
  pointer-events: none;
  z-index: 50;
}
.dn:hover .dn__meta { opacity: 1; }
.dn__meta-title {
  font-size: 10px; text-transform: uppercase; letter-spacing: 0.1em;
  color: #57f1db; font-weight: 700;
  border-bottom: 1px solid rgba(60, 74, 70, 0.1);
  padding-bottom: 8px; margin-bottom: 12px;
}
.dn__meta-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 0;
  font-size: 11px;
}
.dn__meta-key { color: #9ca3af; }
.dn__meta-val { color: white; font-family: monospace; }

.dn--drillable { cursor: zoom-in; }
.dn--drillable .dn__header { border-bottom-color: rgba(87, 241, 219, 0.25); }
</style>
