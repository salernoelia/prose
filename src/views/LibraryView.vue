<script
    setup
    lang="ts"
>
import { ref, onMounted, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { appDataDir } from '@tauri-apps/api/path'
import { convertFileSrc } from '@tauri-apps/api/core'
import DataView from 'primevue/dataview'
import Dialog from 'primevue/dialog'
import { useLibrary } from '../composables/useLibrary'
import { useSync } from '../composables/useSync'
import type { BookDto } from '../ipc/types'

const emit = defineEmits<{
    (e: 'select-book', book: BookDto): void
}>()

const {
    entries,
    query,
    loaded,
    importing,
    importMessage,
    importFraction,
    updateLibraryQuery,
    importBook,
    removeBook,
} = useLibrary()

const {
    configured,
    syncing,
    progressMessage,
    progressFraction,
    lastFinishedResult,
    triggerSync,
    refreshSyncConfig,
    dismissSyncResult,
} = useSync()

const dataViewEntries = computed(() => [...entries.value])

const layout = defineModel<'grid' | 'list'>('layout', { default: 'grid' })
const showDeleteDialog = ref(false)
const bookToDelete = ref<BookDto | null>(null)
const appDataPath = ref('')

onMounted(async () => {
    try {
        appDataPath.value = await appDataDir()
    } catch (err) {
        console.error('Failed to get app data directory:', err)
    }
    try {
        await refreshSyncConfig()
    } catch (err) {
        console.error('Failed to refresh sync status:', err)
    }
})

// Return absolute cover source URL if cover exists
const getCoverUrl = (coverPath: string | null) => {
    if (!coverPath || !appDataPath.value) return ''
    const absolutePath = `${appDataPath.value}/${coverPath}`.replace(/\/+/g, '/')
    return convertFileSrc(absolutePath)
}

// Trigger native file picker and import selected book
const handleImport = async () => {
    try {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [
                {
                    name: 'Books',
                    extensions: ['epub', 'pdf'],
                },
            ],
        })

        if (selected && typeof selected === 'string') {
            await importBook(selected)
        }
    } catch (err) {
        console.error('Failed to pick or import file:', err)
    }
}

const triggerDelete = (book: BookDto, event: Event) => {
    event.stopPropagation()
    bookToDelete.value = book
    showDeleteDialog.value = true
}

const confirmDelete = async () => {
    if (!bookToDelete.value) return
    try {
        await removeBook(bookToDelete.value.id)
        showDeleteDialog.value = false
        bookToDelete.value = null
    } catch (err) {
        console.error('Failed to delete book:', err)
    }
}

// Sort key mapped to string representation in query DTO
const handleSortChange = (key: 'title' | 'author' | 'last_read' | 'progress') => {
    const isCurrentlyActive = query.value.sort === key
    updateLibraryQuery({
        sort: key,
        descending: isCurrentlyActive ? !query.value.descending : false,
    })
}
</script>

