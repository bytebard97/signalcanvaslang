<script setup lang="ts">
import { computed } from 'vue'
import { BaseEdge, type EdgeProps } from '@vue-flow/core'

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEFAULT_STROKE_COLOR = '#57f1db'
const DEFAULT_STROKE_WIDTH = 1.5
const DEFAULT_STROKE_OPACITY = 0.85
const BRIDGE_DASH_ARRAY = '6 4'
const ARROWHEAD_SIZE = 8
const ARROWHEAD_HALF = ARROWHEAD_SIZE / 2

// ---------------------------------------------------------------------------
// Props (Vue Flow edge props)
// ---------------------------------------------------------------------------

const props = defineProps<EdgeProps>()

// ---------------------------------------------------------------------------
// Computed path from waypoints
// ---------------------------------------------------------------------------

interface Waypoint {
  x: number
  y: number
}

const waypoints = computed<Waypoint[]>(() => {
  return (props.data?.waypoints as Waypoint[] | undefined) ?? []
})

const edgeColor = computed<string>(() => {
  return (props.data?.color as string | undefined) ?? DEFAULT_STROKE_COLOR
})

const isBridge = computed<boolean>(() => {
  return props.data?.kind === 'bridge'
})

/** Build an SVG path string from orthogonal waypoints. */
const pathD = computed<string>(() => {
  const pts = waypoints.value
  if (pts.length === 0) return ''
  const segments = [`M ${pts[0].x} ${pts[0].y}`]
  for (let i = 1; i < pts.length; i++) {
    segments.push(`L ${pts[i].x} ${pts[i].y}`)
  }
  return segments.join(' ')
})

/** Arrowhead polygon points at the last segment's endpoint. */
const arrowPoints = computed<string>(() => {
  const pts = waypoints.value
  if (pts.length < 2) return ''

  const end = pts[pts.length - 1]
  const prev = pts[pts.length - 2]

  // Direction from prev to end
  const dx = end.x - prev.x
  const dy = end.y - prev.y
  const len = Math.sqrt(dx * dx + dy * dy)
  if (len === 0) return ''

  // Unit vector along the edge direction
  const ux = dx / len
  const uy = dy / len

  // Perpendicular
  const px = -uy
  const py = ux

  // Arrow tip at end, base offset back by ARROWHEAD_SIZE
  const tipX = end.x
  const tipY = end.y
  const baseX = end.x - ux * ARROWHEAD_SIZE
  const baseY = end.y - uy * ARROWHEAD_SIZE

  const p1x = baseX + px * ARROWHEAD_HALF
  const p1y = baseY + py * ARROWHEAD_HALF
  const p2x = baseX - px * ARROWHEAD_HALF
  const p2y = baseY - py * ARROWHEAD_HALF

  return `${p1x},${p1y} ${tipX},${tipY} ${p2x},${p2y}`
})

const isHidden = computed<boolean>(() => props.data?.hidden === true)

const strokeStyle = computed(() => ({
  stroke: edgeColor.value,
  strokeWidth: DEFAULT_STROKE_WIDTH,
  opacity: DEFAULT_STROKE_OPACITY,
  ...(isBridge.value ? { strokeDasharray: BRIDGE_DASH_ARRAY } : {}),
}))
</script>

<template>
  <g v-if="!isHidden">
    <!-- Edge path -->
    <path
      :d="pathD"
      :style="strokeStyle"
      fill="none"
      class="vue-flow__edge-path"
    />
    <!-- Arrowhead -->
    <polygon
      v-if="arrowPoints"
      :points="arrowPoints"
      :fill="edgeColor"
      :opacity="DEFAULT_STROKE_OPACITY"
    />
  </g>
</template>
