<script setup lang="ts">
import { computed } from 'vue'
import { useReadingStats } from '../composables/useReadingStats'
import { deleteSession } from '../stores/sessions'
import { ReadingChart, StatCard, WeeklyActivityChart } from '../components/stats'

const {
    totalBooks,
    booksFinished,
    booksInProgress,
    totalReadingSeconds,
    sessionHistory,
    currentStreak,
    bestStreak,
    weeklyActivity,
    bookActivity,
    allTimeDaily,
    formatDuration,
} = useReadingStats()

function onDeleteSession(id: string) {
    void deleteSession(id)
}

function formatSessionDate(ms: number): string {
    return new Date(ms).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
    })
}

const CHART_MAX_HEIGHT = 64

const maxWeeklySeconds = computed(() =>
    Math.max(...weeklyActivity.value.map((d) => d.totalSeconds), 1),
)

const weeklyBars = computed(() =>
    weeklyActivity.value.map((d) => ({
        ...d,
        height: Math.max(4, Math.round((d.totalSeconds / maxWeeklySeconds.value) * CHART_MAX_HEIGHT)),
        active: d.totalSeconds > 0,
    })),
)

const todayISO = (() => {
    const d = new Date()
    const y = d.getFullYear()
    const m = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    return `${y}-${m}-${day}`
})()

const todaySeconds = computed(
    () => weeklyActivity.value.find((d) => d.date === todayISO)?.totalSeconds ?? 0,
)

const readThisWeek = computed(() =>
    weeklyActivity.value.reduce((acc, d) => acc + d.totalSeconds, 0),
)

const topBook = computed(() => bookActivity.value[0] ?? null)
</script>

<template>
    <div class="w-full animate-fade-in font-serif pb-[calc(3rem+env(safe-area-inset-bottom,0px))]">
        <header class="pb-6 flex justify-between items-start">
            <div>
                <h1 class="text-xl lg:text-3xl font-semibold tracking-tight text-(--text-primary)">
                    Reading
                </h1>
            </div>
        </header>

        <div
            v-if="totalBooks === 0"
            class="py-16 text-center"
        >
            <span class="material-symbols-outlined text-4xl text-(--text-tertiary) mb-3 block">auto_stories</span>
            <p class="text-base text-(--text-secondary)">No books yet.</p>
            <p class="text-sm text-(--text-tertiary) mt-1">Add a book to start tracking your reading.</p>
        </div>

        <template v-else>
            <div class="grid grid-cols-2 gap-3 mb-3">
                <StatCard
                    label="Streak"
                    icon="local_fire_department"
                    :value="currentStreak"
                    :unit="currentStreak === 1 ? 'day' : 'days'"
                    :subtitle="`Best: ${bestStreak} ${bestStreak === 1 ? 'day' : 'days'}`"
                />

                <StatCard
                    label="Today"
                    icon="today"
                    :value="todaySeconds > 0 ? formatDuration(todaySeconds) : '-'"
                    :subtitle="`This week: ${readThisWeek > 0 ? formatDuration(readThisWeek) : 'None yet'}`"
                />
            </div>

            <WeeklyActivityChart
                :bars="weeklyBars"
                :todayISO="todayISO"
            />

            <div class="grid grid-cols-3 gap-3 mb-3">
                <StatCard
                    label="Total"
                    :value="totalBooks"
                    unit="books"
                />
                <StatCard
                    label="Reading"
                    :value="booksInProgress"
                    unit="in progress"
                />
                <StatCard
                    label="Done"
                    :value="booksFinished"
                    unit="finished"
                />
            </div>

            <div
                v-if="totalReadingSeconds > 0"
                class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 mb-3"
            >
                <div class="flex items-baseline justify-between mb-3">
                    <p class="text-xs font-medium tracking-wider text-(--text-tertiary)">All time</p>
                    <div class="flex items-baseline gap-1.5">
                        <span class="text-lg font-semibold text-(--text-primary) leading-none">
                            {{ formatDuration(totalReadingSeconds) }}
                        </span>
                        <span class="text-xs text-(--text-secondary)">total</span>
                    </div>
                </div>

                <ReadingChart
                    :data="allTimeDaily"
                    :format-duration="formatDuration"
                />

                <div
                    v-if="topBook"
                    class="mt-3 pt-3 border-t border-(--border-color) flex items-center justify-between gap-2"
                >
                    <div class="min-w-0">
                        <p class="text-xs text-(--text-tertiary)">Most read</p>
                        <p class="text-sm font-medium text-(--text-primary) truncate mt-0.5">{{ topBook.bookTitle }}</p>
                    </div>
                    <span class="text-xs font-semibold tabular-nums text-(--text-secondary) shrink-0">
                        {{ formatDuration(topBook.totalSeconds) }}
                    </span>
                </div>
            </div>

            <div
                v-else
                class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-5 mb-3 text-center"
            >
                <span class="material-symbols-outlined text-2xl text-(--text-tertiary) block mb-2 select-none">schedule</span>
                <p class="text-sm text-(--text-secondary)">Reading time will appear here</p>
                <p class="text-xs text-(--text-tertiary) mt-1">Open a book to start tracking.</p>
            </div>

            <div
                v-if="sessionHistory.length > 0"
                class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 mb-3"
            >
                <p class="text-xs font-medium tracking-wider text-(--text-tertiary) mb-1">History</p>
                <ul class="max-h-80 overflow-y-auto">
                    <li
                        v-for="session in sessionHistory"
                        :key="session.id"
                        class="py-2.5 border-b border-(--border-color) last:border-b-0 flex items-center justify-between gap-2"
                    >
                        <div class="min-w-0">
                            <p class="text-sm font-medium text-(--text-primary) truncate">{{ session.bookTitle }}</p>
                            <p class="text-xs text-(--text-tertiary) mt-0.5">{{ formatSessionDate(session.startedAt) }}</p>
                        </div>
                        <div class="flex items-center gap-1 shrink-0">
                            <span class="text-xs font-semibold tabular-nums text-(--text-secondary)">
                                {{ formatDuration(session.durationSeconds) }}
                            </span>
                            <button
                                @click="onDeleteSession(session.id)"
                                class="flex items-center justify-center w-7 h-7 rounded-full text-(--text-tertiary) hover:text-(--danger-color,#dc2626) transition-colors focus-ring-minimal"
                                title="Delete session"
                                aria-label="Delete session"
                            >
                                <span class="material-symbols-outlined text-base">delete</span>
                            </button>
                        </div>
                    </li>
                </ul>
            </div>
        </template>
    </div>
</template>
