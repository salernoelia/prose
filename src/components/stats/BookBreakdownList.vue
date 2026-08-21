<script setup lang="ts">
import { ref, computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { BookDto, LibraryEntryDto } from '../../ipc/types'

export interface EnrichedBookActivity {
    bookId: string
    book: BookDto
    bookTitle: string
    bookAuthor: string | null
    totalSeconds: number
    progress: number
    format: 'epub' | 'pdf'
    cover: string | null
    lastRead: number | null
    percentageOfTotal: number
    rawEntry?: LibraryEntryDto
}

const props = defineProps<{
    books: EnrichedBookActivity[]
    appDataPath?: string
    formatDuration: (s: number) => string
}>()

const emit = defineEmits<{
    (e: 'select-book', book: BookDto): void
}>()

const showAll = ref(false)
const displayedBooks = computed(() => {
    if (showAll.value || props.books.length <= 4) {
        return props.books
    }
    return props.books.slice(0, 4)
})

const getCoverUrl = (coverPath: string | null) => {
    if (!coverPath || !props.appDataPath) return ''
    try {
        const absolutePath = `${props.appDataPath}/${coverPath}`.replace(/\/+/g, '/')
        if (typeof convertFileSrc === 'function') {
            return convertFileSrc(absolutePath)
        }
        return `asset://localhost/${encodeURIComponent(absolutePath.replace(/^\/+/, ''))}`
    } catch {
        return ''
    }
}

// Typographic fallback cover palettes for books without image covers
const COVER_PALETTES = [
    { bg: 'bg-[#09332C] text-[#F7EDDA]', border: 'border-[#FFA74F]/40' },
    { bg: 'bg-[#2C1E13] text-[#F7DFBA]', border: 'border-[#F7DFBA]/30' },
    { bg: 'bg-[#2E4B3C] text-[#F7EDDA]', border: 'border-[#F7EDDA]/30' },
    { bg: 'bg-[#1C1917] text-[#F7EDDA]', border: 'border-[#F0531C]/40' },
    { bg: 'bg-[#4A2E35] text-[#FCEDF0]', border: 'border-[#F2C9D2]/30' },
]

function getFallbackPalette(title: string, author?: string | null) {
    let hash = 0
    const str = title + (author || '')
    for (let i = 0; i < str.length; i++) {
        hash = (hash << 5) - hash + str.charCodeAt(i)
        hash |= 0
    }
    return COVER_PALETTES[Math.abs(hash) % COVER_PALETTES.length]
}
</script>

<template>
    <div class="w-full">
        <!-- Header -->
        <div class="flex items-center justify-between gap-2 mb-4">
            <div>
                <span class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) select-none block">
                    Most Read Books
                </span>
                <h3 class="text-lg sm:text-xl font-serif font-bold text-(--text-primary) mt-0.5">
                    Reading Breakdown
                </h3>
            </div>

            <button
                v-if="books.length > 4"
                @click="showAll = !showAll"
                class="text-xs font-sans font-semibold text-(--accent-color) hover:underline cursor-pointer select-none px-2 py-1"
            >
                {{ showAll ? 'Show less' : `Show all (${books.length})` }}
            </button>
        </div>

        <!-- Book List Rows -->
        <div class="flex flex-col divide-y divide-(--border-color)/40 dark:divide-white/10">
            <div
                v-for="item in displayedBooks"
                :key="item.bookId"
                @click="emit('select-book', item.book)"
                class="py-3.5 first:pt-0 last:pb-0 flex items-center justify-between gap-3 group cursor-pointer hover:bg-(--text-primary)/4 dark:hover:bg-white/5 px-2 -mx-2 rounded-xl transition-all"
            >
                <!-- Left: Mini Cover & Book Info -->
                <div class="flex items-center gap-3 min-w-0 flex-1">
                    <!-- Mini 3D Book Cover -->
                    <div class="w-10 h-14 shrink-0 rounded book-cover-3d overflow-hidden relative shadow-sm group-hover:scale-105 transition-transform">
                        <img
                            v-if="item.cover && appDataPath && getCoverUrl(item.cover)"
                            :src="getCoverUrl(item.cover)"
                            alt="cover"
                            class="w-full h-full object-cover"
                        />
                        <div
                            v-else
                            class="w-full h-full flex flex-col justify-center items-center text-center p-1"
                            :class="getFallbackPalette(item.bookTitle, item.bookAuthor).bg"
                        >
                            <span class="text-[8px] font-sans uppercase font-bold tracking-widest leading-none">
                                {{ item.format }}
                            </span>
                        </div>
                    </div>

                    <!-- Details -->
                    <div class="min-w-0 flex-1">
                        <div class="flex items-center gap-2">
                            <h4 class="text-sm sm:text-base font-semibold font-serif text-(--text-primary) truncate group-hover:text-(--accent-color) transition-colors">
                                {{ item.bookTitle }}
                            </h4>
                            <span class="px-1.5 py-0.5 rounded bg-(--text-primary)/5 dark:bg-white/10 border border-(--border-color)/60 dark:border-white/15 text-[9px] font-sans uppercase tracking-wider text-(--text-tertiary) font-semibold shrink-0">
                                {{ item.format }}
                            </span>
                        </div>

                        <p class="text-xs font-sans text-(--text-secondary) truncate mt-0.5">
                            {{ item.bookAuthor || 'Unknown Author' }}
                        </p>

                        <!-- Level progress bar -->
                        <div class="w-full max-w-[220px] h-1.5 rounded-full bg-(--text-primary)/10 dark:bg-white/10 overflow-hidden mt-2 border border-(--border-color)/40 dark:border-white/15">
                            <div
                                class="h-full bg-(--accent-color) rounded-full transition-all duration-300"
                                :style="{ width: `${Math.round(item.progress * 100)}%` }"
                            ></div>
                        </div>
                    </div>
                </div>

                <!-- Right: Time Logged & Progress -->
                <div class="flex flex-col items-end shrink-0 pl-2 text-right">
                    <span class="text-sm sm:text-base font-bold font-sans tabular-nums text-(--text-primary)">
                        {{ formatDuration(item.totalSeconds) }}
                    </span>
                    <span class="text-xs font-sans tabular-nums text-(--text-secondary) mt-0.5">
                        {{ Math.round(item.progress * 100) }}% complete
                    </span>
                    <span
                        v-if="item.percentageOfTotal > 0"
                        class="text-[10px] font-sans font-semibold text-(--accent-color) mt-0.5"
                    >
                        {{ item.percentageOfTotal }}% of total
                    </span>
                </div>
            </div>
        </div>
    </div>
</template>
