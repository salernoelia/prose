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
        class="group cursor-pointer py-3 px-3.5 rounded-xl border border-transparent hover:border-(--border-color) hover:bg-(--bg-card) transition-all flex items-center justify-between gap-4"
    >
        <div class="flex items-center gap-3.5 min-w-0 flex-1">
            <!-- Format / Status Icon Indicator -->
            <div
                class="w-8 h-11 rounded border border-(--border-color) bg-(--accent-color-light) flex items-center justify-center shrink-0 shadow-xs select-none"
            >
                <span class="text-[9px] font-sans font-semibold uppercase tracking-wider text-(--text-secondary)">
                    {{ entry.book.format }}
                </span>
            </div>

            <!-- Title, Author, Progress Bar -->
            <div class="flex flex-col gap-1 min-w-0 flex-1">
                <h2 class="text-sm font-medium tracking-tight text-(--text-primary) truncate group-hover:text-(--accent-color) transition-colors">
                    {{ entry.book.title }}
                </h2>
                <div class="flex items-center gap-2 text-xs text-(--text-secondary)">
                    <span class="truncate text-[11px]">{{ entry.book.author || "Unknown Author" }}</span>
                    <span v-if="entry.progress > 0" class="text-(--text-tertiary) text-[10px]">·</span>
                    <span
                        v-if="entry.progress > 0"
                        class="text-[10px] font-medium tabular-nums shrink-0"
                        :class="entry.progress >= 1 ? 'text-emerald-700 dark:text-emerald-400 font-semibold' : 'text-(--text-tertiary)'"
                    >
                        {{ Math.round(entry.progress * 100) }}%
                    </span>
                </div>

                <!-- Thin progress bar -->
                <div
                    v-if="entry.progress > 0"
                    class="w-full max-w-xs h-0.5 bg-(--border-color) rounded-full overflow-hidden mt-0.5"
                >
                    <div
                        class="h-full transition-all duration-300"
                        :class="entry.progress >= 1 ? 'bg-emerald-500' : 'bg-(--accent-color)'"
                        :style="{ width: entry.progress * 100 + '%' }"
                    ></div>
                </div>
            </div>
        </div>

        <!-- Menu Action -->
        <button
            @click.stop="(e) => emit('open-menu', e)"
            class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all cursor-pointer shrink-0 opacity-60 group-hover:opacity-100"
            title="Book Actions"
            aria-label="Book Actions"
        >
            <span class="material-symbols-outlined text-base select-none">more_vert</span>
        </button>
    </div>
</template>
