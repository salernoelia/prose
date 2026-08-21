<script
    setup
    lang="ts"
>
defineProps<{
    visible: boolean
    progress: number
    canPrev: boolean
    canNext: boolean
    bookmarked: boolean
    hasToc: boolean
    canZoom: boolean
    canUndoJump: boolean
}>()

const emit = defineEmits<{
    (e: 'back'): void
    (e: 'toc'): void
    (e: 'annotations'): void
    (e: 'toggle-bookmark'): void
    (e: 'prev'): void
    (e: 'next'): void
    (e: 'undo-jump'): void
    (e: 'zoom-in'): void
    (e: 'zoom-out'): void
    (e: 'quick-settings'): void
    (e: 'show'): void
}>()
</script>

<template>
    <!-- Floating Dock Card (Compact, Icon-based and Mobile-friendly) -->
    <div
        class="transition-all duration-300 ease-in-out pointer-events-auto
               w-full fixed bottom-0 left-0 right-0 z-40 bg-(--bg-card)/90 backdrop-blur-md border-t border-(--border-color)
               md:bottom-4 md:left-1/2 md:right-auto md:-translate-x-1/2 md:z-50 md:w-auto md:border-0 md:rounded-none md:bg-transparent"
        :class="[
            visible
                ? 'opacity-100 translate-y-0 py-2 pb-[calc(0.5rem+env(safe-area-inset-bottom,0px))] md:py-0 md:pb-0'
                : 'translate-y-full opacity-0 pointer-events-none md:block md:translate-y-16 md:opacity-0 md:pointer-events-none'
        ]"
    >
        <!-- Small Border Card (Rounded Pill on Desktop, Flat bar on Mobile) -->
        <div
            class="w-full h-full flex items-center justify-around gap-1.5 px-2
                   md:w-auto md:bg-(--bg-card)/90 md:backdrop-blur-md md:border md:border-(--border-color) md:rounded-full md:shadow-lg md:justify-start md:py-1.5 md:px-3 md:gap-2">
            <!-- Back to Library -->
            <button
                @click="emit('back')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                title="Back to Library"
                aria-label="Back to Library"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">arrow_back</span>
            </button>

            <!-- Outline / TOC -->
            <button
                @click="emit('toc')"
                :disabled="!hasToc"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal disabled:opacity-20 disabled:hover:text-(--text-secondary) disabled:hover:bg-transparent cursor-pointer"
                title="Book Structure & Chapters"
                aria-label="Book Structure & Chapters"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">menu_book</span>
            </button>

            <!-- Bookmark Toggler -->
            <button
                @click="emit('toggle-bookmark')"
                class="flex items-center justify-center w-8 h-8 rounded-full transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                :class="bookmarked
                    ? 'text-(--accent-color) bg-(--accent-color-light)'
                    : 'text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light)'"
                title="Toggle Bookmark"
                aria-label="Toggle Bookmark"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">
                    {{ bookmarked ? 'bookmark' : 'bookmark_border' }}
                </span>
            </button>

            <!-- Annotations list (bookmarks and highlights) -->
            <button
                @click="emit('annotations')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                title="Bookmarks & Highlights"
                aria-label="Bookmarks & Highlights"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">format_ink_highlighter</span>
            </button>

            <!-- Zoom controls (fixed-layout formats only) -->
            <template v-if="canZoom">
                <div class="flex items-center gap-0.5 text-(--text-secondary) select-none">
                    <button
                        @click="emit('zoom-out')"
                        class="flex items-center justify-center w-7 h-7 rounded-full hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                        title="Zoom Out"
                        aria-label="Zoom Out"
                    >
                        <span class="material-symbols-outlined text-base">zoom_out</span>
                    </button>
                    <button
                        @click="emit('zoom-in')"
                        class="flex items-center justify-center w-7 h-7 rounded-full hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                        title="Zoom In"
                        aria-label="Zoom In"
                    >
                        <span class="material-symbols-outlined text-base">zoom_in</span>
                    </button>
                </div>
            </template>

            <!-- Page turn indicators in dock -->
            <div class="flex items-center gap-1 text-xs text-(--text-secondary) select-none px-1">
                <button
                    @click="emit('prev')"
                    :disabled="!canPrev"
                    class="flex items-center justify-center w-7 h-7 rounded-full disabled:opacity-20 hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 cursor-pointer"
                    title="Previous Page"
                    aria-label="Previous Page"
                >
                    <span class="material-symbols-outlined text-base">chevron_left</span>
                </button>
                <span class="font-serif font-medium text-[11px] tabular-nums min-w-[28px] text-center text-(--text-primary)">
                    {{ progress }}%
                </span>
                <button
                    @click="emit('next')"
                    :disabled="!canNext"
                    class="flex items-center justify-center w-7 h-7 rounded-full disabled:opacity-20 hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 cursor-pointer"
                    title="Next Page"
                    aria-label="Next Page"
                >
                    <span class="material-symbols-outlined text-base">chevron_right</span>
                </button>
            </div>

            <!-- Quick reading settings (theme, text size, line spacing) -->
            <button
                @click="emit('quick-settings')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                title="Appearance & Typography"
                aria-label="Appearance & Typography"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">tune</span>
            </button>

            <!-- Undo the last jump -->
            <button
                v-if="canUndoJump"
                @click="emit('undo-jump')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-all duration-100 active:scale-90 focus-ring-minimal cursor-pointer"
                title="Back to previous position"
                aria-label="Back to previous position"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">undo</span>
            </button>
        </div>
    </div>

    <!-- Small controls restoration tab (displays when dock is hidden) -->
    <div
        class="fixed bottom-3 left-1/2 -translate-x-1/2 z-50 transition-all duration-300 ease-in-out pointer-events-auto"
        :class="!visible ? 'translate-y-0 opacity-50' : 'translate-y-8 opacity-0 pointer-events-none'"
    >
        <button
            @click="emit('show')"
            class="flex items-center justify-center w-8 h-8 rounded-full bg-(--bg-card) border border-(--border-color) shadow-sm text-(--text-tertiary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
            title="Show Controls"
            aria-label="Show Controls"
        >
            <span class="material-symbols-outlined text-lg leading-none select-none">menu</span>
        </button>
    </div>
</template>
