<script
    setup
    lang="ts"
>
import { ref, computed, onMounted } from 'vue'
import { appDataDir } from '@tauri-apps/api/path'
import { useReadingStats } from '../composables/useReadingStats'
import { deleteSession } from '../stores/sessions'
import type { BookDto } from '../ipc/types'
import {
    WeeklyActivityChart,
    ReadingChart,
    LibraryProgressMatrix,
    GoalSpeedometer,
    BookBreakdownList,
    TimeDistributionChart,
    SessionHistoryList,
} from '../components/stats'
import type { Timeframe } from '../components/stats/ReadingChart.vue'

const emit = defineEmits<{
    (e: 'select-book', book: BookDto): void
}>()

const {
    totalBooks,
    booksFinished,
    booksInProgress,
    booksUnstarted,
    epubCount,
    pdfCount,
    averageProgress,
    totalReadingSeconds,
    todaySeconds,
    currentStreak,
    bestStreak,
    weeklyActivity,
    enrichedBookActivity,
    timeOfDayDistribution,
    sessionHistory,
    getTrendPoints,
    formatDuration,
} = useReadingStats()

const appDataPath = ref('')
const timeframe = ref<Timeframe>('30d')

onMounted(async () => {
    try {
        appDataPath.value = await appDataDir()
    } catch {
        // Fallback for browser/test environments
        appDataPath.value = ''
    }
})

function onDeleteSession(id: string) {
    void deleteSession(id)
}

function onSelectBook(book: BookDto) {
    emit('select-book', book)
}

const todayISO = (() => {
    const d = new Date()
    const y = d.getFullYear()
    const m = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    return `${y}-${m}-${day}`
})()

const maxWeeklySeconds = computed(() =>
    Math.max(...weeklyActivity.value.map((d) => d.totalSeconds), 1),
)

const weeklyBars = computed(() =>
    weeklyActivity.value.map((d) => ({
        ...d,
        height: Math.max(8, Math.round((d.totalSeconds / maxWeeklySeconds.value) * 86)),
        active: d.totalSeconds > 0,
    })),
)

const activeTrendData = computed(() => getTrendPoints(timeframe.value))
</script>

