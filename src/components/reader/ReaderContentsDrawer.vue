<script setup lang="ts">
import { ref, computed } from 'vue'
import Drawer from 'primevue/drawer'
import type { TocItem } from '../../readers'
import type { BookmarkDto, HighlightDto } from '../../ipc/types'
import ReaderTocList from './ReaderTocList.vue'

const props = withDefaults(
    defineProps<{
        visible: boolean
        initialTab?: 'contents' | 'bookmarks' | 'highlights'
        tocItems: TocItem[]
        bookmarks: BookmarkDto[]
        highlights: HighlightDto[]
        bookTitle?: string
    }>(),
    {
        initialTab: 'contents',
        tocItems: () => [],
        bookmarks: () => [],
        highlights: () => [],
        bookTitle: '',
    }
)

const emit = defineEmits<{
    (e: 'update:visible', value: boolean): void
    (e: 'select-toc', href: string): void
    (e: 'select-bookmark', bookmark: BookmarkDto): void
    (e: 'delete-bookmark', id: string): void
    (e: 'select-highlight', highlight: HighlightDto): void
    (e: 'delete-highlight', id: string): void
}>()

const activeTab = ref<'contents' | 'bookmarks' | 'highlights'>(props.initialTab)
const searchQuery = ref('')

function formatDate(ms: number): string {
    return new Date(ms).toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
    })
}

function progressLabel(progression: number): string {
    return `${Math.round(progression * 100)}%`
}

function onSelectToc(href: string) {
    emit('select-toc', href)
    emit('update:visible', false)
}

function onSelectBookmark(bookmark: BookmarkDto) {
    emit('select-bookmark', bookmark)
    emit('update:visible', false)
}

function onSelectHighlight(highlight: HighlightDto) {
    emit('select-highlight', highlight)
    emit('update:visible', false)
}

function copyText(text: string) {
    if (text && navigator.clipboard) {
        void navigator.clipboard.writeText(text)
    }
}

// Flatten and filter TOC items if searching
function filterToc(items: TocItem[], query: string): TocItem[] {
    if (!query.trim()) return items
    const q = query.toLowerCase()
    const result: TocItem[] = []

    for (const item of items) {
        const matchesSelf = item.label.toLowerCase().includes(q)
        const matchedSubitems = filterToc(item.subitems, query)
        if (matchesSelf || matchedSubitems.length > 0) {
            result.push({
                ...item,
                subitems: matchedSubitems,
            })
        }
    }
    return result
}

const filteredToc = computed(() => filterToc(props.tocItems, searchQuery.value))

const filteredBookmarks = computed(() => {
    if (!searchQuery.value.trim()) return props.bookmarks
    const q = searchQuery.value.toLowerCase()
    return props.bookmarks.filter((b) =>
        progressLabel(b.locator.progression).includes(q)
    )
})

const filteredHighlights = computed(() => {
    if (!searchQuery.value.trim()) return props.highlights
    const q = searchQuery.value.toLowerCase()
    return props.highlights.filter((h) =>
        h.text.toLowerCase().includes(q)
    )
})
</script>

