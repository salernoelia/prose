<script setup lang="ts">
/**
 * ReadingChart - an editorial SVG area and line chart for reading activity.
 *
 * Uses dynamic resize-observed coordinates so circles are never warped/squished into ovals,
 * curves have perfect proportional bézier smoothing, and the graph has generous vertical headroom.
 */
import { computed, ref, onMounted, onUnmounted } from 'vue'
import type { ChartPoint } from '../../composables/useReadingStats'

export type Timeframe = '7d' | '30d' | '90d' | 'all'

const props = withDefaults(
  defineProps<{
    data: ChartPoint[]
    formatDuration: (s: number) => string
    timeframe?: Timeframe
    showControls?: boolean
  }>(),
  {
    timeframe: 'all',
    showControls: false,
  },
)

const emit = defineEmits<{
  (e: 'update:timeframe', value: Timeframe): void
}>()

// ── Dynamic SVG Dimensions ──────────────────────────────────────────────────
const svgRef = ref<SVGSVGElement | null>(null)
const svgWidth = ref(600)
const VH = 180        // Generous vertical headroom (no squishing)
const PAD_LEFT = 14
const PAD_RIGHT = 14
const PAD_TOP = 20
const PAD_BOTTOM = 30 // room for x-axis labels

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (svgRef.value) {
    const updateSize = () => {
      if (svgRef.value) {
        const w = svgRef.value.clientWidth
        if (w > 0) {
          svgWidth.value = w
        }
      }
    }
    updateSize()
    resizeObserver = new ResizeObserver(updateSize)
    resizeObserver.observe(svgRef.value)
  }
})

onUnmounted(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
})

const VW = computed(() => svgWidth.value)
const chartW = computed(() => VW.value - PAD_LEFT - PAD_RIGHT)
const chartH = computed(() => VH - PAD_TOP - PAD_BOTTOM)

// ── Derived geometry ────────────────────────────────────────────────────────
const maxSeconds = computed(() =>
  Math.max(...props.data.map((d) => d.seconds), 1),
)

const totalWindowSeconds = computed(() =>
  props.data.reduce((acc, d) => acc + d.seconds, 0),
)

const activeDaysInWindow = computed(
  () => props.data.filter((d) => d.seconds > 0).length,
)

const peakPoint = computed(() => {
  if (props.data.length === 0) return null
  let max = props.data[0]
  for (const d of props.data) {
    if (d.seconds > max.seconds) max = d
  }
  return max.seconds > 0 ? max : null
})

interface PlotPoint {
  x: number
  y: number
  point: ChartPoint
}

const plotPoints = computed<PlotPoint[]>(() => {
  const n = props.data.length
  if (n === 0) return []
  const w = chartW.value
  const h = chartH.value

  return props.data.map((p, i) => {
    const x = PAD_LEFT + (n === 1 ? w / 2 : (i / (n - 1)) * w)
    const y = PAD_TOP + h - (p.seconds / maxSeconds.value) * h
    return { x, y, point: p }
  })
})

function buildLinePath(pts: PlotPoint[]): string {
  if (pts.length === 0) return ''
  if (pts.length === 1) return `M ${pts[0].x} ${pts[0].y}`

  const parts: string[] = [`M ${pts[0].x} ${pts[0].y}`]
  for (let i = 1; i < pts.length; i++) {
    const prev = pts[i - 1]
    const curr = pts[i]
    const dx = (curr.x - prev.x) / 2.5
    parts.push(`C ${prev.x + dx} ${prev.y}, ${curr.x - dx} ${curr.y}, ${curr.x} ${curr.y}`)
  }
  return parts.join(' ')
}

function buildAreaPath(pts: PlotPoint[]): string {
  if (pts.length === 0) return ''
  const baseline = PAD_TOP + chartH.value
  const line = buildLinePath(pts)
  const last = pts[pts.length - 1]
  const first = pts[0]
  return `${line} L ${last.x} ${baseline} L ${first.x} ${baseline} Z`
}

const linePath = computed(() => buildLinePath(plotPoints.value))
const areaPath = computed(() => buildAreaPath(plotPoints.value))

// ── X-axis labels ────────────────────────────────────────────────────────────
const MONTHS = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']

interface XLabel {
  x: number
  text: string
}

