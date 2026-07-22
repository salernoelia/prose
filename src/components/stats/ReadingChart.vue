<script setup lang="ts">
/**
 * ReadingChart - a smooth SVG area and line chart for reading activity.
 *
 * Pure SVG, no external library. Uses cubic-bezier interpolation with
 * horizontal control points so the curve is smooth but never overshoots zero.
 * Fully styled via CSS custom properties so it respects light, dark, and sepia.
 */
import { computed, ref } from 'vue'
import type { ChartPoint } from '../../composables/useReadingStats'

const props = defineProps<{
  data: ChartPoint[]
  formatDuration: (s: number) => string
}>()

// ── Layout constants ────────────────────────────────────────────────────────
const VW = 400      // viewBox width
const VH = 120      // viewBox height
const PAD_LEFT = 0
const PAD_RIGHT = 0
const PAD_TOP = 12
const PAD_BOTTOM = 28  // room for x-axis labels

const chartW = VW - PAD_LEFT - PAD_RIGHT
const chartH = VH - PAD_TOP - PAD_BOTTOM

// ── Derived geometry ────────────────────────────────────────────────────────
const maxSeconds = computed(() =>
  Math.max(...props.data.map((d) => d.seconds), 1),
)

interface PlotPoint {
  x: number
  y: number
  point: ChartPoint
}

const plotPoints = computed<PlotPoint[]>(() => {
  const n = props.data.length
  if (n === 0) return []
  return props.data.map((p, i) => {
    const x = PAD_LEFT + (n === 1 ? chartW / 2 : (i / (n - 1)) * chartW)
    // y=0 is top in SVG; invert so 0 seconds → bottom
    const y = PAD_TOP + chartH - (p.seconds / maxSeconds.value) * chartH
    return { x, y, point: p }
  })
})

// Smooth cubic bezier path: control points are horizontal so the curve
// never overshoots below zero on sparse data.
function buildLinePath(pts: PlotPoint[]): string {
  if (pts.length === 0) return ''
  if (pts.length === 1) return `M ${pts[0].x} ${pts[0].y}`

  const parts: string[] = [`M ${pts[0].x} ${pts[0].y}`]
  for (let i = 1; i < pts.length; i++) {
    const prev = pts[i - 1]
    const curr = pts[i]
    const dx = (curr.x - prev.x) / 2.5
    // cp1: leave previous point horizontally, cp2: arrive at current horizontally
    parts.push(`C ${prev.x + dx} ${prev.y}, ${curr.x - dx} ${curr.y}, ${curr.x} ${curr.y}`)
  }
  return parts.join(' ')
}

function buildAreaPath(pts: PlotPoint[]): string {
  if (pts.length === 0) return ''
  const baseline = PAD_TOP + chartH
  const line = buildLinePath(pts)
  const last = pts[pts.length - 1]
  const first = pts[0]
  return `${line} L ${last.x} ${baseline} L ${first.x} ${baseline} Z`
}

const linePath = computed(() => buildLinePath(plotPoints.value))
const areaPath = computed(() => buildAreaPath(plotPoints.value))

// ── X-axis labels ────────────────────────────────────────────────────────────
// Show a label at month transitions (when the month changes) or at start/end.
// For weekly data, the date is the week-start Monday.
const MONTHS = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']

interface XLabel {
  x: number
  text: string
}

const xLabels = computed<XLabel[]>(() => {
  const pts = plotPoints.value
  if (pts.length === 0) return []

  const labels: XLabel[] = []
  let lastMonth = -1

  for (let i = 0; i < pts.length; i++) {
    const [, mStr] = pts[i].point.date.split('-')
    const month = parseInt(mStr) - 1

    // Label at start, at every month transition, and ensure not too crowded
    const isStart = i === 0
    const monthChanged = month !== lastMonth

    if (isStart || monthChanged) {
      // Don't add if too close to the previous label
      const prev = labels[labels.length - 1]
      const tooClose = prev && pts[i].x - prev.x < 28

      if (!tooClose) {
        labels.push({ x: pts[i].x, text: MONTHS[month] })
      }
      lastMonth = month
    }
  }
  return labels
})

// ── Tooltip ──────────────────────────────────────────────────────────────────
const tooltip = ref<{ x: number; y: number; point: ChartPoint } | null>(null)
const svgRef = ref<SVGElement | null>(null)

function onMouseMove(e: MouseEvent) {
  const svg = svgRef.value
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  // Map client X to viewBox X
  const clientX = e.clientX - rect.left
  const vbX = (clientX / rect.width) * VW

  // Find closest plot point
  const pts = plotPoints.value
  if (pts.length === 0) { tooltip.value = null; return }

  let closest = pts[0]
  let minDist = Math.abs(pts[0].x - vbX)
  for (const p of pts) {
    const d = Math.abs(p.x - vbX)
    if (d < minDist) { minDist = d; closest = p }
  }

  tooltip.value = { x: closest.x, y: closest.y, point: closest.point }
}

