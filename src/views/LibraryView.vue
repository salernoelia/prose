<script
    setup
    lang="ts"
>
import { ref, onMounted, computed, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir } from "@tauri-apps/api/path";
import DataView from "primevue/dataview";
import Dialog from "primevue/dialog";
import Menu from "primevue/menu";
import { useLibrary } from "../composables/useLibrary";
import { useSync } from "../composables/useSync";
import { usePullToSync } from "../composables/usePullToSync";
import type { BookDto, LibraryEntryDto } from "../ipc/types";
import {
    LibraryHeader,
    LibraryToolbar,
    BookGridCard,
    BookListItem,
    LibraryPullToSync,
} from "../components/library";

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
    setBookArchived,
} = useLibrary();

const {
    configured,
    syncing,
    hasSyncError,
    triggerSync,
    refreshSyncConfig,
} = useSync();

type FilterMode = "all" | "reading" | "read" | "archived";

const FILTER_MODE_KEY = "prose.library.filterMode";
const loadFilterMode = (): FilterMode => {
    if (typeof localStorage === "undefined") return "all";
    const saved = localStorage.getItem(FILTER_MODE_KEY);
    return saved === "reading" || saved === "read" || saved === "archived"
        ? saved
        : "all";
};

const filterMode = ref<FilterMode>(loadFilterMode());
watch(filterMode, (mode) => {
    if (typeof localStorage !== "undefined") {
        localStorage.setItem(FILTER_MODE_KEY, mode);
    }
});

const rootEl = ref<HTMLElement | null>(null);
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
    if (filterMode.value === "archived") {
        return list.filter((entry) => entry.archived);
    }
    list = list.filter((entry) => !entry.archived);
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
        console.error("Failed to open file dialog:", err);
    }
};

const handleSelect = (book: BookDto) => {
    emit("select-book", book);
};

const menuRef = ref();
const selectedEntry = ref<LibraryEntryDto | null>(null);

const menuModel = computed(() => {
    if (!selectedEntry.value) return [];
    const isArchived = selectedEntry.value.archived;
    return [
        {
            label: isArchived ? "Unarchive Book" : "Archive Book",
            icon: isArchived ? "unarchive" : "archive",
            command: () => {
                if (selectedEntry.value) {
                    setBookArchived(
                        selectedEntry.value.book.id,
                        !selectedEntry.value.archived
                    );
                }
            },
        },
        {
            separator: true,
        },
        {
            label: "Delete from Library",
            icon: "delete",
            class: "text-red-500",
            command: () => {
                if (selectedEntry.value) {
                    bookToDelete.value = selectedEntry.value.book;
                    showDeleteDialog.value = true;
                }
            },
        },
    ];
});

const handleOpenMenu = (event: {
    originalEvent: MouseEvent;
    entry: LibraryEntryDto;
}) => {
    selectedEntry.value = event.entry;
    menuRef.value.toggle(event.originalEvent);
};

const confirmDelete = async () => {
    if (bookToDelete.value) {
        await removeBook(bookToDelete.value.id);
        bookToDelete.value = null;
        showDeleteDialog.value = false;
    }
};
</script>

