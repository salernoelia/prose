<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { appDataDir } from '@tauri-apps/api/path'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useLibrary } from '../composables/useLibrary'
import { useReadingStats } from '../composables/useReadingStats'
import { useSync } from '../composables/useSync'
import GoalSpeedometer from '../components/stats/GoalSpeedometer.vue'
import type { BookDto } from '../ipc/types'

const emit = defineEmits<{
    (e: 'select-book', book: BookDto): void
    (e: 'navigate', view: 'library' | 'stats' | 'settings'): void
}>()

const {
    entries,
    loaded,
    importBook,
} = useLibrary()

const {
    todaySeconds,
    currentStreak,
    enrichedBookActivity,
    formatDuration,
} = useReadingStats()

const {
    configured,
    syncing,
    hasSyncError,
    triggerSync,
} = useSync()

const appDataPath = ref('')

onMounted(async () => {
    try {
        appDataPath.value = await appDataDir()
    } catch {
        appDataPath.value = ''
    }
})

function getCoverUrl(coverPath: string | null): string | null {
    if (!coverPath || !appDataPath.value) return null
    const normalizedAppData = appDataPath.value.replace(/[\\/]+$/, '')
    const cleanCover = coverPath.replace(/^[\\/]+/, '')
    const fullPath = `${normalizedAppData}/${cleanCover}`
    return convertFileSrc(fullPath)
}

// Top 3 most read or active books
interface PosedBook {
    book: BookDto
    progress: number
    totalSeconds: number
    lastRead: number | null
    coverUrl: string | null
}

const topThreeBooks = computed<PosedBook[]>(() => {
    if (!entries.value || entries.value.length === 0) return []

    // 1. If we have reading activity history, prioritize books with highest reading time
    if (enrichedBookActivity.value.length > 0) {
        const sorted = [...enrichedBookActivity.value]
            .filter((b) => b.totalSeconds > 0 || b.progress > 0)
            .sort((a, b) => b.totalSeconds - a.totalSeconds || b.progress - a.progress)
            .slice(0, 3)

        if (sorted.length > 0) {
            return sorted.map((item) => ({
                book: item.book,
                progress: item.progress,
                totalSeconds: item.totalSeconds,
                lastRead: item.lastRead,
                coverUrl: getCoverUrl(item.book.cover),
            }))
        }
    }

    // 2. Fallback to recent in-progress or library entries
    const inProgress = entries.value
        .filter((e) => !e.archived)
        .sort((a, b) => (b.progress > 0 ? 1 : 0) - (a.progress > 0 ? 1 : 0) || (b.lastRead ?? 0) - (a.lastRead ?? 0))
        .slice(0, 3)

    return inProgress.map((e) => ({
        book: e.book,
        progress: e.progress,
        totalSeconds: 0,
        lastRead: e.lastRead,
        coverUrl: getCoverUrl(e.book.cover),
    }))
})

async function handleImport() {
    try {
        const selected = await open({
            multiple: true,
            filters: [{ name: 'Books', extensions: ['epub', 'pdf'] }],
        })
        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected]
            for (const path of paths) {
                if (typeof path === 'string') {
                    await importBook(path)
                }
            }
        }
    } catch (err) {
        console.error('Import failed:', err)
    }
}
</script>

