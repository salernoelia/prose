<script setup lang="ts">
import type { LibraryEntryDto } from "../../ipc/types";

defineProps<{
    entry: LibraryEntryDto;
}>();

const emit = defineEmits<{
    (e: "select"): void;
    (e: "open-menu", event: Event): void;
}>();
</script>

<template>
    <div
        @click="emit('select')"
        class="group cursor-pointer py-3.5 px-4 rounded-xl hover:bg-(--accent-color-light) transition-all flex flex-col gap-2.5"
    >
        <div class="flex justify-between items-start gap-4">
            <h2 class="text-base font-medium tracking-tight text-(--text-primary) group-hover:translate-x-0.5 transition-transform duration-200">
                {{ entry.book.title }}
            </h2>
            <button
                @click.stop="(e) => emit('open-menu', e)"
                class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer shrink-0"
                title="Book Actions"
                aria-label="Book Actions"
            >
                <span class="material-symbols-outlined text-base select-none">more_vert</span>
            </button>
        </div>

        <div class="flex justify-between items-center text-xs">
            <span class="text-(--text-secondary)">
                {{ entry.book.author || "Unknown Author" }}
            </span>
            <span
                class="text-xs font-semibold tabular-nums px-2 py-0.5 rounded shrink-0 transition-colors"
                :class="entry.progress >= 1
                    ? 'bg-emerald-100 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-400'
                    : 'bg-(--accent-color-light) text-(--text-secondary)'"
            >
                {{ Math.round(entry.progress * 100) }}%
            </span>
        </div>

        <div class="w-full h-1 bg-(--border-color) rounded-full overflow-hidden mt-1 opacity-60 group-hover:opacity-100 transition-opacity">
            <div
                class="h-full transition-all duration-300"
                :class="entry.progress >= 1
                    ? 'bg-emerald-700 dark:bg-emerald-600'
                    : 'bg-(--text-primary)'"
                :style="{ width: entry.progress * 100 + '%' }"
            ></div>
        </div>
    </div>
</template>
