<script setup lang="ts">
import { computed } from 'vue'

export interface TimeDistributionItem {
    id: string
    label: string
    period: string
    icon: string
    seconds: number
    percentage: number
    sessionCount: number
}

const props = defineProps<{
    distribution: TimeDistributionItem[]
    formatDuration: (s: number) => string
}>()

const totalSeconds = computed(() =>
    props.distribution.reduce((acc, item) => acc + item.seconds, 0),
)

const dominantTime = computed(() => {
    if (totalSeconds.value === 0) return null
    let max = props.distribution[0]
    for (const item of props.distribution) {
        if (item.seconds > max.seconds) max = item
    }
    return max.seconds > 0 ? max : null
})

const habitPersona = computed(() => {
    if (!dominantTime.value) return null
    switch (dominantTime.value.id) {
        case 'morning':
            return { title: 'Early Bird', desc: 'Most of your reading takes place in the morning hours.' }
        case 'afternoon':
            return { title: 'Daylight Reader', desc: 'You prefer reading during the afternoon hours.' }
        case 'evening':
            return { title: 'Evening Unwinder', desc: 'Reading is your favorite evening ritual.' }
        case 'night':
            return { title: 'Night Owl', desc: 'Late night hours are when you get the most reading done.' }
        default:
            return null
    }
})

// Clean time range formatting to prevent overflow
function cleanPeriod(id: string): string {
    switch (id) {
        case 'morning': return '5 AM - 12 PM'
        case 'afternoon': return '12 PM - 5 PM'
        case 'evening': return '5 PM - 10 PM'
        case 'night': return '10 PM - 5 AM'
        default: return ''
    }
}
</script>

<template>
    <div class="w-full flex flex-col justify-between">
        <!-- Header -->
        <div class="flex flex-wrap items-center justify-between gap-2 mb-4">
            <div>
                <span class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) select-none block">
                    Habits & Rhythm
                </span>
                <h3 class="text-lg sm:text-xl font-serif font-bold text-(--text-primary) mt-0.5">
                    Time of Day
                </h3>
            </div>

            <span
                v-if="habitPersona"
                class="px-3 py-1 rounded-full bg-(--accent-color-light) border border-(--border-color) dark:border-white/20 text-xs font-sans font-semibold text-(--accent-color)"
            >
                {{ habitPersona.title }}
            </span>
        </div>

        <!-- 4-Part Segmented Bar -->
        <div
            v-if="totalSeconds > 0"
            class="h-3 w-full rounded-full bg-(--text-primary)/10 dark:bg-white/10 overflow-hidden flex mb-5 border border-(--border-color) dark:border-white/20 p-0.5"
        >
            <div
                v-for="item in distribution"
                :key="item.id"
                class="h-full transition-all duration-500 first:rounded-l-full last:rounded-r-full"
                :class="[
                    item.id === 'morning' ? 'bg-amber-400' :
                    item.id === 'afternoon' ? 'bg-orange-500' :
                    item.id === 'evening' ? 'bg-(--accent-color)' :
                    'bg-indigo-500 dark:bg-indigo-400'
                ]"
                :style="{ width: `${item.percentage}%` }"
                :title="`${item.label}: ${item.percentage}% (${formatDuration(item.seconds)})`"
            ></div>
        </div>

        <!-- 4 Spacious Columns Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            <div
                v-for="item in distribution"
                :key="item.id"
                class="p-3.5 rounded-xl bg-(--text-primary)/4 dark:bg-white/5 border border-(--border-color)/60 dark:border-white/10 flex flex-col justify-between transition-all"
                :class="dominantTime && dominantTime.id === item.id ? 'ring-1 ring-(--accent-color) dark:ring-(--accent-color)' : ''"
            >
                <div class="flex items-center gap-1.5 mb-1.5">
                    <span
                        class="material-symbols-outlined text-lg shrink-0"
                        :class="[
                            item.id === 'morning' ? 'text-amber-500 dark:text-amber-400' :
                            item.id === 'afternoon' ? 'text-orange-500 dark:text-orange-400' :
                            item.id === 'evening' ? 'text-(--accent-color)' :
                            'text-indigo-500 dark:text-indigo-400'
                        ]"
                    >
                        {{ item.icon }}
                    </span>
                    <span class="text-sm font-bold font-serif text-(--text-primary) truncate">
                        {{ item.label }}
                    </span>
                </div>

                <p class="text-[11px] font-sans text-(--text-secondary) dark:text-(--text-secondary) mb-2">
                    {{ cleanPeriod(item.id) }}
                </p>

                <div class="pt-2 border-t border-(--border-color)/50 dark:border-white/10 flex items-baseline justify-between gap-2">
                    <span class="text-sm sm:text-base font-bold font-sans tabular-nums text-(--text-primary)">
                        {{ formatDuration(item.seconds) }}
                    </span>
                    <span class="text-[10px] font-sans text-(--text-tertiary)">
                        {{ item.sessionCount }} {{ item.sessionCount === 1 ? 'session' : 'sessions' }}
                    </span>
                </div>
            </div>
        </div>
    </div>
</template>
