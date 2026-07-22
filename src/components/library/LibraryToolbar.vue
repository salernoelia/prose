<script setup lang="ts">
import Select from "primevue/select";

type SortKey = "title" | "author" | "last_read" | "progress";
type FilterMode = "all" | "reading" | "read" | "archived";

defineProps<{
    search: string;
    layout: "grid" | "list";
    filterMode: FilterMode;
    sortKey: SortKey;
    descending: boolean;
}>();

const emit = defineEmits<{
    (e: "update:search", value: string | null): void;
    (e: "update:layout", value: "grid" | "list"): void;
    (e: "update:filterMode", value: FilterMode): void;
    (e: "update:sortKey", value: SortKey): void;
    (e: "toggle-direction"): void;
}>();

const filterOptions = [
    { label: "All Books", value: "all" },
    { label: "In Progress", value: "reading" },
    { label: "Read", value: "read" },
    { label: "Archived", value: "archived" },
];

const sortOptions = [
    { label: "Progress", value: "progress" },
    { label: "Title", value: "title" },
    { label: "Author", value: "author" },
    { label: "Recent", value: "last_read" },
];

const getFilterIcon = (mode: string) => {
    switch (mode) {
        case "reading": return "menu_book";
        case "read": return "task_alt";
        case "archived": return "inventory_2";
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
</script>

<template>
    <div class="flex flex-col gap-4 mb-8">
        <div class="relative w-full">
            <span
                class="material-symbols-outlined absolute left-3.5 top-1/2 -translate-y-1/2 text-(--text-tertiary) text-lg select-none"
            >search</span>
            <input
                :value="search"
                @input="(e) => emit('update:search', (e.target as HTMLInputElement).value || null)"
                type="text"
                placeholder="Search by title or author..."
                class="w-full bg-(--bg-card) border border-(--border-color) text-(--text-primary) text-sm rounded-full pl-10 pr-4 py-2.5 focus-ring-minimal focus:outline-none focus:border-(--border-color-hover) transition-all placeholder:text-(--text-tertiary)"
            />
        </div>

        <div class="flex items-center justify-between gap-3 w-full">
            <div class="flex items-center gap-2 flex-nowrap min-w-0">
                <div class="flex items-center border border-(--border-color) bg-(--bg-card) rounded-full p-0.5 shrink-0">
                    <button
                        @click="emit('update:layout', 'grid')"
                        class="flex items-center justify-center w-7 h-7 rounded-full transition-all duration-100 active:scale-90 cursor-pointer text-(--text-secondary)"
                        :class="layout === 'grid'
                            ? 'bg-(--accent-color-light) !text-(--text-primary) font-semibold'
                            : 'hover:text-(--text-primary)'"
                        title="Grid Layout"
                        aria-label="Grid Layout"
                    >
                        <span class="material-symbols-outlined text-base leading-none select-none">grid_view</span>
                    </button>
                    <button
                        @click="emit('update:layout', 'list')"
                        class="flex items-center justify-center w-7 h-7 rounded-full transition-all duration-100 active:scale-90 cursor-pointer text-(--text-secondary)"
                        :class="layout === 'list'
                            ? 'bg-(--accent-color-light) !text-(--text-primary) font-semibold'
                            : 'hover:text-(--text-primary)'"
                        title="List Layout"
                        aria-label="List Layout"
                    >
                        <span class="material-symbols-outlined text-base leading-none select-none">view_list</span>
                    </button>
                </div>

                <Select
                    :modelValue="filterMode"
                    @update:modelValue="(val) => emit('update:filterMode', val as FilterMode)"
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
                            <span class="truncate">{{ filterOptions.find(o => o.value === slotProps.value)?.label }}</span>
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

                <Select
                    :modelValue="sortKey"
                    @update:modelValue="(val) => emit('update:sortKey', val as SortKey)"
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
                            <span class="truncate">{{ sortOptions.find(o => o.value === slotProps.value)?.label }}</span>
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

                <button
                    @click="emit('toggle-direction')"
                    class="flex items-center justify-center w-8 h-8 rounded-full border border-(--border-color) bg-(--bg-card) text-(--text-secondary) hover:text-(--text-primary) transition-all cursor-pointer focus-ring-minimal active:scale-90 shrink-0"
                    title="Toggle Sort Direction"
                >
                    <span class="material-symbols-outlined text-sm leading-none select-none">
                        {{ descending ? "arrow_downward" : "arrow_upward" }}
                    </span>
                </button>
            </div>
        </div>
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
