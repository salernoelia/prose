<script
    setup
    lang="ts"
>
import { ref, onMounted, computed, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import DataView from "primevue/dataview";
import Dialog from "primevue/dialog";
import Select from "primevue/select";
import { useLibrary } from "../composables/useLibrary";
import { useSync } from "../composables/useSync";
import { usePullToSync } from "../composables/usePullToSync";
import type { BookDto } from "../ipc/types";

const emit = defineEmits<{
    (e: "select-book", book: BookDto): void;
}>();

const {
    entries,
    query,
    loaded,
    updateLibraryQuery,
    importBook,
    removeBook,
} = useLibrary();

const {
    configured,
    syncing,
    hasSyncError,
    triggerSync,
    refreshSyncConfig,
} = useSync();

// Filter mode persists locally so the library reopens with the same view. This
// is a device preference, not synced.
const FILTER_MODE_KEY = "prose.library.filterMode";
const loadFilterMode = (): "all" | "reading" | "read" => {
    if (typeof localStorage === "undefined") return "all";
    const saved = localStorage.getItem(FILTER_MODE_KEY);
    return saved === "reading" || saved === "read" ? saved : "all";
};
const filterMode = ref<"all" | "reading" | "read">(loadFilterMode());
watch(filterMode, (mode) => {
    if (typeof localStorage !== "undefined") {
        localStorage.setItem(FILTER_MODE_KEY, mode);
    }
});

// Pull-to-sync: drag the library down from the top to start a sync. The gesture
// runs on the scrolling <main> ancestor, the only scroll container on the page.
const rootEl = ref<HTMLElement | null>(null);
// Track whether the running sync came from the pull gesture, so the floating
// indicator stays only for those; auto-syncs use the progress banner alone.
const pullInitiated = ref(false);
const { pull, dragging, threshold } = usePullToSync(
    () => rootEl.value?.closest("main") ?? null,
    {
        enabled: () => configured.value && !syncing.value,
        onTrigger: () => {
            pullInitiated.value = true;
            triggerSync();
        },
    }
);
watch(syncing, (active) => {
    if (!active) pullInitiated.value = false;
});

const pullActive = computed(() => syncing.value && pullInitiated.value);
const pullVisible = computed(() => pull.value > 0 || pullActive.value);
const pullProgress = computed(() => Math.min(pull.value / threshold, 1));
const pullIndicatorStyle = computed(() => {
    const travel = pullActive.value ? threshold : pull.value;
    return {
        transform: `translateX(-50%) translateY(calc(${travel - 44}px + env(safe-area-inset-top, 0px)))`,
        opacity: pullActive.value ? 1 : pullProgress.value,
    };
});

const dataViewEntries = computed(() => {
    let list = [...entries.value];
    if (filterMode.value === "reading") {
        list = list.filter((entry) => entry.progress > 0 && entry.progress < 1);
    } else if (filterMode.value === "read") {
        list = list.filter((entry) => entry.progress >= 1);
    }
    return list;
});

const layout = defineModel<"grid" | "list">("layout", { default: "grid" });
const showDeleteDialog = ref(false);
const bookToDelete = ref<BookDto | null>(null);
const appDataPath = ref("");

onMounted(async () => {
    try {
        appDataPath.value = await appDataDir();
    } catch (err) {
        console.error("Failed to get app data directory:", err);
    }
    try {
        await refreshSyncConfig();
    } catch (err) {
        console.error("Failed to refresh sync status:", err);
    }
});

// Return absolute cover source URL if cover exists
const getCoverUrl = (coverPath: string | null) => {
    if (!coverPath || !appDataPath.value) return "";
    const absolutePath = `${appDataPath.value}/${coverPath}`.replace(/\/+/g, "/");
    return convertFileSrc(absolutePath);
};

// Trigger native file picker and import selected book
const handleImport = async () => {
    try {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [
                {
                    name: "Books",
                    extensions: ["epub", "pdf"],
                },
            ],
        });

        if (selected && typeof selected === "string") {
            await importBook(selected);
        }
    } catch (err) {
        console.error("Failed to pick or import file:", err);
    }
};

const triggerDelete = (book: BookDto, event: Event) => {
    event.stopPropagation();
    bookToDelete.value = book;
    showDeleteDialog.value = true;
};

