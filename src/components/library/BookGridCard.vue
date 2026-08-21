<script setup lang="ts">
import { ref, computed } from "vue";
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

const imageLoadFailed = ref(false);

const getCoverUrl = (coverPath: string | null) => {
    if (!coverPath || !props.appDataPath || imageLoadFailed.value) return "";
    try {
        const absolutePath = `${props.appDataPath}/${coverPath}`.replace(/\/+/g, "/");
        if (typeof convertFileSrc === "function") {
            return convertFileSrc(absolutePath);
        }
        return `asset://localhost/${encodeURIComponent(absolutePath.replace(/^\/+/, ""))}`;
    } catch {
        return "";
    }
};

// Generate consistent classic book palettes for books without an embedded cover
const COVER_PALETTES = [
    { bg: 'bg-[#09332C] text-[#F7EDDA]', accent: 'border-[#FFA74F]/40', label: 'text-[#FFA74F]/80' },
    { bg: 'bg-[#2C1E13] text-[#F7DFBA]', accent: 'border-[#F7DFBA]/30', label: 'text-[#F7DFBA]/70' },
    { bg: 'bg-[#2E4B3C] text-[#F7EDDA]', accent: 'border-[#F7EDDA]/30', label: 'text-[#F7EDDA]/70' },
    { bg: 'bg-[#1C1917] text-[#F7EDDA]', accent: 'border-[#F0531C]/40', label: 'text-[#F0531C]/80' },
    { bg: 'bg-[#4A2E35] text-[#FCEDF0]', accent: 'border-[#F2C9D2]/30', label: 'text-[#F2C9D2]/70' },
    { bg: 'bg-[#5C4028] text-[#FAEDD8]', accent: 'border-[#FFA74F]/30', label: 'text-[#FFA74F]/70' },
];

const fallbackPalette = computed(() => {
    let hash = 0;
    const str = props.entry.book.title + (props.entry.book.author || '');
    for (let i = 0; i < str.length; i++) {
        hash = (hash << 5) - hash + str.charCodeAt(i);
        hash |= 0;
    }
    const idx = Math.abs(hash) % COVER_PALETTES.length;
    return COVER_PALETTES[idx];
});
</script>

<template>
    <div
        @click="emit('select')"
        class="group cursor-pointer flex flex-col gap-2.5 pb-2 transition-all"
    >
        <!-- Book Cover Container with physical book depth and 3D spine -->
        <div
            class="aspect-2/3 w-full book-cover-3d relative bg-(--bg-card) group-hover:-translate-y-1 transition-transform duration-200"
        >
            <!-- Actions Menu Button -->
            <button
                @click.stop="(e) => emit('open-menu', e)"
                class="absolute top-2 right-2 w-7 h-7 rounded-full bg-(--bg-card)/90 backdrop-blur border border-(--border-color) flex items-center justify-center text-(--text-tertiary) hover:text-(--text-primary) shadow-sm active:scale-90 transition-all duration-150 cursor-pointer z-10 opacity-80 group-hover:opacity-100"
                title="Book Actions"
                aria-label="Book Actions"
            >
                <span class="material-symbols-outlined text-base select-none">more_vert</span>
            </button>

            <!-- Book Cover Image -->
            <img
                v-if="entry.book.cover && appDataPath && !imageLoadFailed && getCoverUrl(entry.book.cover)"
                :src="getCoverUrl(entry.book.cover)"
                alt="Book cover"
                class="w-full h-full object-cover"
                @error="imageLoadFailed = true"
            />

            <!-- Bookish Typographic Fallback Cover -->
            <div
                v-else
                class="w-full h-full p-4 flex flex-col justify-between items-center text-center relative select-none"
                :class="fallbackPalette.bg"
            >
                <div
                    class="w-full h-full border border-dashed rounded flex flex-col justify-between p-3"
                    :class="fallbackPalette.accent"
                >
                    <span
                        class="text-[10px] font-sans uppercase tracking-widest"
                        :class="fallbackPalette.label"
                    >
                        {{ entry.book.format }}
                    </span>

                    <div class="my-auto py-2">
                        <h3 class="text-sm font-semibold tracking-tight leading-snug line-clamp-4 font-serif">
                            {{ entry.book.title }}
                        </h3>
                    </div>

                    <span
                        class="text-[11px] font-sans truncate w-full"
                        :class="fallbackPalette.label"
                    >
                        {{ entry.book.author || "Unknown Author" }}
                    </span>
                </div>
            </div>

            <!-- Reading progress bar along bottom edge of book -->
            <div
                v-if="entry.progress > 0"
                class="absolute bottom-0 left-0 w-full h-1 bg-black/20 z-10"
            >
                <div
                    class="h-full transition-all duration-300"
                    :class="entry.progress >= 1
                        ? 'bg-emerald-500'
                        : 'bg-(--accent-color)'"
                    :style="{ width: entry.progress * 100 + '%' }"
                ></div>
            </div>
        </div>

        <!-- Book Metadata -->
        <div class="flex flex-col gap-0.5 text-left px-0.5">
            <h2 class="text-sm font-medium tracking-tight text-(--text-primary) truncate w-full group-hover:text-(--accent-color) transition-colors duration-150">
                {{ entry.book.title }}
            </h2>
            <div class="flex justify-between items-center text-xs text-(--text-secondary)">
                <span class="truncate max-w-[70%] text-[11px]">
                    {{ entry.book.author || "Unknown Author" }}
                </span>
                <span
                    v-if="entry.progress > 0"
                    class="tabular-nums text-[11px] font-medium"
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
