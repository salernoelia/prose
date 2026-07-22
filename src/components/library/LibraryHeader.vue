<script setup lang="ts">
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
    <header class="pb-6 flex justify-between items-center">
        <h1 class="text-xl lg:text-3xl font-semibold tracking-tight text-(--text-primary)">
            Library
        </h1>

        <div class="flex items-center gap-3">
            <button
                v-if="configured"
                @click="emit('sync')"
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

            <button
                @click="emit('import')"
                class="px-4 py-1.5 text-xs font-semibold rounded border border-(--border-color) text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer focus-ring-minimal flex items-center gap-2"
            >
                <span class="material-symbols-outlined text-base select-none">add</span>
                <span>Import</span>
            </button>
        </div>
    </header>
</template>