<template>
    <Drawer
        :visible="visible"
        @update:visible="emit('update:visible', $event)"
        position="right"
        :modal="true"
        :show-close-icon="false"
        class="!w-96 !max-w-[92vw] !bg-(--bg-app) !border-l !border-(--border-color) font-serif"
    >
        <template #container="{ closeCallback }">
            <div class="flex flex-col h-full bg-(--bg-app) text-(--text-primary)">
                <!-- Header -->
                <header
                    class="px-5 pb-3 border-b border-(--border-color) flex flex-col gap-3"
                    :style="{ paddingTop: 'calc(1rem + env(safe-area-inset-top, 0px))' }"
                >
                    <div class="flex items-center justify-between">
                        <div>
                            <span class="text-[10px] font-sans font-medium uppercase tracking-widest text-(--text-tertiary) select-none">
                                Navigation & Notes
                            </span>
                            <h2 class="text-base font-semibold tracking-tight text-(--text-primary) truncate max-w-[200px]">
                                {{ bookTitle || 'Book Details' }}
                            </h2>
                        </div>
                        <button
                            @click="closeCallback"
                            class="flex items-center justify-center w-8 h-8 rounded-full border border-(--border-color) bg-(--bg-card) text-(--text-secondary) hover:text-(--text-primary) transition-all cursor-pointer focus-ring-minimal active:scale-90"
                            title="Close"
                            aria-label="Close"
                        >
                            <span class="material-symbols-outlined text-lg leading-none select-none">close</span>
                        </button>
                    </div>

                    <!-- Segmented Tab Navigation -->
                    <div class="flex items-center p-1 bg-(--bg-card) border border-(--border-color) rounded-full select-none shadow-xs">
                        <button
                            @click="activeTab = 'contents'"
                            class="flex-1 py-1.5 px-2 rounded-full text-xs font-medium transition-all duration-150 cursor-pointer flex items-center justify-center gap-1.5"
                            :class="activeTab === 'contents'
                                ? 'bg-(--accent-color-light) text-(--text-primary) font-semibold shadow-xs'
                                : 'text-(--text-secondary) hover:text-(--text-primary)'"
                        >
                            <span class="material-symbols-outlined text-sm leading-none">toc</span>
                            <span>Contents</span>
                        </button>

                        <button
                            @click="activeTab = 'bookmarks'"
                            class="flex-1 py-1.5 px-2 rounded-full text-xs font-medium transition-all duration-150 cursor-pointer flex items-center justify-center gap-1.5"
                            :class="activeTab === 'bookmarks'
                                ? 'bg-(--accent-color-light) text-(--text-primary) font-semibold shadow-xs'
                                : 'text-(--text-secondary) hover:text-(--text-primary)'"
                        >
                            <span class="material-symbols-outlined text-sm leading-none">bookmark</span>
                            <span>Bookmarks</span>
                            <span v-if="bookmarks.length" class="text-[10px] opacity-70">({{ bookmarks.length }})</span>
                        </button>

                        <button
                            @click="activeTab = 'highlights'"
                            class="flex-1 py-1.5 px-2 rounded-full text-xs font-medium transition-all duration-150 cursor-pointer flex items-center justify-center gap-1.5"
                            :class="activeTab === 'highlights'
                                ? 'bg-(--accent-color-light) text-(--text-primary) font-semibold shadow-xs'
                                : 'text-(--text-secondary) hover:text-(--text-primary)'"
                        >
                            <span class="material-symbols-outlined text-sm leading-none">format_ink_highlighter</span>
                            <span>Notes</span>
                            <span v-if="highlights.length" class="text-[10px] opacity-70">({{ highlights.length }})</span>
                        </button>
                    </div>

                    <!-- Search Filter within drawer -->
                    <div class="relative w-full">
                        <span
                            class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-(--text-tertiary) text-sm select-none"
                        >search</span>
                        <input
                            v-model="searchQuery"
                            type="text"
                            :placeholder="activeTab === 'contents' ? 'Filter chapters...' : activeTab === 'bookmarks' ? 'Filter bookmarks...' : 'Search highlights...'"
                            class="w-full bg-(--bg-card) border border-(--border-color) text-(--text-primary) text-xs rounded-full pl-8 pr-7 py-1.5 focus-ring-minimal focus:outline-none focus:border-(--border-color-hover) transition-all placeholder:text-(--text-tertiary)"
                        />
                        <button
                            v-if="searchQuery"
                            @click="searchQuery = ''"
                            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-(--text-tertiary) hover:text-(--text-primary) cursor-pointer"
                        >
                            <span class="material-symbols-outlined text-xs select-none">close</span>
                        </button>
                    </div>
                </header>

                <!-- Body Content -->
                <div
                    class="flex-1 overflow-y-auto px-4 py-3"
                    :style="{ paddingBottom: 'calc(1.5rem + env(safe-area-inset-bottom, 0px))' }"
                >
                    <!-- 1. Table of Contents / Book Structure -->
                    <div v-if="activeTab === 'contents'">
                        <ReaderTocList
                            v-if="filteredToc.length"
                            :items="filteredToc"
                            @select="onSelectToc"
                        />
                        <div
                            v-else
                            class="py-12 text-center select-none"
                        >
                            <span class="material-symbols-outlined text-3xl text-(--text-tertiary) mb-1 select-none">menu_book</span>
                            <p class="text-xs text-(--text-secondary)">No chapters found.</p>
                        </div>
                    </div>

                    <!-- 2. Bookmarks -->
                    <div v-else-if="activeTab === 'bookmarks'">
                        <div
                            v-if="filteredBookmarks.length"
                            class="flex flex-col gap-2.5"
                        >
                            <div
                                v-for="bookmark in filteredBookmarks"
                                :key="bookmark.id"
                                @click="onSelectBookmark(bookmark)"
                                class="group cursor-pointer bg-(--bg-card) border border-(--border-color) rounded-xl p-3 hover:border-(--border-color-hover) hover:bg-(--accent-color-light)/40 transition-all flex items-center justify-between gap-3 shadow-xs"
                            >
                                <div class="flex items-center gap-2.5 min-w-0 flex-1">
                                    <div class="w-8 h-8 rounded-full bg-(--accent-color-light) text-(--accent-color) flex items-center justify-center shrink-0">
                                        <span class="material-symbols-outlined text-base">bookmark</span>
                                    </div>
                                    <div class="flex flex-col gap-0.5 min-w-0 flex-1">
                                        <div class="flex items-center gap-2">
                                            <span class="text-xs font-semibold text-(--text-primary) tabular-nums">
                                                {{ progressLabel(bookmark.locator.progression) }}
                                            </span>
                                            <span class="text-[10px] text-(--text-tertiary)">
                                                {{ formatDate(bookmark.createdAt) }}
                                            </span>
                                        </div>
                                        <span class="text-[11px] text-(--text-secondary) truncate">
                                            Bookmark at {{ progressLabel(bookmark.locator.progression) }}
                                        </span>
                                    </div>
                                </div>

                                <button
                                    @click.stop="emit('delete-bookmark', bookmark.id)"
                                    class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-all cursor-pointer shrink-0 opacity-70 group-hover:opacity-100"
                                    title="Delete bookmark"
                                    aria-label="Delete bookmark"
                                >
                                    <span class="material-symbols-outlined text-base">delete</span>
                                </button>
                            </div>
                        </div>

                        <div
                            v-else
                            class="py-16 text-center select-none flex flex-col items-center gap-2"
                        >
                            <div class="w-12 h-12 rounded-full bg-(--bg-card) border border-(--border-color) flex items-center justify-center text-(--text-tertiary) mb-1">
                                <span class="material-symbols-outlined text-2xl">bookmark_border</span>
                            </div>
                            <p class="text-xs font-semibold text-(--text-primary)">No bookmarks saved</p>
                            <p class="text-[11px] text-(--text-tertiary) max-w-[200px] leading-relaxed">
                                Tap the bookmark ribbon in the reader dock to save your favorite passages.
                            </p>
                        </div>
                    </div>

                    <!-- 3. Highlights & Notes -->
                    <div v-else-if="activeTab === 'highlights'">
                        <div
                            v-if="filteredHighlights.length"
                            class="flex flex-col gap-3"
                        >
                            <div
                                v-for="highlight in filteredHighlights"
                                :key="highlight.id"
                                @click="onSelectHighlight(highlight)"
                                class="group cursor-pointer bg-(--bg-card) border border-(--border-color) rounded-xl p-3.5 hover:border-(--border-color-hover) transition-all flex flex-col gap-2 shadow-xs relative overflow-hidden"
                            >
                                <div class="absolute left-0 top-0 bottom-0 w-1 bg-(--accent-color)"></div>

                                <p class="text-xs text-(--text-primary) italic leading-relaxed line-clamp-4 pl-1 font-serif">
                                    &ldquo;{{ highlight.text }}&rdquo;
                                </p>

                                <div class="flex items-center justify-between pt-1 border-t border-(--border-color)/50 text-[10px] text-(--text-tertiary) pl-1">
                                    <span>
                                        {{ progressLabel(highlight.locator.progression) }} · {{ formatDate(highlight.createdAt) }}
                                    </span>
                                    <div class="flex items-center gap-1">
                                        <button
                                            @click.stop="copyText(highlight.text)"
                                            class="w-6 h-6 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer"
                                            title="Copy quote"
                                            aria-label="Copy quote"
                                        >
                                            <span class="material-symbols-outlined text-sm">content_copy</span>
                                        </button>
                                        <button
                                            @click.stop="emit('delete-highlight', highlight.id)"
                                            class="w-6 h-6 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/20 transition-all cursor-pointer"
                                            title="Delete highlight"
                                            aria-label="Delete highlight"
                                        >
                                            <span class="material-symbols-outlined text-sm">delete</span>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div
                            v-else
                            class="py-16 text-center select-none flex flex-col items-center gap-2"
                        >
                            <div class="w-12 h-12 rounded-full bg-(--bg-card) border border-(--border-color) flex items-center justify-center text-(--text-tertiary) mb-1">
                                <span class="material-symbols-outlined text-2xl">format_ink_highlighter</span>
                            </div>
                            <p class="text-xs font-semibold text-(--text-primary)">No highlights yet</p>
                            <p class="text-[11px] text-(--text-tertiary) max-w-[200px] leading-relaxed">
                                Select any text on a page while reading to save highlights and look up definitions.
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </template>
    </Drawer>
</template>
