<script
    setup
    lang="ts"
>
import { computed } from "vue";

const props = withDefaults(
    defineProps<{
        configured?: boolean;
        syncing?: boolean;
        hasSyncError?: boolean;
    }>(),
    {
        configured: false,
        syncing: false,
        hasSyncError: false,
    }
);

const emit = defineEmits<{
    (e: "sync"): void;
    (e: "import"): void;
}>();

const getSyncButtonText = computed(() => {
    if (props.syncing) return "Syncing";
    if (props.hasSyncError) return "Sync error";
    return "Sync";
});
</script>

<template>
    <header
        class="pt-1 pb-2 mb-4 flex flex-wrap items-end justify-between gap-3"
    >
        <div>

            <h1 class="text-2xl lg:text-4xl font-semibold tracking-tight text-(--text-primary) font-serif">
                Library
            </h1>
        </div>

        <div class="flex items-center gap-2">
            <button
                v-if="configured"
                @click="emit('sync')"
                :disabled="syncing"
                class="px-3.5 py-1.5 text-xs font-medium rounded-full border text-(--text-primary) bg-(--bg-card) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-1.5 disabled:opacity-50 disabled:cursor-not-allowed shadow-xs"
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
                class="px-3.5 py-1.5 text-xs font-medium rounded-full border border-(--border-color) text-(--text-tertiary) opacity-40 cursor-not-allowed flex items-center gap-1.5"
            >
                <span class="material-symbols-outlined text-base select-none">sync</span>
                <span>Sync</span>
            </button>

            <button
                @click="emit('import')"
                class="px-3.5 py-1.5 text-xs font-medium rounded-full border border-(--border-color) bg-(--bg-card) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-1.5 shadow-xs"
            >
                <span class="material-symbols-outlined text-base select-none">add</span>
                <span>Import</span>
            </button>
        </div>
    </header>
</template>
