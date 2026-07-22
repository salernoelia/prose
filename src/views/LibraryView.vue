<script setup lang="ts">
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
type SortKey = "title" | "author" | "last_read" | "progress";

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
        console.error("Failed to pick or import file:", err);
    }
};

const bookMenu = ref<InstanceType<typeof Menu> | null>(null);
const menuEntry = ref<LibraryEntryDto | null>(null);

const openMenu = (entry: LibraryEntryDto, event: Event) => {
    event.stopPropagation();
    menuEntry.value = entry;
    bookMenu.value?.toggle(event);
};

const menuItems = computed(() => {
    const archived = menuEntry.value?.archived ?? false;
    return [
        {
            label: archived ? "Unarchive" : "Archive",
            icon: archived ? "unarchive" : "archive",
            command: () => toggleArchive(),
        },
        {
            label: "Remove",
            icon: "delete",
            danger: true,
            command: () => {
                if (menuEntry.value) triggerDelete(menuEntry.value.book);
            },
        },
    ];
});

const toggleArchive = async () => {
    if (!menuEntry.value) return;
    try {
        await setBookArchived(menuEntry.value.book.id, !menuEntry.value.archived);
    } catch (err) {
        console.error("Failed to archive book:", err);
    }
};

const triggerDelete = (book: BookDto) => {
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

const handleSortChange = (key: SortKey) => {
    updateLibraryQuery({
        sort: key,
        descending: key === "progress" || key === "last_read",
    });
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
            @update:layout="(val) => layout = val"
            @update:filterMode="(val) => filterMode = val"
            @update:sortKey="handleSortChange"
            @toggle-direction="updateLibraryQuery({ descending: !query.descending })"
        />

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
            <template #list="slotProps">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-x-8 gap-y-4">
                    <BookListItem
                        v-for="entry in slotProps.items"
                        :key="entry.book.id"
                        :entry="entry"
                        @select="emit('select-book', entry.book)"
                        @open-menu="(e) => openMenu(entry, e)"
                    />
                </div>
            </template>

            <template #grid="slotProps">
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-6">
                    <BookGridCard
                        v-for="entry in slotProps.items"
                        :key="entry.book.id"
                        :entry="entry"
                        :appDataPath="appDataPath"
                        @select="emit('select-book', entry.book)"
                        @open-menu="(e) => openMenu(entry, e)"
                    />
                </div>
            </template>

            <template #empty>
                <div class="py-12 text-left">
                    <p class="text-base text-(--text-secondary) leading-relaxed">
                        No books found in the library. Click "Import Book" to add one.
                    </p>
                </div>
            </template>
        </DataView>

        <Menu
            ref="bookMenu"
            :model="menuItems"
            popup
            class="font-serif"
            :pt="{
                root: { class: '!bg-(--bg-card) !border-(--border-color) rounded-lg shadow-lg text-sm min-w-40' },
            }"
        >
            <template #item="{ item, props }">
                <a
                    v-bind="props.action"
                    class="flex items-center gap-2 px-3 py-2 cursor-pointer transition-colors"
                    :class="item.danger
                        ? 'text-red-600 dark:text-red-400 hover:!bg-red-50 dark:hover:!bg-red-950/20'
                        : 'text-(--text-primary) hover:!bg-(--accent-color-light)'"
                >
                    <span class="material-symbols-outlined text-base leading-none select-none">{{ item.icon }}</span>
                    <span class="font-medium">{{ item.label }}</span>
                </a>
            </template>
        </Menu>

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
