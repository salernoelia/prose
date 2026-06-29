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
}>()

const emit = defineEmits<{
    (e: 'back'): void
    (e: 'toc'): void
    (e: 'annotations'): void
    (e: 'toggle-bookmark'): void
    (e: 'prev'): void
    (e: 'next'): void
    (e: 'zoom-in'): void
    (e: 'zoom-out'): void
    (e: 'hide'): void
    (e: 'show'): void
}>()
</script>

<template>
    <!-- Floating Dock Card (Compact, Icon-based and Mobile-friendly) -->
    <div
        class="transition-all duration-300 ease-in-out pointer-events-auto
               w-full relative bottom-0 left-0 right-0 z-40 bg-(--bg-card) border-t border-(--border-color)
               md:fixed md:bottom-3 md:left-1/2 md:right-auto md:-translate-x-1/2 md:z-50 md:w-auto md:border-0 md:rounded-none md:bg-transparent"
        :class="[
            visible
                ? 'opacity-100 h-auto py-2 pb-[calc(0.5rem+env(safe-area-inset-bottom,0px))] md:py-0 md:pb-0'
                : 'h-0 overflow-hidden border-t-0 opacity-0 pointer-events-none md:block md:h-auto md:translate-y-16 md:opacity-0 md:pointer-events-none'
        ]"
    >
        <!-- Small Border Card (Rounded Pill on Desktop, Flat bar on Mobile) -->
        <div
            class="w-full h-full flex items-center justify-around gap-4 px-4
                   md:w-auto md:bg-(--bg-card) md:border md:border-(--border-color) md:rounded-full md:shadow-md md:justify-start md:py-2 md:px-4"
        >
            <!-- Back to Library -->
            <button
                @click="emit('back')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
                title="Back to Library"
                aria-label="Back to Library"
            >
                <span class="material-symbols-outlined text-xl leading-none select-none">arrow_back</span>
            </button>
 
            <span class="w-px h-4 bg-(--border-color)"></span>
 
            <!-- Outline / TOC -->
            <button
                @click="emit('toc')"
                :disabled="!hasToc"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal disabled:opacity-20 disabled:hover:text-(--text-secondary) disabled:active:scale-100"
                title="Table of Contents"
                aria-label="Table of Contents"
            >
                <span class="material-symbols-outlined text-xl leading-none select-none">toc</span>
            </button>
 
            <!-- Bookmark Toggler -->
            <button
                @click="emit('toggle-bookmark')"
                class="flex items-center justify-center w-8 h-8 rounded-full transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
                :class="bookmarked
                    ? 'text-(--accent-color) bg-(--accent-color-light)'
                    : 'text-(--text-secondary) hover:text-(--text-primary)'
                    "
                title="Toggle Bookmark"
                aria-label="Toggle Bookmark"
            >
                <span class="material-symbols-outlined text-xl leading-none select-none">
                    {{ bookmarked ? 'bookmark' : 'bookmark_border' }}
                </span>
            </button>

            <!-- Annotations list (bookmarks and highlights) -->
            <button
                @click="emit('annotations')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
                title="Annotations"
                aria-label="Annotations"
            >
                <span class="material-symbols-outlined text-xl leading-none select-none">format_list_bulleted</span>
            </button>

            <!-- Zoom controls (fixed-layout formats only) -->
            <template v-if="canZoom">
                <span class="w-px h-4 bg-(--border-color)"></span>
                <div class="flex items-center gap-1 text-(--text-secondary) select-none">
                    <button
                        @click="emit('zoom-out')"
                        class="flex items-center justify-center w-6 h-6 rounded-full hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
                        title="Zoom Out"
                        aria-label="Zoom Out"
                    >
                        <span class="material-symbols-outlined text-base">zoom_out</span>
                    </button>
                    <button
                        @click="emit('zoom-in')"
                        class="flex items-center justify-center w-6 h-6 rounded-full hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
                        title="Zoom In"
                        aria-label="Zoom In"
                    >
                        <span class="material-symbols-outlined text-base">zoom_in</span>
                    </button>
                </div>
            </template>
 
            <span class="w-px h-4 bg-(--border-color)"></span>
 
            <!-- Page turn indicators in dock -->
            <div class="flex items-center gap-2 text-xs text-(--text-secondary) select-none">
                <button
                    @click="emit('prev')"
                    :disabled="!canPrev"
                    class="flex items-center justify-center w-6 h-6 rounded-full disabled:opacity-20 hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 disabled:active:scale-100"
                >
                    <span class="material-symbols-outlined text-base">chevron_left</span>
                </button>
                <span>{{ progress }}%</span>
                <button
                    @click="emit('next')"
                    :disabled="!canNext"
                    class="flex items-center justify-center w-6 h-6 rounded-full disabled:opacity-20 hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 disabled:active:scale-100"
                >
                    <span class="material-symbols-outlined text-base">chevron_right</span>
                </button>
            </div>
 
            <span class="w-px h-4 bg-(--border-color)"></span>
 
            <!-- Hide Dock Action -->
            <button
                @click="emit('hide')"
                class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal"
                title="Hide Controls"
                aria-label="Hide Controls"
            >
                <span class="material-symbols-outlined text-xl leading-none select-none">visibility_off</span>
            </button>
        </div>
    </div>

    <!-- Small controls restoration tab (displays when dock is hidden) -->
    <div
        class="fixed bottom-3 left-1/2 -translate-x-1/2 z-50 transition-all duration-300 ease-in-out pointer-events-auto"
        :class="!visible ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0 pointer-events-none'"
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