const confirmDelete = async () => {
    if (!bookToDelete.value) return;
    try {
        await removeBook(bookToDelete.value.id);
        showDeleteDialog.value = false;
        bookToDelete.value = null;
    } catch (err) {
        console.error("Failed to delete book:", err);
    }
};

// Sort key mapped to string representation in query DTO
const handleSortChange = (
    key: "title" | "author" | "last_read" | "progress"
) => {
    updateLibraryQuery({
        sort: key,
        descending: key === 'progress' || key === 'last_read',
    });
};

const filterOptions = [
    { label: "All Books", value: "all" },
    { label: "In Progress", value: "reading" },
    { label: "Read", value: "read" }
];

const sortOptions = [
    { label: "Progress", value: "progress" },
    { label: "Title", value: "title" },
    { label: "Author", value: "author" },
    { label: "Recent", value: "last_read" }
];

const getFilterIcon = (mode: string) => {
    switch (mode) {
        case "reading": return "menu_book";
        case "read": return "task_alt";
        default: return "all_inclusive";
    }
};

const getSortIcon = (sort: string) => {
    switch (sort) {
        case "progress": return "percent";
        case "title": return "sort_by_alpha";
        case "author": return "person";
        default: return "history";
    }
};

const getSyncButtonText = computed(() => {
    if (syncing.value) return "Syncing";
    if (hasSyncError.value) return "Sync error";
    return "Sync";
});
</script>