<template>
    <div class="w-full animate-fade-in font-serif">
        <!-- Editorial Header -->
        <header
            class="pb-6 pt-4 border-b border-(--border-color) dark:border-white/20 mb-8 flex flex-wrap items-end justify-between gap-3 select-none"
        >
            <div>
                <h1 class="text-2xl lg:text-4xl font-semibold tracking-tight text-(--text-primary) font-serif">
                    Home
                </h1>
            </div>

            <!-- Quick Sync & Streak Status -->
            <div class="flex items-center gap-2">
                <span
                    v-if="currentStreak > 0"
                    class="flex items-center gap-1 px-3 py-1.5 rounded-full bg-(--accent-color-light) border border-(--border-color) dark:border-white/20 text-xs font-sans font-semibold text-(--accent-color) shadow-xs"
                >
                    <span class="material-symbols-outlined text-sm">local_fire_department</span>
                    <span>{{ currentStreak }} {{ currentStreak === 1 ? 'day' : 'days' }}</span>
                </span>

                <button
                    v-if="configured"
                    @click="triggerSync"
                    :disabled="syncing"
                    class="px-3.5 py-1.5 text-xs font-sans font-medium rounded-full border text-(--text-primary) bg-(--bg-card) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed shadow-xs"
                    :class="hasSyncError
                        ? 'border-red-500 text-red-500 dark:border-red-500 dark:text-red-400'
                        : 'border-(--border-color) dark:border-white/20'"
                >
                    <span
                        class="material-symbols-outlined text-base select-none"
                        :class="{ 'animate-spin': syncing, 'text-red-500 dark:text-red-400': hasSyncError && !syncing }"
                    >sync</span>
                    <span>{{ syncing ? 'Syncing' : 'Sync' }}</span>
                </button>
            </div>
        </header>

        <!-- Empty State (No Books in Library) -->
        <div
            v-if="loaded && entries.length === 0"
            class="py-20 text-center select-none"
        >
            <span class="material-symbols-outlined text-5xl text-(--accent-color) mb-4 block">auto_stories</span>
            <h2 class="text-2xl font-serif font-bold text-(--text-primary)">Your library is quiet</h2>
            <p class="text-sm font-sans text-(--text-secondary) mt-1.5 max-w-sm mx-auto mb-6">
                Add your favorite EPUB or PDF books to begin your reading journey.
            </p>
            <button
                @click="handleImport"
                class="px-5 py-2.5 rounded-full bg-(--text-primary) text-(--bg-app) text-xs font-semibold hover:opacity-90 transition-opacity cursor-pointer inline-flex items-center gap-2 shadow-xs"
            >
                <span class="material-symbols-outlined text-base leading-none">add</span>
                <span>Import Book</span>
            </button>
        </div>

        <div v-else class="space-y-12 sm:space-y-16">
            <!-- 1. Daily Reading Goal Gauge (Topmost directly below header) -->
            <section class="w-full max-w-sm mx-auto select-none">
                <GoalSpeedometer
                    :current-seconds="todaySeconds"
                    :target-seconds="1800"
                    label="Today's Pace"
                    :format-duration="formatDuration"
                />
            </section>

            <!-- 2. The 3 Most Read Books (Exposé without cards) -->
            <section v-if="topThreeBooks.length > 0" class="w-full max-w-xl mx-auto">
                <div class="mb-8 text-center select-none">
                    <span class="text-[11px] font-sans font-medium uppercase tracking-widest text-(--text-tertiary) block">
                        Pick up where you left off
                    </span>
                    <h2 class="text-xl sm:text-2xl font-bold font-serif text-(--text-primary) mt-0.5">
                        Continue Reading
                    </h2>
                </div>

                <!-- Exposed Books List (Vertically & Horizontally Centered) -->
                <div class="space-y-10 sm:space-y-12">
                    <div
                        v-for="(item, idx) in topThreeBooks"
                        :key="item.book.id"
                        @click="emit('select-book', item.book)"
                        class="group cursor-pointer active:scale-[0.99] transition-transform select-none flex items-center justify-center gap-6 sm:gap-8 max-w-lg mx-auto w-full"
                    >
                        <!-- Big Book 3D Cover on Left -->
                        <div class="w-24 h-34 sm:w-28 sm:h-40 book-cover-3d shrink-0 overflow-hidden flex items-center justify-center rounded-sm shadow-md">
                            <img
                                v-if="item.coverUrl"
                                :src="item.coverUrl"
                                alt=""
                                class="w-full h-full object-cover"
                            />
                            <div
                                v-else
                                class="w-full h-full p-3 bg-[#09332C] text-[#F7EDDA] flex flex-col justify-between items-center text-center select-none"
                            >
                                <span class="text-[8px] uppercase tracking-widest opacity-60">{{ item.book.format }}</span>
                                <span class="text-[11px] sm:text-xs font-serif font-semibold line-clamp-3 leading-tight">{{ item.book.title }}</span>
                                <span></span>
                            </div>
                        </div>

                        <!-- Details on Right (Vertically Centered with Cover) -->
                        <div class="flex flex-col justify-center min-w-0 flex-1 py-1">
                            <span class="text-[10px] sm:text-[11px] font-sans font-semibold uppercase tracking-widest text-(--accent-color) select-none mb-1">
                                {{ idx === 0 ? 'Current Read' : `Top Pick #${idx + 1}` }}
                            </span>
                            <h3 class="text-lg sm:text-xl md:text-2xl font-bold font-serif text-(--text-primary) line-clamp-2 leading-snug group-hover:text-(--accent-color) transition-colors">
                                {{ item.book.title }}
                            </h3>
                            <p class="text-xs sm:text-sm font-serif text-(--text-secondary) truncate mt-1">
                                {{ item.book.author || 'Unknown Author' }}
                            </p>

                            <!-- Reading Progress Bar & Details -->
                            <div class="mt-3.5 sm:mt-4 w-full max-w-sm flex flex-col gap-1.5">
                                <div class="h-1.5 w-full bg-(--text-primary)/10 dark:bg-white/10 rounded-full overflow-hidden">
                                    <div
                                        class="h-full bg-(--accent-color) rounded-full transition-all duration-300"
                                        :style="{ width: `${item.progress * 100}%` }"
                                    ></div>
                                </div>
                                <div class="flex items-center gap-2 text-xs font-sans tabular-nums">
                                    <span class="font-medium text-(--text-primary)">
                                        {{ Math.round(item.progress * 100) }}% completed
                                    </span>
                                    <span v-if="item.totalSeconds > 0" class="text-(--text-tertiary)">
                                        · {{ formatDuration(item.totalSeconds) }} read
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </section>
        </div>
    </div>
</template>
