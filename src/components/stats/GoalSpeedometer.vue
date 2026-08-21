<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
    defineProps<{
        currentSeconds: number
        targetSeconds?: number
        label?: string
        subtitle?: string
        formatDuration: (s: number) => string
    }>(),
    {
        targetSeconds: 1800, // default 30 mins
        label: 'Daily Pace',
    },
)

const progressRatio = computed(() => {
    if (props.targetSeconds <= 0) return 0
    return Math.min(props.currentSeconds / props.targetSeconds, 1)
})

const percentage = computed(() => {
    if (props.targetSeconds <= 0) return 0
    return Math.round((props.currentSeconds / props.targetSeconds) * 100)
})

// Generate radial ticks around a semi-circle arch (180 degrees from -180 to 0 or 180 to 360)
// Center at (100, 95), radius 75
const TICK_COUNT = 41
const ticks = computed(() => {
    const list = []
    const startAngle = Math.PI // 180 deg (left)
    const endAngle = 2 * Math.PI // 360 deg (right)
    const activeIndex = Math.round(progressRatio.value * (TICK_COUNT - 1))

    for (let i = 0; i < TICK_COUNT; i++) {
        const t = i / (TICK_COUNT - 1)
        const angle = startAngle + t * (endAngle - startAngle)
        const rInner = 62
        const rOuter = i % 5 === 0 ? 80 : 73
        const x1 = 100 + rInner * Math.cos(angle)
        const y1 = 92 + rInner * Math.sin(angle)
        const x2 = 100 + rOuter * Math.cos(angle)
        const y2 = 92 + rOuter * Math.sin(angle)

        list.push({
            id: i,
            x1,
            y1,
            x2,
            y2,
            isActive: i <= activeIndex && props.currentSeconds > 0,
            isMajor: i % 5 === 0,
        })
    }
    return list
})
</script>

<template>
    <div class="w-full flex flex-col justify-between items-center text-center relative">
        <!-- Top Label & Badge -->
        <div class="w-full flex items-center justify-between gap-2 mb-4 select-none">
            <div class="text-left">
                <span class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) block">
                    Daily Goal
                </span>
                <h3 class="text-lg sm:text-xl font-serif font-bold text-(--text-primary) mt-0.5">
                    {{ label }}
                </h3>
            </div>
            <span
                class="text-xs font-sans font-semibold px-2.5 py-0.5 rounded-full uppercase tracking-wider border bg-(--accent-color-light) text-(--accent-color) border-(--border-color) dark:border-white/20"
            >
                {{ percentage }}%
            </span>
        </div>

        <!-- Fan Arch Speedometer Gauge -->
        <div class="relative w-full max-w-[240px] aspect-[200/115] my-2 flex items-end justify-center select-none">
            <svg
                viewBox="0 0 200 105"
                class="w-full h-full overflow-visible"
            >
                <!-- Arch ticks with high OLED contrast -->
                <g stroke-linecap="round">
                    <line
                        v-for="tick in ticks"
                        :key="tick.id"
                        :x1="tick.x1"
                        :y1="tick.y1"
                        :x2="tick.x2"
                        :y2="tick.y2"
                        :stroke-width="tick.isMajor ? 2.5 : 1.5"
                        :stroke="tick.isActive ? 'var(--accent-color)' : 'var(--text-primary)'"
                        :stroke-opacity="tick.isActive ? 1 : 0.18"
                        class="transition-all duration-300"
                    />
                </g>
            </svg>

            <!-- Center Metric -->
            <div class="absolute bottom-1 left-1/2 -translate-x-1/2 flex flex-col items-center pointer-events-none">
                <span class="text-2xl sm:text-3xl font-bold font-sans text-(--text-primary) tabular-nums leading-none tracking-tight">
                    {{ formatDuration(currentSeconds) }}
                </span>
                <span class="text-[10px] font-sans text-(--text-secondary) dark:text-(--text-secondary) mt-1 uppercase tracking-wider">
                    of {{ formatDuration(targetSeconds) }} goal
                </span>
            </div>
        </div>

        <!-- Subtitle Footer -->
        <p class="text-xs font-sans text-(--text-secondary) mt-2 select-none">
            {{ subtitle || (percentage >= 100 ? 'Daily reading goal reached!' : `${formatDuration(Math.max(0, targetSeconds - currentSeconds))} remaining today`) }}
        </p>
    </div>
</template>