<template>
    <div
        ref="rootEl"
        class="w-full animate-fade-in font-serif"
    >
        <!-- Pull-to-sync indicator -->
        <div
            v-show="pullVisible"
            class="pointer-events-none fixed top-0 left-1/2 z-40"
            :class="dragging ? '' : 'transition-all duration-300 ease-out'"
            :style="pullIndicatorStyle"
        >
            <div
                class="w-9 h-9 rounded-full bg-(--bg-card) border border-(--border-color) shadow-md flex items-center justify-center">
                <span
                    class="material-symbols-outlined text-lg text-(--text-secondary) select-none"
                    :class="{ 'animate-spin': pullActive }"
                    :style="pullActive ? undefined : { transform: `rotate(${pullProgress * 180}deg)` }"
                >sync</span>
            </div>
        </div>

        <!-- Header -->
        <header class="pb-6 flex justify-between items-center">
            <h1 class="text-xl lg:text-3xl font-semibold tracking-tight text-(--text-primary)">
                Library
            </h1>

            <div class="flex items-center gap-3">
                <!-- Sync Button -->
                <button
                    v-if="configured"
                    @click="triggerSync"
                    :disabled="syncing"
                    class="px-4 py-1.5 text-xs font-semibold rounded border text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                    :class="hasSyncError
                        ? 'border-red-500 text-red-500 dark:border-red-500 dark:text-red-400'
                        : 'border-(--border-color)'"
                >
                    <span
                        class="material-symbols-outlined text-base select-none"
                        :class="{ 'animate-spin': syncing, 'text-red-500 dark:text-red-400': hasSyncError && !syncing }"
                    >sync</span>
                    <span>{{ getSyncButtonText }}</span>
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
                    <span>Import</span>
                </button>
            </div>
        </header>

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
                    class="w-full bg-(--bg-card) border border-(--border-color) text-(--text-primary) text-sm rounded-full pl-10 pr-4 py-2.5 focus-ring-minimal focus:outline-none focus:border-(--border-color-hover) transition-all placeholder:text-(--text-tertiary)"
                />
            </div>

            <!-- Filter & Sort Controls Row -->
            <div class="flex items-center justify-between gap-3 w-full">
                <div class="flex items-center gap-2 flex-nowrap min-w-0">
                    <!-- Layout Switcher -->
                    <div
                        class="flex items-center border border-(--border-color) bg-(--bg-card) rounded-full p-0.5 shrink-0">
                        <button
                            @click="layout = 'grid'"
                            class="flex items-center justify-center w-7 h-7 rounded-full transition-all duration-100 active:scale-90 cursor-pointer text-(--text-secondary)"
                            :class="layout === 'grid'
                                ? 'bg-(--accent-color-light) !text-(--text-primary) font-semibold'
                                : 'hover:text-(--text-primary)'
                                "
                            title="Grid Layout"
                            aria-label="Grid Layout"
                        >
                            <span class="material-symbols-outlined text-base leading-none select-none">grid_view</span>
                        </button>
                        <button
                            @click="layout = 'list'"
                            class="flex items-center justify-center w-7 h-7 rounded-full transition-all duration-100 active:scale-90 cursor-pointer text-(--text-secondary)"
                            :class="layout === 'list'
                                ? 'bg-(--accent-color-light) !text-(--text-primary) font-semibold'
                                : 'hover:text-(--text-primary)'
                                "
                            title="List Layout"
                            aria-label="List Layout"
                        >
                            <span class="material-symbols-outlined text-base leading-none select-none">view_list</span>
                        </button>
                    </div>


                    <!-- Filter Dropdown -->
                    <Select
                        v-model="filterMode"
                        :options="filterOptions"
                        optionLabel="label"
                        optionValue="value"
                        class="select-chip focus-ring-minimal"
                    >
                        <template #value="slotProps">
                            <div
                                v-if="slotProps.value"
                                class="flex items-center gap-1.5 text-xs font-semibold text-(--text-primary) min-w-0"
                            >
                                <span
                                    class="material-symbols-outlined text-sm leading-none text-(--text-secondary) select-none shrink-0"
                                >
                                    {{ getFilterIcon(slotProps.value) }}
                                </span>
                                <span class="truncate">{{filterOptions.find(o => o.value === slotProps.value)?.label}}</span>
                            </div>
                        </template>
                        <template #option="slotProps">
                            <div class="flex items-center gap-1.5 text-xs font-semibold text-(--text-primary)">
                                <span
                                    class="material-symbols-outlined text-sm leading-none text-(--text-secondary) select-none"
                                >
                                    {{ getFilterIcon(slotProps.option.value) }}
                                </span>
                                <span>{{ slotProps.option.label }}</span>
                            </div>
                        </template>
                    </Select>

                    <!-- Sort Dropdown -->
                    <Select
                        :modelValue="query.sort"
                        @update:modelValue="(val) => handleSortChange(val as any)"
                        :options="sortOptions"
                        optionLabel="label"
                        optionValue="value"
                        class="select-chip focus-ring-minimal"
                    >
                        <template #value="slotProps">
                            <div
                                v-if="slotProps.value"
                                class="flex items-center gap-1.5 text-xs font-semibold text-(--text-primary) min-w-0"
                            >
                                <span
                                    class="material-symbols-outlined text-sm leading-none text-(--text-secondary) select-none shrink-0"
                                >
                                    {{ getSortIcon(slotProps.value) }}
                                </span>
                                <span class="truncate">{{sortOptions.find(o => o.value === slotProps.value)?.label}}</span>
                            </div>
                        </template>
                        <template #option="slotProps">
                            <div class="flex items-center gap-1.5 text-xs font-semibold text-(--text-primary)">
                                <span
                                    class="material-symbols-outlined text-sm leading-none text-(--text-secondary) select-none"
                                >
                                    {{ getSortIcon(slotProps.option.value) }}
                                </span>
                                <span>{{ slotProps.option.label }}</span>
                            </div>
                        </template>
                    </Select>

                    <!-- Sort Direction Button -->
                    <button
                        @click="updateLibraryQuery({ descending: !query.descending })"
                        class="flex items-center justify-center w-8 h-8 rounded-full border border-(--border-color) bg-(--bg-card) text-(--text-secondary) hover:text-(--text-primary) transition-all cursor-pointer focus-ring-minimal active:scale-90 shrink-0"
                        title="Toggle Sort Direction"
                    >
                        <span class="material-symbols-outlined text-sm leading-none select-none">
                            {{ query.descending ? "arrow_downward" : "arrow_upward" }}
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
                        class="group cursor-pointer py-3.5 px-4 rounded-xl hover:bg-(--accent-color-light) transition-all flex flex-col gap-2.5"
                    >
                        <div class="flex justify-between items-start gap-4">
                            <h2
                                class="text-base font-medium tracking-tight text-(--text-primary) group-hover:translate-x-0.5 transition-transform duration-200">
                                {{ entry.book.title }}
                            </h2>
                            <!-- Circular Trash Button (top-right) -->
                            <button
                                @click="(e) => triggerDelete(entry.book, e)"
                                class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-950/20 transition-all cursor-pointer shrink-0"
                                title="Remove Book"
                                aria-label="Remove Book"
                            >
                                <span class="material-symbols-outlined text-base select-none">delete</span>
                            </button>
                        </div>
                        <div class="flex justify-between items-center text-xs">
                            <span class="text-(--text-secondary)">{{
                                entry.book.author || "Unknown Author"
                            }}</span>

                            <span
                                class="text-xs font-semibold tabular-nums px-2 py-0.5 rounded shrink-0 transition-colors"
                                :class="entry.progress >= 1
                                    ? 'bg-emerald-100 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-400'
                                    : 'bg-(--accent-color-light) text-(--text-secondary)'
                                    "
                            >
                                {{ Math.round(entry.progress * 100) }}%
                            </span>
                        </div>

                        <!-- Full-width progress line at bottom of row -->
                        <div
                            class="w-full h-1 bg-(--border-color) rounded-full overflow-hidden mt-1 opacity-60 group-hover:opacity-100 transition-opacity">
                            <div
                                class="h-full transition-all duration-300"
                                :class="entry.progress >= 1
                                    ? 'bg-emerald-700 dark:bg-emerald-600'
                                    : 'bg-(--text-primary)'
                                    "
                                :style="{ width: entry.progress * 100 + '%' }"
                            ></div>
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
                            <!-- Floating Trash Button on Card Cover -->
                            <button
                                @click="(e) => triggerDelete(entry.book, e)"
                                class="absolute top-2 right-2 w-7 h-7 rounded-full bg-(--bg-card)/90 backdrop-blur border border-(--border-color) flex items-center justify-center text-(--text-tertiary) hover:text-red-500 shadow-sm active:scale-90 transition-all duration-200 cursor-pointer z-10"
                                title="Remove Book"
                                aria-label="Remove Book"
                            >
                                <span class="material-symbols-outlined text-base select-none">delete</span>
                            </button>

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
                                    {{ entry.book.author || "Unknown Author" }}
                                </span>
                            </div>

                            <!-- hover progress bar line at bottom of cover -->
                            <div class="absolute bottom-0 left-0 w-full h-1 bg-(--border-color)">
                                <div
                                    class="h-full transition-all duration-300"
                                    :class="entry.progress >= 1
                                        ? 'bg-emerald-700 dark:bg-emerald-600'
                                        : 'bg-(--text-primary)'
                                        "
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
                                    {{ entry.book.author || "Unknown Author" }}
                                </span>
                                <span
                                    class="tabular-nums transition-colors"
                                    :class="entry.progress >= 1
                                        ? 'text-emerald-700 dark:text-emerald-400 font-semibold'
                                        : 'text-(--text-tertiary)'
                                        "
                                >
                                    {{ Math.round(entry.progress * 100) }}%
                                </span>
                            </div>
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
                root: {
                    class: 'p-2 shadow-xl bg-[var(--bg-card)] text-[var(--text-primary)]',
                },
                closeButton: {
                    class:
                        'text-[var(--text-tertiary)] hover:text-[var(--text-primary)] border-0 cursor-pointer focus:outline-none',
                },
            }"
        >
            <p>
                Are you sure you want to remove
                <span class="font-semibold text-(--text-primary)">"{{ bookToDelete?.title }}"</span>
                from your library?
            </p>
            <p class="mt-2 text-xs text-(--text-tertiary)">
                This will delete the book record and its locally stored file and cover
                thumbnail.
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

<style scoped>
    .select-chip {
        padding: 0.15rem 0.6rem !important;
        height: 2rem;
        display: inline-flex;
        align-items: center;
        border: 1px solid var(--border-color) !important;
        background-color: var(--bg-card) !important;
        border-radius: 9999px !important;
        box-shadow: none !important;
        transition: all 0.2s ease;
        cursor: pointer;
        min-width: 0;
    }

    .select-chip:hover {
        border-color: var(--border-color-hover) !important;
    }

    :deep(.p-select-label) {
        padding: 0 !important;
        margin-right: 0.25rem !important;
        display: flex;
        align-items: center;
        min-width: 0;
        overflow: hidden;
    }

    :deep(.p-select-dropdown) {
        width: auto !important;
        height: auto !important;
        color: var(--text-secondary) !important;
    }

    :deep(.p-select-dropdown-icon) {
        font-size: 0.75rem !important;
    }
</style>
