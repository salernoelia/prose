<script
    setup
    lang="ts"
>
import { computed, onMounted, onUnmounted, ref, toRef } from 'vue'
import { useSettings } from '../composables/useSettings'
import { useReader } from '../composables/useReader'
import ReaderClickZones from '../components/reader/ReaderClickZones.vue'
import ReaderDock from '../components/reader/ReaderDock.vue'
import ReaderTocDrawer from '../components/reader/ReaderTocDrawer.vue'
import { useSync } from '../composables/useSync'
import type { BookDto } from '../ipc/types'

const props = defineProps<{
    book: BookDto
}>()

const emit = defineEmits<{
    (e: 'back-to-library'): void
}>()

const { clickZoneSize } = useSettings()

const {
    host,
    loading,
    error,
    progress,
    toc,
    hasToc,
    canZoom,
    next,
    prev,
    goToHref,
    zoomIn,
    zoomOut,
} = useReader(toRef(props, 'book'))

const showDock = ref(true)
const showToc = ref(false)
const isBookmarked = ref(false)

const canPrev = computed(() => progress.value > 0)
const canNext = computed(() => progress.value < 100)

const { configured, syncing, triggerSync } = useSync()

function toggleDock() {
    showDock.value = !showDock.value
}

function onSelectToc(href: string) {
    void goToHref(href)
}

function handleBack() {
    emit('back-to-library')
    if (configured.value && !syncing.value) {
        void triggerSync()
    }
}

const handleKeyDown = (e: KeyboardEvent) => {
    const target = e.target as HTMLElement | null
    if (target && (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable
    )) {
        return
    }

    if (e.key === 'ArrowRight') {
        next()
    } else if (e.key === 'ArrowLeft') {
        prev()
    }
}

onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
    <div class="w-full relative h-full flex flex-col justify-between select-none">
        <ReaderClickZones
            :zone-size="clickZoneSize"
            @prev="prev"
            @next="next"
            @toggle="toggleDock"
        />

        <!-- Non-Scrolling Reading Canvas (Overflow hidden, flex-1, with fade-in) -->
        <div
            class="relative z-0 w-full flex-1 overflow-hidden select-text transition-all duration-300 flex flex-col animate-fade-in"
        >
            <!-- Book Header Info (Subtle) -->
            <header
                class="mb-3 pb-2 border-b border-(--border-color) flex justify-between items-center text-xs text-(--text-tertiary) select-none"
                :style="{ paddingLeft: '1.5rem', paddingRight: '1.5rem' }"
            >
                <span class="truncate pr-4">{{ book.title }}</span>
                <span>{{ book.author }}</span>
            </header>

            <!-- Renderer host: foliate-js (ePub) or pdf.js (PDF) mounts here -->
            <div class="relative flex-1 overflow-hidden">
                <div
                    ref="host"
                    class="absolute inset-0"
                ></div>

                <!-- Loading state -->
                <div
                    v-if="loading"
                    class="absolute inset-0 flex items-center justify-center text-sm text-(--text-tertiary) select-none animate-fade-in"
                >
                    <span class="material-symbols-outlined animate-spin mr-2">progress_activity</span>
                    Opening book
                </div>

                <!-- Error state -->
                <div
                    v-else-if="error"
                    class="absolute inset-0 flex flex-col items-center justify-center gap-2 px-8 text-center select-none"
                >
                    <span class="material-symbols-outlined text-3xl text-(--text-tertiary)">error</span>
                    <p class="text-sm text-(--text-secondary)">{{ error }}</p>
                </div>
            </div>
        </div>

        <ReaderDock
            :visible="showDock"
            :progress="progress"
            :can-prev="canPrev"
            :can-next="canNext"
            :bookmarked="isBookmarked"
            :has-toc="hasToc"
            :can-zoom="canZoom"
            @back="handleBack"
            @toc="showToc = true"
            @toggle-bookmark="isBookmarked = !isBookmarked"
            @prev="prev"
            @next="next"
            @zoom-in="zoomIn"
            @zoom-out="zoomOut"
            @hide="showDock = false"
            @show="showDock = true"
        />

        <ReaderTocDrawer
            v-model:visible="showToc"
            :items="toc"
            @select="onSelectToc"
        />
    </div>
</template>
