<script
    setup
    lang="ts"
>
import { computed } from 'vue'
import { useReadingStats } from '../composables/useReadingStats'
import { deleteSession } from '../stores/sessions'
import ReadingChart from '../components/stats/ReadingChart.vue'

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

/** Max bar height in px for the weekly chart */
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

// Most read book from sessions
const topBook = computed(() => bookActivity.value[0] ?? null)

</script>

<template>
    <div class="w-full animate-fade-in font-serif pb-[calc(3rem+env(safe-area-inset-bottom,0px))]">
        <!-- Header -->
        <header class="pb-6 flex justify-between items-start">
            <div>
                <h1 class="text-xl lg:text-3xl font-semibold tracking-tight text-(--text-primary)">
                    Reading
                </h1>
            </div>
        </header>

        <!-- Empty state -->
        <div
            v-if="totalBooks === 0"
            class="py-16 text-center"
        >
            <span class="material-symbols-outlined text-4xl text-(--text-tertiary) mb-3 block">auto_stories</span>
            <p class="text-base text-(--text-secondary)">No books yet.</p>
            <p class="text-sm text-(--text-tertiary) mt-1">Add a book to start tracking your reading.</p>
        </div>

        <template v-else>
            <!-- ── Streak + Today row ─────────────────────────────────────── -->
            <div class="grid grid-cols-2 gap-3 mb-3">
                <!-- Current streak -->
                <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 flex flex-col gap-1">
                    <div class="flex items-center gap-1.5 text-(--text-tertiary) text-xs font-medium tracking-wider">
                        <span class="material-symbols-outlined text-sm select-none">local_fire_department</span>
                        Streak
                    </div>
                    <div class="flex items-end gap-1.5 mt-1">
                        <span class="text-3xl font-semibold tabular-nums text-(--text-primary) leading-none">
                            {{ currentStreak }}
                        </span>
                        <span class="text-sm text-(--text-secondary) mb-0.5">
                            {{ currentStreak === 1 ? 'day' : 'days' }}
                        </span>
                    </div>
                    <p class="text-xs text-(--text-tertiary) mt-0.5">
                        Best: {{ bestStreak }} {{ bestStreak === 1 ? 'day' : 'days' }}
                    </p>
                </div>

                <!-- Today's reading -->
                <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 flex flex-col gap-1">
                    <div class="flex items-center gap-1.5 text-(--text-tertiary) text-xs font-medium tracking-wider">
                        <span class="material-symbols-outlined text-sm select-none">today</span>
                        Today
                    </div>
                    <div class="flex items-end gap-1.5 mt-1">
                        <span class="text-3xl font-semibold tabular-nums text-(--text-primary) leading-none">
                            {{ todaySeconds > 0 ? formatDuration(todaySeconds) : '—' }}
                        </span>
                    </div>
                    <p class="text-xs text-(--text-tertiary) mt-0.5">
                        This week: {{ readThisWeek > 0 ? formatDuration(readThisWeek) : 'None yet' }}
                    </p>
                </div>
            </div>

            <!-- ── Weekly activity chart ──────────────────────────────────── -->
            <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 mb-3">
                <p class="text-xs font-medium tracking-wider text-(--text-tertiary) mb-4">This week</p>
                <div
                    class="flex items-end justify-between gap-1.5"
                    style="height: 80px;"
                >
                    <div
                        v-for="bar in weeklyBars"
                        :key="bar.date"
                        class="flex-1 flex flex-col items-center justify-end gap-1.5"
                    >
                        <div
                            class="w-full rounded-full transition-all duration-500"
                            :class="[
                                bar.date === todayISO
                                    ? 'bg-(--text-primary)'
                                    : bar.active
                                        ? 'bg-(--text-secondary)'
                                        : 'bg-(--border-color)',
                            ]"
                            :style="{ height: bar.active ? bar.height + 'px' : '4px' }"
                        ></div>
                        <span
                            class="text-[10px] font-medium select-none tabular-nums"
                            :class="bar.date === todayISO ? 'text-(--text-primary) font-semibold' : 'text-(--text-tertiary)'"
                        >
                            {{ bar.label }}
                        </span>
                    </div>
                </div>
            </div>

            <!-- ── Library overview ──────────────────────────────────────── -->
            <div class="grid grid-cols-3 gap-3 mb-3">
                <!-- Total -->
                <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 flex flex-col gap-1">
                    <span class="text-xs font-medium tracking-wider text-(--text-tertiary)">Total</span>
                    <span class="text-3xl font-semibold tabular-nums text-(--text-primary) leading-tight mt-1">{{
                        totalBooks }}</span>
                    <span class="text-[11px] text-(--text-tertiary)">books</span>
                </div>

                <!-- In progress -->
                <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 flex flex-col gap-1">
                    <span class="text-xs font-medium tracking-wider text-(--text-tertiary)">Reading</span>
                    <span class="text-3xl font-semibold tabular-nums text-(--text-primary) leading-tight mt-1">{{
                        booksInProgress }}</span>
                    <span class="text-[11px] text-(--text-tertiary)">in progress</span>
                </div>

                <!-- Finished -->
                <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 flex flex-col gap-1">
                    <span class="text-xs font-medium tracking-wider text-(--text-tertiary)">Done</span>
                    <span class="text-3xl font-semibold tabular-nums leading-tight mt-1">{{ booksFinished }}</span>
                    <span class="text-[11px] text-(--text-tertiary)">finished</span>
                </div>
            </div>



            <!-- ── All-time chart ───────────────────────────────────────── -->
            <div
                v-if="totalReadingSeconds > 0"
                class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 mb-3"
            >
                <!-- Header row -->
                <div class="flex items-baseline justify-between mb-3">
                    <p class="text-xs font-medium tracking-wider text-(--text-tertiary)">All time</p>
                    <div class="flex items-baseline gap-1.5">
                        <span class="text-lg font-semibold text-(--text-primary) leading-none">{{
                            formatDuration(totalReadingSeconds) }}</span>
                        <span class="text-xs text-(--text-secondary)">total</span>
                    </div>
                </div>

                <!-- Chart -->
                <ReadingChart
                    :data="allTimeDaily"
                    :format-duration="formatDuration"
                />

                <!-- Top book by time -->
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

            <!-- Empty session state (books exist but no tracked sessions yet) -->
            <div
                v-else
                class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-5 mb-3 text-center"
            >
                <span
                    class="material-symbols-outlined text-2xl text-(--text-tertiary) block mb-2 select-none">schedule</span>
                <p class="text-sm text-(--text-secondary)">Reading time will appear here</p>
                <p class="text-xs text-(--text-tertiary) mt-1">Open a book to start tracking.</p>
            </div>

            <!-- ── Session history ──────────────────────────────────────── -->
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
