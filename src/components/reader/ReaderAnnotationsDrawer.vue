<script
    setup
    lang="ts"
>
import { ref } from 'vue'
import Drawer from 'primevue/drawer'
import type { BookmarkDto, HighlightDto } from '../../ipc/types'

defineProps<{
    visible: boolean
    bookmarks: BookmarkDto[]
    highlights: HighlightDto[]
}>()

const emit = defineEmits<{
    (e: 'update:visible', value: boolean): void
    (e: 'select-bookmark', bookmark: BookmarkDto): void
    (e: 'delete-bookmark', id: string): void
    (e: 'select-highlight', highlight: HighlightDto): void
    (e: 'delete-highlight', id: string): void
}>()

const tab = ref<'bookmarks' | 'highlights'>('bookmarks')

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
</script>

<template>
    <Drawer
        :visible="visible"
        @update:visible="emit('update:visible', $event)"
        position="right"
        :modal="true"
        :show-close-icon="false"
        class="!w-80 !max-w-[85vw] !bg-(--bg-app) !border-l !border-(--border-color)"
    >
        <template #container="{ closeCallback }">
            <div class="flex flex-col h-full">
                <header
                    class="flex items-center justify-between px-4 pb-3 border-b border-(--border-color)"
                    :style="{ paddingTop: 'calc(0.75rem + env(safe-area-inset-top, 0px))' }"
                >
                    <span class="text-sm font-semibold tracking-wide text-(--text-primary) select-none">
                        Annotations
                    </span>
                    <button
                        @click="closeCallback"
                        class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                        title="Close"
                        aria-label="Close"
                    >
                        <span class="material-symbols-outlined text-xl leading-none select-none">close</span>
                    </button>
                </header>

                <!-- Tab switcher -->
                <div class="flex border-b border-(--border-color) select-none">
                    <button
                        @click="tab = 'bookmarks'"
                        class="flex-1 py-2 text-sm transition-colors focus-ring-minimal"
                        :class="tab === 'bookmarks'
                            ? 'text-(--text-primary) border-b-2 border-(--accent-color) font-medium'
                            : 'text-(--text-tertiary) hover:text-(--text-secondary)'"
                    >
                        Bookmarks
                    </button>
                    <button
                        @click="tab = 'highlights'"
                        class="flex-1 py-2 text-sm transition-colors focus-ring-minimal"
                        :class="tab === 'highlights'
                            ? 'text-(--text-primary) border-b-2 border-(--accent-color) font-medium'
                            : 'text-(--text-tertiary) hover:text-(--text-secondary)'"
                    >
                        Highlights
                    </button>
                </div>

                <div
                    class="flex-1 overflow-y-auto px-2 pt-2"
                    :style="{ paddingBottom: 'calc(0.5rem + env(safe-area-inset-bottom, 0px))' }"
                >
                    <!-- Bookmarks -->
                    <ul
                        v-if="tab === 'bookmarks'"
                        class="flex flex-col"
                    >
                        <li
                            v-for="bookmark in bookmarks"
                            :key="bookmark.id"
                            class="group flex items-center gap-2 rounded-md hover:bg-(--accent-color-light) transition-colors"
                        >
                            <button
                                @click="emit('select-bookmark', bookmark)"
                                class="flex-1 flex items-center gap-2 text-left px-3 py-2 text-sm text-(--text-secondary) hover:text-(--text-primary) focus-ring-minimal truncate"
                            >
                                <span class="material-symbols-outlined text-base text-(--accent-color)">bookmark</span>
                                <span class="truncate">{{ progressLabel(bookmark.locator.progression) }}</span>
                                <span class="ml-auto text-xs text-(--text-tertiary)">{{ formatDate(bookmark.createdAt) }}</span>
                            </button>
                            <button
                                @click="emit('delete-bookmark', bookmark.id)"
                                class="flex items-center justify-center w-7 h-7 mr-1 rounded-full text-(--text-tertiary) hover:text-(--danger-color,#dc2626) transition-colors focus-ring-minimal"
                                title="Delete bookmark"
                                aria-label="Delete bookmark"
                            >
                                <span class="material-symbols-outlined text-base">delete</span>
                            </button>
                        </li>
                        <p
                            v-if="!bookmarks.length"
                            class="px-3 py-4 text-sm text-(--text-tertiary) select-none"
                        >
                            No bookmarks yet.
                        </p>
                    </ul>

                    <!-- Highlights -->
                    <ul
                        v-else
                        class="flex flex-col"
                    >
                        <li
                            v-for="highlight in highlights"
                            :key="highlight.id"
                            class="group flex items-start gap-2 rounded-md hover:bg-(--accent-color-light) transition-colors"
                        >
                            <button
                                @click="emit('select-highlight', highlight)"
                                class="flex-1 text-left px-3 py-2 text-sm text-(--text-secondary) hover:text-(--text-primary) focus-ring-minimal"
                            >
                                <span class="line-clamp-3">{{ highlight.text }}</span>
                                <span class="block mt-1 text-xs text-(--text-tertiary)">
                                    {{ progressLabel(highlight.locator.progression) }} · {{ formatDate(highlight.createdAt) }}
                                </span>
                            </button>
                            <button
                                @click="emit('delete-highlight', highlight.id)"
                                class="flex items-center justify-center w-7 h-7 mr-1 mt-1 rounded-full text-(--text-tertiary) hover:text-(--danger-color,#dc2626) transition-colors focus-ring-minimal"
                                title="Delete highlight"
                                aria-label="Delete highlight"
                            >
                                <span class="material-symbols-outlined text-base">delete</span>
                            </button>
                        </li>
                        <p
                            v-if="!highlights.length"
                            class="px-3 py-4 text-sm text-(--text-tertiary) select-none"
                        >
                            No highlights yet.
                        </p>
                    </ul>
                </div>
            </div>
        </template>
    </Drawer>
</template>
