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

const layout = ref<'grid' | 'list'>('grid')
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
        <header class="pb-6 border-b border-(--border-color) flex justify-between items-center">
            <h1 class="text-xl font-semibold tracking-tight text-(--text-primary)">Library</h1>

            <div class="flex items-center gap-3">
                <!-- Sync Button -->
                <button
                    v-if="configured"
                    @click="triggerSync"
                    :disabled="syncing"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    <span v-if="syncing" class="inline-block animate-spin w-3 h-3 border border-current border-t-transparent rounded-full"></span>
                    <span>{{ syncing ? 'Syncing...' : 'Sync' }}</span>
                </button>
                <button
                    v-else
                    disabled
                    title="Configure WebDAV sync in settings"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-tertiary) opacity-50 cursor-not-allowed"
                >
                    Sync (Not Configured)
                </button>

                <!-- Import Trigger (Typographic Pill Button) -->
                <button
                    @click="handleImport"
                    class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal"
                >
                    Import Book
                </button>
            </div>
        </header>

        <!-- Sync Progress Indicator -->
        <div
            v-if="syncing"
            class="mt-6 mb-2 p-4 border border-(--border-color) rounded bg-(--accent-color-light) text-sm text-(--text-primary)"
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

        <!-- Toolbar: Search, Filters & View Switcher (Zero Icons, Typographic) -->
        <div class="flex flex-col gap-4 justify-between items-start mb-8">
            <!-- Search Input -->
            <div class="flex-1 w-full">
                <input
                    :value="query.search || ''"
                    @input="
                        (e) => updateLibraryQuery({ search: (e.target as HTMLInputElement).value || null })
                    "
                    type="text"
                    placeholder="Search by title or author"
                    class="w-full bg-(--bg-card) border border-(--border-color) text-(--text-primary) text-sm rounded px-4 py-2 focus-ring-minimal focus:outline-none transition-all placeholder:text-(--text-tertiary)"
                />
            </div>

            <!-- Filters & Toggle links -->
            <div class="flex lg:flex-row flex-col items-start gap-4 text-xs text-(--text-secondary)">
                <!-- Sort links -->
                <div class="flex items-center gap-2">
                    <span class="text-(--text-tertiary) uppercase tracking-wider font-medium">Sort:</span>
                    <div class="flex gap-1.5">
                        <button
                            v-for="opt in ['title', 'author', 'last_read', 'progress'] as const"
                            :key="opt"
                            @click="handleSortChange(opt)"
                            class="px-2 py-0.5 capitalize transition-all rounded"
                            :class="query.sort === opt
                                ? 'text-(--text-primary) font-semibold bg-(--accent-color-light)'
                                : 'hover:text-(--text-primary)'
                                "
                        >
                            {{ opt.replace('_', ' ') }}
                            <span v-if="query.sort === opt">{{ query.descending ? '↓' : '↑' }}</span>
                        </button>
                    </div>
                </div>

                <!-- Layout switcher links -->
                <div class="flex items-center gap-2">
                    <span class="text-(--text-tertiary) uppercase tracking-wider font-medium">Layout:</span>
                    <div class="flex gap-1.5">

                        <button
                            @click="layout = 'grid'"
                            class="px-2 py-0.5 transition-all rounded"
                            :class="layout === 'grid'
                                ? 'text-(--text-primary) font-semibold bg-(--accent-color-light)'
                                : 'hover:text-(--text-primary)'
                                "
                        >
                            Grid
                        </button>

                        <button
                            @click="layout = 'list'"
                            class="px-2 py-0.5 transition-all rounded"
                            :class="layout === 'list'
                                ? 'text-(--text-primary) font-semibold bg-(--accent-color-light)'
                                : 'hover:text-(--text-primary)'
                                "
                        >
                            List
                        </button>
                    </div>
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