const xLabels = computed<XLabel[]>(() => {
  const pts = plotPoints.value
  if (pts.length === 0) return []

  const labels: XLabel[] = []
  const n = pts.length

  if (n <= 10) {
    // Show day labels
    for (let i = 0; i < n; i++) {
      const [, mStr, dStr] = pts[i].point.date.split('-')
      labels.push({
        x: pts[i].x,
        text: `${MONTHS[parseInt(mStr) - 1]} ${parseInt(dStr)}`,
      })
    }
    return labels
  }

  let lastMonth = -1
  for (let i = 0; i < n; i++) {
    const [, mStr] = pts[i].point.date.split('-')
    const month = parseInt(mStr) - 1
    const isStart = i === 0
    const isEnd = i === n - 1
    const monthChanged = month !== lastMonth

    if (isStart || isEnd || monthChanged) {
      const prev = labels[labels.length - 1]
      const tooClose = prev && pts[i].x - prev.x < 42
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

function onMouseMove(e: MouseEvent) {
  const svg = svgRef.value
  if (!svg) return
  const rect = svg.getBoundingClientRect()
  const clientX = e.clientX - rect.left

  const pts = plotPoints.value
  if (pts.length === 0) { tooltip.value = null; return }

  let closest = pts[0]
  let minDist = Math.abs(pts[0].x - clientX)
  for (const p of pts) {
    const d = Math.abs(p.x - clientX)
    if (d < minDist) { minDist = d; closest = p }
  }

  tooltip.value = { x: closest.x, y: closest.y, point: closest.point }
}

function onMouseLeave() {
  tooltip.value = null
}

function formatDate(iso: string): string {
  const [y, m, d] = iso.split('-').map(Number)
  const date = new Date(y, m - 1, d)
  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
}

function tooltipX(x: number): number {
  return Math.min(Math.max(x, 48), VW.value - 48)
}
</script>

<template>
  <div class="relative w-full select-none">
    <!-- Optional Controls / Timeframe Filter Header -->
    <div
      v-if="showControls"
      class="flex flex-wrap items-center justify-between gap-3 mb-5"
    >
      <div class="flex items-baseline gap-2">
        <span class="text-xl sm:text-2xl font-bold font-sans text-(--text-primary) tabular-nums">
          {{ formatDuration(totalWindowSeconds) }}
        </span>
        <span class="text-xs font-sans text-(--text-secondary)">
          {{ activeDaysInWindow }} active {{ activeDaysInWindow === 1 ? 'day' : 'days' }}
        </span>
      </div>

      <!-- Timeframe Pills with OLED borders -->
      <div class="flex items-center gap-1 bg-(--text-primary)/5 dark:bg-white/10 p-1 rounded-xl border border-(--border-color)/60 dark:border-white/20">
        <button
          v-for="tf in (['7d', '30d', '90d', 'all'] as Timeframe[])"
          :key="tf"
          @click="emit('update:timeframe', tf)"
          class="px-2.5 py-1 text-xs font-sans font-medium rounded-lg transition-all capitalize cursor-pointer"
          :class="timeframe === tf
            ? 'bg-(--bg-card) dark:bg-zinc-800 text-(--text-primary) shadow-xs font-bold border border-(--border-color)/80 dark:border-white/30'
            : 'text-(--text-secondary) hover:text-(--text-primary)'"
        >
          {{ tf === 'all' ? 'All' : tf.toUpperCase() }}
        </button>
      </div>
    </div>

    <!-- Responsive SVG (Never squished) -->
    <div class="w-full relative">
      <svg
        ref="svgRef"
        :viewBox="`0 0 ${VW} ${VH}`"
        class="w-full overflow-visible block"
        :style="{ height: `${VH}px` }"
        @mousemove="onMouseMove"
        @mouseleave="onMouseLeave"
      >
        <defs>
          <linearGradient id="reading-area-grad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--accent-color)" stop-opacity="0.35" />
            <stop offset="100%" stop-color="var(--accent-color)" stop-opacity="0.02" />
          </linearGradient>
        </defs>

        <!-- Background Gridlines with High OLED Visibility -->
        <line
          :x1="PAD_LEFT" :y1="PAD_TOP"
          :x2="VW - PAD_RIGHT" :y2="PAD_TOP"
          stroke="var(--text-primary)" stroke-opacity="0.18" stroke-width="1" stroke-dasharray="4 4"
        />
        <line
          :x1="PAD_LEFT" :y1="PAD_TOP + chartH / 2"
          :x2="VW - PAD_RIGHT" :y2="PAD_TOP + chartH / 2"
          stroke="var(--text-primary)" stroke-opacity="0.18" stroke-width="1" stroke-dasharray="4 4"
        />
        <line
          :x1="PAD_LEFT" :y1="PAD_TOP + chartH"
          :x2="VW - PAD_RIGHT" :y2="PAD_TOP + chartH"
          stroke="var(--text-primary)" stroke-opacity="0.35" stroke-width="1.2"
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
            stroke="var(--accent-color)"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />

          <!-- Points for smaller series (rendered as perfect round circles) -->
          <template v-if="plotPoints.length <= 30">
            <circle
              v-for="(pt, i) in plotPoints"
              :key="i"
              :cx="pt.x"
              :cy="pt.y"
              :r="pt.point.isToday ? 4 : 2.5"
              :fill="pt.point.seconds > 0 ? 'var(--accent-color)' : 'var(--text-primary)'"
              :fill-opacity="pt.point.seconds > 0 ? '1' : '0.3'"
              :stroke="'var(--bg-card)'"
              stroke-width="1.5"
            />
          </template>

          <!-- Peak Point Indicator Badge / Ring -->
          <template v-if="peakPoint && plotPoints.length > 1">
            <template v-for="pt in plotPoints" :key="'peak-' + pt.point.date">
              <circle
                v-if="pt.point.date === peakPoint.date"
                :cx="pt.x"
                :cy="pt.y"
                r="5"
                fill="var(--accent-color)"
                stroke="var(--bg-card)"
                stroke-width="2"
              />
            </template>
          </template>

          <!-- Today indicator line -->
          <template v-for="pt in plotPoints" :key="'today-' + pt.point.date">
            <line
              v-if="pt.point.isToday"
              :x1="pt.x" :y1="PAD_TOP"
              :x2="pt.x" :y2="PAD_TOP + chartH"
              stroke="var(--accent-color)"
              stroke-width="1.5"
              stroke-dasharray="3 3"
              opacity="0.8"
            />
          </template>

          <!-- Hover crosshair -->
          <template v-if="tooltip">
            <line
              :x1="tooltip.x" :y1="PAD_TOP"
              :x2="tooltip.x" :y2="PAD_TOP + chartH"
              stroke="var(--text-secondary)"
              stroke-width="1"
              stroke-dasharray="2 2"
            />
            <circle
              :cx="tooltip.x"
              :cy="tooltip.y"
              r="5"
              fill="var(--accent-color)"
              stroke="var(--bg-card)"
              stroke-width="2"
            />
          </template>

          <!-- X-axis labels -->
          <text
            v-for="(lbl, i) in xLabels"
            :key="i"
            :x="lbl.x"
            :y="VH - 6"
            font-size="11"
            text-anchor="middle"
            fill="var(--text-secondary)"
            font-family="inherit"
            font-weight="500"
          >{{ lbl.text }}</text>
        </template>
      </svg>

      <!-- Tooltip bubble -->
      <Transition name="tip">
        <div
          v-if="tooltip && tooltip.point.seconds > 0"
          class="absolute top-1 pointer-events-none z-10 bg-(--bg-card) dark:bg-zinc-900 border border-(--border-color) dark:border-white/30 rounded-xl px-3.5 py-1.5 shadow-lg text-xs font-sans font-medium text-(--text-primary) whitespace-nowrap -translate-x-1/2"
          :style="{ left: `${(tooltipX(tooltip.x) / VW) * 100}%` }"
        >
          <span class="text-(--text-secondary) mr-1.5">{{ formatDate(tooltip.point.date) }}:</span>
          <span class="font-bold text-(--accent-color)">{{ formatDuration(tooltip.point.seconds) }}</span>
        </div>
        <div
          v-else-if="tooltip && tooltip.point.seconds === 0"
          class="absolute top-1 pointer-events-none z-10 bg-(--bg-card) dark:bg-zinc-900 border border-(--border-color) dark:border-white/30 rounded-xl px-3.5 py-1.5 shadow-lg text-xs font-sans text-(--text-tertiary) whitespace-nowrap -translate-x-1/2"
          :style="{ left: `${(tooltipX(tooltip.x) / VW) * 100}%` }"
        >
          {{ formatDate(tooltip.point.date) }}: no reading
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.tip-enter-active,
.tip-leave-active {
  transition: opacity 0.12s ease, transform 0.12s ease;
}
.tip-enter-from,
.tip-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-3px);
}
.tip-enter-to,
.tip-leave-from {
  transform: translateX(-50%) translateY(0);
}
</style>
