<script setup lang="ts">
import { ref, computed } from 'vue'

export interface WeeklyBar {
    date: string;
    label: string;
    totalSeconds: number;
    height: number;
    active: boolean;
}

const props = withDefaults(
    defineProps<{
        bars: WeeklyBar[];
        todayISO: string;
        formatDuration?: (s: number) => string;
    }>(),
    {
        formatDuration: (s: number) => {
            if (s === 0) return '0 min'
            const h = Math.floor(s / 3600)
            const m = Math.floor((s % 3600) / 60)
            if (h > 0 && m > 0) return `${h}h ${m}m`
            if (h > 0) return `${h}h`
            return `${m} min`
        },
    },
)

const hoveredBar = ref<WeeklyBar | null>(null)

const totalWeekSeconds = computed(() =>
    props.bars.reduce((acc, b) => acc + b.totalSeconds, 0),
)

const activeDaysCount = computed(() =>
    props.bars.filter((b) => b.totalSeconds > 0).length,
)

const peakBar = computed(() => {
    if (props.bars.length === 0) return null
    let max = props.bars[0]
    for (const b of props.bars) {
        if (b.totalSeconds > max.totalSeconds) max = b
    }
    return max.totalSeconds > 0 ? max : null
})

const maxSeconds = computed(() =>
    Math.max(...props.bars.map((b) => b.totalSeconds), 1),
)
</script>

<template>
    <div class="w-full flex flex-col justify-between">
        <!-- Header -->
        <div class="flex flex-wrap items-center justify-between gap-2 mb-4">
            <div>
                <span class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) select-none block">
                    Weekly Rhythm
                </span>
                <div class="flex items-baseline gap-2 mt-0.5">
                    <span class="text-xl sm:text-2xl font-bold font-serif text-(--text-primary) tabular-nums">
                        This week
                    </span>
                    <span class="text-sm font-sans font-semibold text-(--accent-color) tabular-nums">
                        ({{ formatDuration(totalWeekSeconds) }})
                    </span>
                    <span class="text-xs font-sans text-(--text-secondary)">
                        {{ activeDaysCount }} of 7 days
                    </span>
                </div>
            </div>

            <div
                v-if="peakBar"
                class="flex items-center gap-1.5 px-3 py-1 rounded-full bg-(--accent-color-light) border border-(--border-color) dark:border-white/20 text-xs font-sans font-medium text-(--accent-color)"
            >
                <span class="material-symbols-outlined text-sm">trending_up</span>
                <span>Peak: {{ peakBar.label }} ({{ formatDuration(peakBar.totalSeconds) }})</span>
            </div>
        </div>

        <!-- Equalizer Bars Area -->
        <div class="relative pt-4 pb-2">
            <!-- Active hover tooltip bubble -->
            <div
                v-if="hoveredBar"
                class="absolute -top-1 left-1/2 -translate-x-1/2 px-3 py-1 rounded-xl bg-(--bg-card) border border-(--border-color) dark:border-white/20 shadow-md text-xs font-sans font-medium text-(--text-primary) whitespace-nowrap pointer-events-none transition-all z-10"
            >
                <span class="font-semibold mr-1.5">{{ hoveredBar.label }}:</span>
                <span :class="hoveredBar.totalSeconds > 0 ? 'text-(--accent-color) font-bold' : 'text-(--text-tertiary)'">
                    {{ formatDuration(hoveredBar.totalSeconds) }}
                </span>
            </div>

            <!-- Equalizer Track Container -->
            <div
                class="flex items-end justify-between gap-2 sm:gap-4"
                style="height: 110px;"
            >
                <div
                    v-for="bar in bars"
                    :key="bar.date"
                    class="flex-1 flex flex-col items-center justify-end h-full gap-2 cursor-pointer group"
                    @mouseenter="hoveredBar = bar"
                    @mouseleave="hoveredBar = null"
                >
                    <!-- Column Slot with distinct background track in OLED -->
                    <div class="w-full max-w-[32px] h-full rounded-full bg-(--text-primary)/5 dark:bg-white/5 p-1 flex flex-col justify-end items-center relative border border-(--border-color) dark:border-white/15 group-hover:border-(--accent-color) transition-all">
                        <!-- Bar fill capsule -->
                        <div
                            class="w-full rounded-full transition-all duration-300 min-h-[8px]"
                            :class="[
                                bar.active
                                    ? 'bg-(--accent-color)'
                                    : 'bg-(--text-primary)/15 dark:bg-white/15',
                            ]"
                            :style="{
                                height: bar.active
                                    ? `${Math.max(10, Math.round((bar.totalSeconds / maxSeconds) * 86))}px`
                                    : '8px'
                            }"
                        ></div>
                    </div>

                    <!-- Day Label & Today Indicator -->
                    <div class="flex flex-col items-center gap-0.5">
                        <span
                            class="text-xs font-sans select-none tabular-nums transition-colors"
                            :class="[
                                bar.date === todayISO
                                    ? 'text-(--accent-color) font-bold'
                                    : 'text-(--text-secondary) group-hover:text-(--text-primary)'
                            ]"
                        >
                            {{ bar.label }}
                        </span>
                        <span
                            v-if="bar.date === todayISO"
                            class="w-1.5 h-1.5 rounded-full bg-(--accent-color)"
                        ></span>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