<template>
    <div
        ref="rootEl"
        class="w-full animate-fade-in font-serif"
    >
        <LibraryPullToSync
            :visible="pullVisible"
            :dragging="dragging"
            :active="pullActive"
            :progress="pullProgress"
            :indicatorStyle="pullIndicatorStyle"
        />

        <LibraryHeader
            :configured="configured"
            :syncing="syncing"
            :hasSyncError="hasSyncError"
            @sync="triggerSync"
            @import="handleImport"
        />

        <LibraryToolbar
            :search="query.search || ''"
            :layout="layout"
            :filterMode="filterMode"
            :sortKey="query.sort"
            :descending="query.descending"
            @update:search="(val) => updateLibraryQuery({ search: val })"
            @update:layout="(val) => (layout = val)"
            @update:filterMode="(val) => (filterMode = val)"
            @update:sortKey="(val) => updateLibraryQuery({ sort: val })"
            @toggle-direction="() => updateLibraryQuery({ descending: !query.descending })"
        />

        <div
            v-if="!loaded"
            class="grid grid-cols-2 gap-4 py-8 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5"
        >
            <div
                v-for="i in 10"
                :key="i"
                class="flex flex-col gap-2.5 animate-pulse"
            >
                <div class="w-full aspect-[2/3] rounded-2xl bg-(--border-color)/50"></div>
                <div class="h-3 w-3/4 bg-(--border-color)/40 rounded-full"></div>
                <div class="h-2.5 w-1/2 bg-(--border-color)/30 rounded-full"></div>
            </div>
        </div>

        <div
            v-else-if="entries.length === 0"
            class="flex flex-col items-center justify-center py-20 text-center"
        >
            <span class="material-symbols-outlined mb-3 text-5xl text-(--text-tertiary) select-none">
                local_library
            </span>
            <p class="text-base font-medium text-(--text-primary)">
                Your library is empty
            </p>
            <p class="mt-1 text-xs text-(--text-secondary) max-w-xs">
                Import EPUB or PDF files to start reading and tracking your progress.
            </p>
            <button
                @click="handleImport"
                class="mt-4 px-4 py-2 text-xs font-medium rounded-full bg-(--text-primary) text-(--bg-app) hover:opacity-90 transition-opacity cursor-pointer inline-flex items-center gap-1.5 shadow-xs"
            >
                <span class="material-symbols-outlined text-base leading-none select-none">add</span>
                <span>Import your first book</span>
            </button>
        </div>

        <div
            v-else-if="dataViewEntries.length === 0"
            class="flex flex-col items-center justify-center py-16 text-center"
        >
            <span class="material-symbols-outlined mb-3 text-4xl text-(--text-tertiary) select-none">
                search_off
            </span>
            <p class="text-sm font-medium text-(--text-primary)">
                No books match your criteria
            </p>
            <p class="mt-1 text-xs text-(--text-secondary)">
                Try adjusting your search query or active filter.
            </p>
        </div>

        <DataView
            v-else
            :value="dataViewEntries"
            :layout="layout"
            dataKey="book.id"
            class="w-full"
        >
            <template #grid="slotProps">
                <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:gap-6">
                    <BookGridCard
                        v-for="entry in slotProps.items"
                        :key="entry.book.id"
                        :entry="entry"
                        :appDataPath="appDataPath"
                        @select="handleSelect(entry.book)"
                        @openMenu="handleOpenMenu"
                    />
                </div>
            </template>

            <template #list="slotProps">
                <div class="flex flex-col gap-2">
                    <BookListItem
                        v-for="entry in slotProps.items"
                        :key="entry.book.id"
                        :entry="entry"
                        :appDataPath="appDataPath"
                        @select="handleSelect(entry.book)"
                        @openMenu="handleOpenMenu"
                    />
                </div>
            </template>
        </DataView>

        <Menu
            ref="menuRef"
            :model="menuModel"
            :popup="true"
            class="border border-(--border-color) bg-(--bg-card) shadow-lg rounded-2xl p-1 font-serif text-xs"
        >
            <template #item="{ item }">
                <button
                    class="flex items-center gap-2 w-full px-3 py-2 text-left rounded-xl transition-colors cursor-pointer hover:bg-(--accent-color-light)"
                    :class="item.class || 'text-(--text-primary)'"
                >
                    <span class="material-symbols-outlined text-base select-none">{{ item.icon }}</span>
                    <span>{{ item.label }}</span>
                </button>
            </template>
        </Menu>

        <Dialog
            v-model:visible="showDeleteDialog"
            modal
            header="Delete Book"
            :style="{ width: '90vw', maxWidth: '380px' }"
            class="border border-(--border-color) bg-(--bg-card) rounded-2xl shadow-xl font-serif"
        >
            <div class="flex flex-col gap-4 p-1">
                <p class="text-xs text-(--text-secondary) leading-relaxed">
                    Are you sure you want to remove
                    <strong class="text-(--text-primary)">{{ bookToDelete?.title }}</strong>
                    from your library? This will delete local reading data and bookmarks.
                </p>
                <div class="flex items-center justify-end gap-2 pt-2">
                    <button
                        @click="showDeleteDialog = false"
                        class="px-3.5 py-1.5 text-xs font-medium rounded-full border border-(--border-color) text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer"
                    >
                        Cancel
                    </button>
                    <button
                        @click="confirmDelete"
                        class="px-3.5 py-1.5 text-xs font-medium rounded-full bg-red-600 text-white hover:bg-red-700 transition-all cursor-pointer shadow-xs"
                    >
                        Delete
                    </button>
                </div>
            </div>
        </Dialog>

        <!-- Invisible bottom spacer so scrolling comfortably clears floating bottom nav -->
        <div class="h-28 sm:h-36 w-full shrink-0 pointer-events-none" aria-hidden="true"></div>
    </div>
</template>