<template>
    <div class="w-full animate-fade-in font-serif">
        <!-- Header -->
        <header class="pb-6 flex justify-between items-center">
            <h1 class="text-xl lg:text-3xl font-semibold tracking-tight text-(--text-primary)">Library</h1>

            <div class="flex items-center gap-3">
                <!-- Sync Button -->
                <button
                    v-if="configured"
                    @click="triggerSync"
                    :disabled="syncing"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    <span
                        class="material-symbols-outlined text-base select-none"
                        :class="{ 'animate-spin': syncing }"
                    >sync</span>
                    <span>{{ syncing ? 'Syncing...' : 'Sync' }}</span>
                </button>
                <button
                    v-else
                    disabled
                    title="Configure WebDAV sync in settings"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-tertiary) opacity-50 cursor-not-allowed flex items-center gap-2"
                >
                    <span class="material-symbols-outlined text-base select-none">sync</span>
                    <span>Sync</span>
                </button>

                <!-- Import Trigger (Typographic Pill Button) -->
                <button
                    @click="handleImport"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-2"
                >
                    <span class="material-symbols-outlined text-base select-none">add</span>
                    <span>Import Book</span>
                </button>
            </div>
        </header>

        <!-- Sync Progress Indicator -->
        <div
            v-if="syncing"
            class="mb-4 p-4 rounded-2xl border border-(--border-color) bg-(--accent-color-medium) text-sm text-(--text-primary)"
        >
            <div class="flex justify-between items-center mb-1">
                <span>{{ progressMessage || 'Syncing library...' }}</span>
                <span class="tabular-nums font-semibold">{{ Math.round(progressFraction * 100) }}%</span>
            </div>
            <div class="w-full h-1 bg-(--border-color) rounded overflow-hidden">
                <div
                    class="h-full bg-(--text-primary) transition-all duration-300"
                    :style="{ width: progressFraction * 100 + '%' }"
                ></div>
            </div>
        </div>

        <!-- Sync Error Alert -->
        <div
            v-if="lastFinishedResult && !lastFinishedResult.success"
            class="mt-6 mb-2 p-4 border border-red-200 dark:border-red-950/40 rounded bg-red-50 dark:bg-red-950/10 text-sm text-red-700 dark:text-red-400 flex justify-between items-center"
        >
            <span>Sync failed: {{ lastFinishedResult.message }}</span>
            <button
                @click="dismissSyncResult"
                class="text-xs font-semibold hover:underline cursor-pointer border-0 bg-transparent text-red-700 dark:text-red-400"
            >
                Dismiss
            </button>
        </div>

        <!-- Import Progress Indicator (Zero Icon) -->
        <div
            v-if="importing"
            class="mb-6 p-4 border border-(--border-color) rounded bg-(--accent-color-light) text-sm text-(--text-primary)"
        >
            <div class="flex justify-between items-center mb-1">
                <span>{{ importMessage }}</span>
                <span>{{ Math.round(importFraction * 100) }}%</span>
            </div>
            <div class="w-full h-1 bg-(--border-color) rounded overflow-hidden">
                <div
                    class="h-full bg-(--text-primary) transition-all duration-300"
                    :style="{ width: importFraction * 100 + '%' }"
                ></div>
            </div>
        </div>

        <!-- Search & Filter Bar -->
        <div class="flex flex-col gap-4 mb-8">
            <!-- Search bar with Icon -->
            <div class="relative w-full">
                <span
                    class="material-symbols-outlined absolute left-3.5 top-1/2 -translate-y-1/2 text-(--text-tertiary) text-lg select-none"
                >search</span>
                <input
                    :value="query.search || ''"
                    @input="
                        (e) => updateLibraryQuery({ search: (e.target as HTMLInputElement).value || null })
                    "
                    type="text"
                    placeholder="Search by title or author..."
                    class="w-full bg-(--bg-card) border border-(--border-color) text-(--text-primary) text-sm rounded-full pl-10 pr-4 py-2.5 focus-ring-minimal focus:outline-none focus:border-(--border-color-hover) transition-all placeholder:text-(--text-tertiary) shadow-sm"
                />
            </div>

            <!-- Filter Chips Row -->
            <div class="flex items-center justify-between gap-3 w-full">
                <!-- Sort options horizontally scrollable on mobile, flex-wrap on desktop -->
                <div class="flex items-center gap-2 overflow-x-auto no-scrollbar scroll-smooth flex-1 py-1 -my-1 -mx-6 px-6 md:mx-0 md:px-0">
                    <button
                        v-for="opt in [
                            { key: 'progress', label: 'Progress', icon: 'percent' },
                            { key: 'title', label: 'Title', icon: 'sort_by_alpha' },
                            { key: 'author', label: 'Author', icon: 'person' },
                            { key: 'last_read', label: 'Recent', icon: 'history' }
                        ] as const"
                        :key="opt.key"
                        @click="handleSortChange(opt.key)"
                        class="flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-full border transition-all duration-100 whitespace-nowrap cursor-pointer active:scale-95 select-none shrink-0"
                        :class="query.sort === opt.key
                            ? 'bg-(--text-primary) border-(--text-primary) text-(--bg-app)'
                            : 'bg-(--bg-card) border-(--border-color) text-(--text-secondary) hover:text-(--text-primary) hover:border-(--border-color-hover)'
                            "
                    >
                        <span class="material-symbols-outlined text-sm leading-none select-none">{{ opt.icon }}</span>
                        <span>{{ opt.label }}</span>
                        <span
                            v-if="query.sort === opt.key"
                            class="material-symbols-outlined text-xs leading-none ml-0.5 select-none"
                        >
                            {{ query.descending ? 'arrow_downward' : 'arrow_upward' }}
                        </span>
                    </button>
                </div>
            </div>
        </div>

        <!-- Switchable DataView Catalog -->
        <DataView
            v-if="loaded"
            :value="dataViewEntries"
            :layout="layout"
            class="w-full"
            :pt="{
                root: { class: '!bg-transparent !border-none' },
                header: { class: '!bg-transparent !border-none' },
                content: { class: '!bg-transparent !border-0 p-0' },
                footer: { class: '!bg-transparent !border-none' },
            }"
        >
            <!-- List View Mode -->
            <template #list="slotProps">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-4">
                    <div
                        v-for="entry in slotProps.items"
                        :key="entry.book.id"
                        @click="emit('select-book', entry.book)"
                        class="group cursor-pointer py-4 border-b border-(--border-color) hover:border-(--text-secondary) transition-all flex flex-col gap-2"
                    >
                        <div class="flex justify-between items-start gap-4">
                            <h2
                                class="text-base font-medium tracking-tight text-(--text-primary) group-hover:translate-x-0.5 transition-transform duration-200">
                                {{ entry.book.title }}
                            </h2>
                            <div class="flex items-center gap-3">
                                <span class="text-xs text-(--text-tertiary) tabular-nums">{{ Math.round(entry.progress *
                                    100) }}%</span>

                                <!-- Muted textual remove link -->
                                <button
                                    @click="(e) => triggerDelete(entry.book, e)"
                                    class="text-(--text-tertiary) hover:text-red-500 text-xs px-1 hover:underline cursor-pointer"
                                >
                                    Remove
                                </button>
                            </div>
                        </div>
                        <div class="flex justify-between items-center text-xs">
                            <span class="text-(--text-secondary)">{{
                                entry.book.author || 'Unknown Author'
                                }}</span>

                            <!-- Minimal progress line -->
                            <div class="w-16 h-0.5 overflow-hidden">
                                <div
                                    class="h-full bg-(--text-primary) transition-all duration-300"
                                    :style="{ width: entry.progress * 100 + '%' }"
                                ></div>
                            </div>
                        </div>
                    </div>
                </div>
            </template>

            <!-- Grid View Mode -->
            <template #grid="slotProps">
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-6">
                    <div
                        v-for="entry in slotProps.items"
                        :key="entry.book.id"
                        @click="emit('select-book', entry.book)"
                        class="group cursor-pointer flex flex-col gap-3 pb-4 border-b border-transparent hover:border-(--border-color) transition-all"
                    >
                        <!-- Typographic cover container -->
                        <div
                            class="aspect-3/4 w-full bg-(--bg-card) border border-(--border-color) rounded overflow-hidden relative shadow-sm group-hover:shadow transition-shadow flex items-center justify-center">
                            <img
                                v-if="entry.book.cover && appDataPath"
                                :src="getCoverUrl(entry.book.cover)"
                                alt="Book cover"
                                class="w-full h-full object-cover"
                            />

                            <!-- Editorial Placeholder Cover -->
                            <div
                                v-else
                                class="w-full h-full p-4 flex flex-col justify-between items-start text-left bg-(--bg-card)"
                            >
                                <span
                                    class="text-xs font-semibold uppercase tracking-wider text-(--text-tertiary) leading-none select-none"
                                >
                                    {{ entry.book.format }}
                                </span>
                                <span
                                    class="text-sm font-semibold tracking-tight text-(--text-primary) line-clamp-3 select-none"
                                >
                                    {{ entry.book.title }}
                                </span>
                                <span class="text-xs text-(--text-secondary) truncate w-full select-none">
                                    {{ entry.book.author || 'Unknown Author' }}
                                </span>
                            </div>

                            <!-- hover progress bar line at bottom of cover -->
                            <div class="absolute bottom-0 left-0 w-full h-1 bg-(--border-color)">
                                <div
                                    class="h-full bg-(--text-primary)"
                                    :style="{ width: entry.progress * 100 + '%' }"
                                ></div>
                            </div>
                        </div>

                        <!-- Meta details under card -->
                        <div class="flex flex-col gap-1 text-left">
                            <h2
                                class="text-sm font-medium tracking-tight text-(--text-primary) truncate w-full group-hover:translate-x-0.5 transition-transform duration-200">
                                {{ entry.book.title }}
                            </h2>
                            <div class="flex justify-between items-center text-xs">
                                <span class="text-(--text-secondary) truncate max-w-[70%]">
                                    {{ entry.book.author || 'Unknown Author' }}
                                </span>
                                <span class="text-(--text-tertiary) tabular-nums">{{ Math.round(entry.progress * 100)
                                    }}%</span>
                            </div>

                            <!-- Muted textual remove link -->
                            <button
                                @click="(e) => triggerDelete(entry.book, e)"
                                class="text-(--text-tertiary) hover:text-red-500 text-left text-xs self-start hover:underline mt-1 cursor-pointer"
                            >
                                Remove
                            </button>
                        </div>
                    </div>
                </div>
            </template>

            <!-- Empty Catalog State -->
            <template #empty>
                <div class="py-12 text-left">
                    <p class="text-base text-(--text-secondary) leading-relaxed">
                        No books found in the library. Click "Import Book" to add one.
                    </p>
                </div>
            </template>
        </DataView>

        <!-- Delete Confirmation Dialog (Zero Icons, Large Padding) -->
        <Dialog
            v-model:visible="showDeleteDialog"
            modal
            header="Remove Book"
            class="font-serif border border-(--border-color) bg-(--bg-card) rounded"
            :style="{ width: '28rem' }"
            :pt="{
                root: { class: 'p-2 shadow-xl bg-[var(--bg-card)] text-[var(--text-primary)]' },
                closeButton: {
                    class:
                        'text-[var(--text-tertiary)] hover:text-[var(--text-primary)] border-0 cursor-pointer focus:outline-none',
                },
            }"
        >
            <p>
                Are you sure you want to remove
                <span class="font-semibold text-(--text-primary)">"{{ bookToDelete?.title }}"</span> from
                your library?
            </p>
            <p class="mt-2 text-xs text-(--text-tertiary)">
                This will delete the book record and its locally stored file and cover thumbnail.
            </p>

            <template #footer>
                <button
                    @click="showDeleteDialog = false"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-secondary) hover:text-(--text-primary) cursor-pointer focus-ring-minimal"
                >
                    Cancel
                </button>
                <button
                    @click="confirmDelete"
                    class="px-4 py-1.5 text-xs font-semibold rounded bg-red-600 hover:bg-red-700 text-white cursor-pointer focus-ring-minimal"
                >
                    Remove
                </button>
            </template>
        </Dialog>
    </div>
</template>