function onMouseLeave() {
  tooltip.value = null
}

// Tooltip label: format date nicely
function formatDate(iso: string): string {
  const [y, m, d] = iso.split('-').map(Number)
  const date = new Date(y, m - 1, d)
  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

// Clamp tooltip bubble horizontally so it stays within the SVG
function tooltipX(x: number): number {
  return Math.min(Math.max(x, 32), VW - 32)
}
</script>

<template>
  <div class="relative w-full select-none">
    <svg
      ref="svgRef"
      :viewBox="`0 0 ${VW} ${VH}`"
      preserveAspectRatio="none"
      class="w-full overflow-visible"
      :style="{ height: '120px' }"
      @mousemove="onMouseMove"
      @mouseleave="onMouseLeave"
    >
      <defs>
        <!-- Area gradient: top = accent colour at opacity, bottom = transparent -->
        <linearGradient id="reading-area-grad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="var(--text-primary)" stop-opacity="0.15" />
          <stop offset="100%" stop-color="var(--text-primary)" stop-opacity="0.0" />
        </linearGradient>
      </defs>

      <!-- Empty state baseline -->
      <line
        v-if="data.length === 0"
        :x1="PAD_LEFT" :y1="PAD_TOP + chartH"
        :x2="VW - PAD_RIGHT" :y2="PAD_TOP + chartH"
        stroke="var(--border-color)" stroke-width="1"
      />

      <template v-if="plotPoints.length > 0">
        <!-- Filled area -->
        <path
          :d="areaPath"
          fill="url(#reading-area-grad)"
        />

        <!-- Line -->
        <path
          :d="linePath"
          fill="none"
          stroke="var(--text-primary)"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />

        <!-- Dots: only render for sparse data, else too cluttered -->
        <template v-if="plotPoints.length <= 30">
          <circle
            v-for="(pt, i) in plotPoints"
            :key="i"
            :cx="pt.x"
            :cy="pt.y"
            :r="pt.point.isToday ? 3 : 2"
            :fill="pt.point.seconds > 0 ? 'var(--text-primary)' : 'var(--border-color)'"
            :stroke="'var(--bg-card)'"
            stroke-width="1.5"
          />
        </template>

        <!-- Today indicator line -->
        <template v-for="pt in plotPoints" :key="pt.point.date">
          <line
            v-if="pt.point.isToday"
            :x1="pt.x" :y1="PAD_TOP"
            :x2="pt.x" :y2="PAD_TOP + chartH"
            stroke="var(--text-primary)"
            stroke-width="1"
            stroke-dasharray="3 3"
            opacity="0.3"
          />
        </template>

        <!-- Hover crosshair -->
        <template v-if="tooltip">
          <line
            :x1="tooltip.x" :y1="PAD_TOP"
            :x2="tooltip.x" :y2="PAD_TOP + chartH"
            stroke="var(--text-tertiary)"
            stroke-width="1"
            stroke-dasharray="2 2"
          />
          <circle
            :cx="tooltip.x"
            :cy="tooltip.y"
            r="4"
            fill="var(--text-primary)"
            stroke="var(--bg-card)"
            stroke-width="2"
          />
        </template>

        <!-- X-axis labels -->
        <text
          v-for="(lbl, i) in xLabels"
          :key="i"
          :x="lbl.x"
          :y="VH - 4"
          font-size="9"
          text-anchor="middle"
          fill="var(--text-tertiary)"
          font-family="inherit"
        >{{ lbl.text }}</text>
      </template>
    </svg>

    <!-- Tooltip bubble (HTML, sits above SVG) -->
    <Transition name="tip">
      <div
        v-if="tooltip && tooltip.point.seconds > 0"
        class="absolute top-1 pointer-events-none z-10 bg-(--bg-app) border border-(--border-color) rounded-lg px-2.5 py-1.5 shadow-md text-xs font-medium text-(--text-primary) whitespace-nowrap -translate-x-1/2"
        :style="{ left: `${(tooltipX(tooltip.x) / VW) * 100}%` }"
      >
        <span class="text-(--text-secondary) font-normal mr-1">{{ formatDate(tooltip.point.date) }}</span>
        {{ formatDuration(tooltip.point.seconds) }}
      </div>
      <div
        v-else-if="tooltip && tooltip.point.seconds === 0"
        class="absolute top-1 pointer-events-none z-10 bg-(--bg-app) border border-(--border-color) rounded-lg px-2.5 py-1.5 shadow-md text-xs text-(--text-tertiary) whitespace-nowrap -translate-x-1/2"
        :style="{ left: `${(tooltipX(tooltip.x) / VW) * 100}%` }"
      >
        {{ formatDate(tooltip.point.date) }}: no reading
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.tip-enter-active,
.tip-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}
.tip-enter-from,
.tip-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-2px);
}
.tip-enter-to,
.tip-leave-from {
  transform: translateX(-50%) translateY(0);
}
</style>
