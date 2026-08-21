<script
    setup
    lang="ts"
>
import { computed } from 'vue'

const props = defineProps<{
    totalBooks: number
    booksFinished: number
    booksInProgress: number
    booksUnstarted: number
    epubCount?: number
    pdfCount?: number
    averageProgress?: number
}>()

const completionPct = computed(() => {
    if (props.totalBooks === 0) return 0
    return Math.round((props.booksFinished / props.totalBooks) * 100)
})

const readingPct = computed(() => {
    if (props.totalBooks === 0) return 0
    return Math.round((props.booksInProgress / props.totalBooks) * 100)
})

const unreadPct = computed(() => {
    if (props.totalBooks === 0) return 0
    return Math.max(0, 100 - completionPct.value - readingPct.value)
})
</script>

<template>
    <div class="w-full flex flex-col justify-between">
        <!-- Header -->
        <div class="flex flex-wrap items-center justify-between gap-2 mb-4">
            <div>
                <span
                    class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) select-none block"
                >
                    Library Status
                </span>
                <h3 class="text-lg sm:text-xl font-serif font-bold text-(--text-primary) mt-0.5">
                    Reading Progress
                </h3>
            </div>

            <div class="flex items-center gap-2">
                <div
                    v-if="(epubCount ?? 0) > 0 || (pdfCount ?? 0) > 0"
                    class="hidden sm:flex items-center gap-1.5 text-[11px]"
                >
                    <span
                        v-if="(epubCount ?? 0) > 0"
                        class="px-2 py-0.5 rounded-full bg-(--text-primary)/5 dark:bg-white/10 border border-(--border-color) dark:border-white/15 font-sans font-semibold text-(--text-secondary)"
                    >
                        {{ epubCount }} EPUB
                    </span>
                    <span
                        v-if="(pdfCount ?? 0) > 0"
                        class="px-2 py-0.5 rounded-full bg-(--text-primary)/5 dark:bg-white/10 border border-(--border-color) dark:border-white/15 font-sans font-semibold text-(--text-secondary)"
                    >
                        {{ pdfCount }} PDF
                    </span>
                </div>

                <span
                    class="text-xs font-sans font-semibold px-3 py-1 rounded-full border bg-(--accent-color-light) text-(--accent-color) border-(--border-color) dark:border-white/20"
                >
                    {{ completionPct }}% Finished
                </span>
            </div>
        </div>

        <!-- Big Typographic Fraction -->
        <div class="flex items-baseline gap-2 mb-4">
            <span
                class="text-3xl sm:text-4xl font-bold font-serif text-(--text-primary) tracking-tight leading-none tabular-nums"
            >
                {{ booksFinished }}
            </span>
            <span class="text-base sm:text-lg font-serif text-(--text-secondary)">
                out of {{ totalBooks }} {{ totalBooks === 1 ? 'book completed' : 'books completed' }}
            </span>
        </div>

        <!-- Sleek Segmented Progress Bar -->
        <div
            class="h-3 w-full rounded-full bg-(--text-primary)/10 dark:bg-white/10 overflow-hidden flex mb-5 border border-(--border-color) dark:border-white/20 p-0.5">
            <div
                v-if="booksFinished > 0"
                class="h-full bg-(--text-primary) rounded-l-full transition-all duration-500"
                :style="{ width: `${(booksFinished / totalBooks) * 100}%` }"
                title="Finished"
            ></div>
            <div
                v-if="booksInProgress > 0"
                class="h-full bg-(--accent-color) transition-all duration-500"
                :class="{
                    'rounded-l-full': booksFinished === 0,
                    'rounded-r-full': booksUnstarted === 0,
                }"
                :style="{ width: `${(booksInProgress / totalBooks) * 100}%` }"
                title="Reading"
            ></div>
            <div
                v-if="booksUnstarted > 0"
                class="h-full bg-(--text-primary)/20 dark:bg-white/20 rounded-r-full transition-all duration-500"
                :style="{ width: `${(booksUnstarted / totalBooks) * 100}%` }"
                title="Unread"
            ></div>
        </div>

        <!-- 3 Clean Status Blocks -->
        <div class="grid grid-cols-3 gap-3">
            <!-- Done -->
            <div
                class="p-3.5 rounded-xl bg-(--text-primary)/5 dark:bg-white/5 border border-(--border-color) dark:border-white/15 flex flex-col justify-between">
                <div class="flex items-center justify-between gap-1 mb-1">
                    <span class="material-symbols-outlined text-base text-(--text-primary)">check_circle</span>
                    <span class="text-xs font-sans font-bold text-(--text-primary) tabular-nums">
                        {{ completionPct }}%
                    </span>
                </div>
                <span class="text-2xl sm:text-3xl font-bold font-serif tabular-nums text-(--text-primary) mt-1">
                    {{ booksFinished }}
                </span>
                <span class="text-xs font-sans font-medium text-(--text-secondary) mt-0.5">
                    {{ booksFinished }} Done
                </span>
            </div>

            <!-- Reading -->
            <div
                class="p-3.5 rounded-xl bg-(--text-primary)/5 dark:bg-white/5 border border-(--border-color) dark:border-white/15 flex flex-col justify-between ring-1 ring-(--accent-color)/40">
                <div class="flex items-center justify-between gap-1 mb-1">
                    <span class="material-symbols-outlined text-base text-(--accent-color)">auto_stories</span>
                    <span class="text-xs font-sans font-bold text-(--accent-color) tabular-nums">
                        {{ readingPct }}%
                    </span>
                </div>
                <span class="text-2xl sm:text-3xl font-bold font-serif tabular-nums text-(--text-primary) mt-1">
                    {{ booksInProgress }}
                </span>
                <span class="text-xs font-sans font-medium text-(--text-secondary) mt-0.5">
                    {{ booksInProgress }} Reading
                </span>
            </div>

            <!-- Unread -->
            <div
                class="p-3.5 rounded-xl bg-(--text-primary)/5 dark:bg-white/5 border border-(--border-color) dark:border-white/15 flex flex-col justify-between">
                <div class="flex items-center justify-between gap-1 mb-1">
                    <span class="material-symbols-outlined text-base text-(--text-tertiary)">bookmark</span>
                    <span class="text-xs font-sans font-bold text-(--text-tertiary) tabular-nums">
                        {{ unreadPct }}%
                    </span>
                </div>
                <span class="text-2xl sm:text-3xl font-bold font-serif tabular-nums text-(--text-primary) mt-1">
                    {{ booksUnstarted }}
                </span>
                <span class="text-xs font-sans font-medium text-(--text-secondary) mt-0.5">
                    {{ booksUnstarted }} Unread
                </span>
            </div>
        </div>
    </div>
</template>