<template>
    <div class="w-full animate-fade-in font-serif pb-[calc(4rem+env(safe-area-inset-bottom,0px))]">
        <!-- Editorial Header with High Contrast Divider -->
        <header
            class="pb-6 pt-4 border-b border-(--border-color) dark:border-white/20 mb-8 flex flex-wrap items-end justify-between gap-3"
        >
            <div>
                <h1 class="text-2xl lg:text-4xl font-semibold tracking-tight text-(--text-primary) font-serif">
                    Reading Insights
                </h1>
            </div>

            <!-- Header Quick Summary Badges -->
            <div
                v-if="totalReadingSeconds > 0"
                class="flex items-center gap-2 select-none"
            >
                <span
                    class="px-3.5 py-1 rounded-full bg-(--text-primary)/5 dark:bg-white/10 border border-(--border-color) dark:border-white/20 text-xs font-sans font-medium text-(--text-secondary) shadow-xs"
                >
                    Total: <strong class="text-(--text-primary) font-bold font-sans tabular-nums">{{
                        formatDuration(totalReadingSeconds) }}</strong>
                </span>
                <span
                    v-if="currentStreak > 0"
                    class="flex items-center gap-1.5 px-3.5 py-1 rounded-full bg-(--accent-color-light) border border-(--border-color) dark:border-white/20 text-xs font-sans font-semibold text-(--accent-color) shadow-xs"
                >
                    <span class="material-symbols-outlined text-sm">local_fire_department</span>
                    <span>{{ currentStreak }} {{ currentStreak === 1 ? 'day' : 'days' }}</span>
                </span>
            </div>
        </header>

        <!-- Empty State (No Books) -->
        <div
            v-if="totalBooks === 0"
            class="py-20 text-center"
        >
            <span
                class="material-symbols-outlined text-5xl text-(--accent-color) mb-4 block select-none">auto_stories</span>
            <h2 class="text-xl font-serif font-bold text-(--text-primary)">Your library is waiting</h2>
            <p class="text-sm font-sans text-(--text-secondary) mt-1 max-w-sm mx-auto">
                Add an EPUB or PDF to start your reading journey and view detailed statistics.
            </p>
        </div>

        <!-- Borderless Editorial Stream with High Contrast Section Dividers & Generous Spacing -->
        <div
            v-else
            class="space-y-16 sm:space-y-20"
        >
            <!-- 1. Reading Streak Section -->
            <section class="w-full">
                <div class="flex flex-wrap items-center justify-between gap-2 mb-4 select-none">
                    <div
                        class="flex items-center gap-2 text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary)">
                        <span
                            class="material-symbols-outlined text-lg text-(--accent-color)">local_fire_department</span>
                        <span>Reading Streak</span>
                    </div>
                    <span
                        class="text-xs font-sans font-semibold px-3 py-1 rounded-full bg-(--accent-color-light) text-(--accent-color) border border-(--border-color) dark:border-white/20"
                    >
                        Best: {{ bestStreak }} {{ bestStreak === 1 ? 'day' : 'days' }}
                    </span>
                </div>

                <div class="flex items-baseline gap-2 mt-2">
                    <span
                        class="text-4xl sm:text-5xl font-bold font-serif text-(--text-primary) tracking-tight leading-none tabular-nums"
                    >
                        {{ currentStreak }}
                    </span>
                    <span class="text-base sm:text-lg font-serif text-(--text-secondary)">
                        {{ currentStreak === 1 ? 'day in a row' : 'days in a row' }}
                    </span>
                </div>

                <!-- 7-Day Streak Momentum Pills -->
                <div class="mt-6">
                    <p class="text-[11px] font-sans text-(--text-tertiary) uppercase tracking-wider mb-3 select-none">
                        This week's momentum
                    </p>
                    <div class="grid grid-cols-7 gap-2 sm:gap-3 text-center">
                        <div
                            v-for="bar in weeklyActivity"
                            :key="bar.date"
                            class="flex flex-col items-center gap-1.5"
                        >
                            <div
                                class="w-full max-w-[48px] aspect-square rounded-xl flex items-center justify-center transition-all duration-300"
                                :class="[
                                    bar.totalSeconds > 0
                                        ? 'bg-(--accent-color) text-(--accent-ink) shadow-xs'
                                        : bar.date === todayISO
                                            ? 'border-2 border-dashed border-(--accent-color) bg-(--text-primary)/5 dark:bg-white/10'
                                            : 'bg-(--text-primary)/4 dark:bg-white/5 border border-(--border-color)/80 dark:border-white/15 text-(--text-tertiary)'
                                ]"
                            >
                                <span
                                    v-if="bar.totalSeconds > 0"
                                    class="material-symbols-outlined text-sm font-bold"
                                >
                                    check
                                </span>
                                <span
                                    v-else-if="bar.date === todayISO"
                                    class="w-2 h-2 rounded-full bg-(--accent-color)"
                                ></span>
                            </div>
                            <span
                                class="text-xs font-sans"
                                :class="bar.date === todayISO ? 'font-bold text-(--accent-color)' : 'text-(--text-secondary)'"
                            >
                                {{ bar.label }}
                            </span>
                        </div>
                    </div>
                </div>

                <p class="text-xs font-sans text-(--text-secondary) mt-4">
                    <template v-if="todaySeconds > 0">
                        You have completed today's reading session.
                    </template>
                    <template v-else-if="currentStreak > 0">
                        Read today to keep your {{ currentStreak }}-day streak going!
                    </template>
                    <template v-else>
                        Complete a reading session today to begin a new streak.
                    </template>
                </p>
            </section>

            <!-- 2. Today's Goal Pace -->
            <section class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20">
                <GoalSpeedometer
                    :current-seconds="todaySeconds"
                    :target-seconds="1800"
                    label="Today's Pace"
                    :format-duration="formatDuration"
                />
            </section>

            <!-- 3. Weekly Equalizer Chart -->
            <section class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20">
                <WeeklyActivityChart
                    :bars="weeklyBars"
                    :todayISO="todayISO"
                    :format-duration="formatDuration"
                />
            </section>

            <!-- 4. Library Reading Progress -->
            <section class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20">
                <LibraryProgressMatrix
                    :total-books="totalBooks"
                    :books-finished="booksFinished"
                    :books-in-progress="booksInProgress"
                    :books-unstarted="booksUnstarted"
                    :epub-count="epubCount"
                    :pdf-count="pdfCount"
                    :average-progress="averageProgress"
                />
            </section>

            <!-- 5. Reading Velocity Trends -->
            <section class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20">
                <div class="mb-4">
                    <span
                        class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) select-none block"
                    >
                        Activity Trends
                    </span>
                    <h3 class="text-lg sm:text-xl font-serif font-bold text-(--text-primary) mt-0.5">
                        Reading Velocity
                    </h3>
                </div>

                <ReadingChart
                    :data="activeTrendData"
                    :format-duration="formatDuration"
                    v-model:timeframe="timeframe"
                    :show-controls="true"
                />
            </section>

            <!-- 6. Most Read Books Breakdown -->
            <section
                v-if="enrichedBookActivity.length > 0"
                class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20"
            >
                <BookBreakdownList
                    :books="enrichedBookActivity"
                    :app-data-path="appDataPath"
                    :format-duration="formatDuration"
                    @select-book="onSelectBook"
                />
            </section>

            <!-- 7. Habits & Time of Day -->
            <section
                v-if="totalReadingSeconds > 0"
                class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20"
            >
                <TimeDistributionChart
                    :distribution="timeOfDayDistribution"
                    :format-duration="formatDuration"
                />
            </section>

            <!-- 8. Reading History Log -->
            <section
                v-if="sessionHistory.length > 0"
                class="w-full pt-12 sm:pt-16 border-t border-(--border-color) dark:border-white/20"
            >
                <SessionHistoryList
                    :sessions="sessionHistory"
                    :format-duration="formatDuration"
                    @delete-session="onDeleteSession"
                />
            </section>
        </div>
    </div>
</template>
