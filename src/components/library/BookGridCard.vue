<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import type { LibraryEntryDto } from "../../ipc/types";

const props = defineProps<{
    entry: LibraryEntryDto;
    appDataPath: string;
}>();

const emit = defineEmits<{
    (e: "select"): void;
    (e: "open-menu", event: Event): void;
}>();

const getCoverUrl = (coverPath: string | null) => {
    if (!coverPath || !props.appDataPath) return "";
    const absolutePath = `${props.appDataPath}/${coverPath}`.replace(/\/+/g, "/");
    return convertFileSrc(absolutePath);
};
</script>

<template>
    <div
        @click="emit('select')"
        class="group cursor-pointer flex flex-col gap-3 pb-4 border-b border-transparent hover:border-(--border-color) transition-all"
    >
        <div
            class="aspect-3/4 w-full bg-(--bg-card) border border-(--border-color) rounded overflow-hidden relative shadow-sm group-hover:shadow transition-shadow flex items-center justify-center"
        >
            <button
                @click.stop="(e) => emit('open-menu', e)"
                class="absolute top-2 right-2 w-7 h-7 rounded-full bg-(--bg-card)/90 backdrop-blur border border-(--border-color) flex items-center justify-center text-(--text-tertiary) hover:text-(--text-primary) shadow-sm active:scale-90 transition-all duration-200 cursor-pointer z-10"
                title="Book Actions"
                aria-label="Book Actions"
            >
                <span class="material-symbols-outlined text-base select-none">more_vert</span>
            </button>

            <img
                v-if="entry.book.cover && appDataPath"
                :src="getCoverUrl(entry.book.cover)"
                alt="Book cover"
                class="w-full h-full object-cover"
            />

            <div
                v-else
                class="w-full h-full p-4 flex flex-col justify-between items-start text-left bg-(--bg-card)"
            >
                <span class="text-xs font-semibold uppercase tracking-wider text-(--text-tertiary) leading-none select-none">
                    {{ entry.book.format }}
                </span>
                <span class="text-sm font-semibold tracking-tight text-(--text-primary) line-clamp-3 select-none">
                    {{ entry.book.title }}
                </span>
                <span class="text-xs text-(--text-secondary) truncate w-full select-none">
                    {{ entry.book.author || "Unknown Author" }}
                </span>
            </div>

            <div class="absolute bottom-0 left-0 w-full h-1 bg-(--border-color)">
                <div
                    class="h-full transition-all duration-300"
                    :class="entry.progress >= 1
                        ? 'bg-emerald-700 dark:bg-emerald-600'
                        : 'bg-(--text-primary)'"
                    :style="{ width: entry.progress * 100 + '%' }"
                ></div>
            </div>
        </div>

        <div class="flex flex-col gap-1 text-left">
            <h2 class="text-sm font-medium tracking-tight text-(--text-primary) truncate w-full group-hover:translate-x-0.5 transition-transform duration-200">
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
                        : 'text-(--text-tertiary)'"
                >
                    {{ Math.round(entry.progress * 100) }}%
                </span>
            </div>
        </div>
    </div>
</template>
