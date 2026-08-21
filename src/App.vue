<script setup lang="ts">
import { ref, watchEffect, watch, onUnmounted, onMounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettings } from './composables/useSettings'
import { useLibrary } from './composables/useLibrary'
import { useSync } from './composables/useSync'
import HomeView from './views/HomeView.vue'
import SettingsView from './views/SettingsView.vue'
import LibraryView from './views/LibraryView.vue'
import ReaderView from './views/ReaderView.vue'
import BottomNavigationBar from './components/BottomNavigationBar.vue'
import StatsView from './views/StatsView.vue'
import {
    DragDropOverlay,
    ClickZonePreviewOverlay,
    ToastNotifications,
} from './components/app'
import type { BookDto } from './ipc/types'

const { theme, loaded, showClickZonePreview, clickZoneSize } = useSettings()

const {
    importing,
    importMessage,
    importFraction,
    importBook,
} = useLibrary()

const {
    syncing,
    progressMessage,
    progressFraction,
    lastFinishedResult,
    dismissSyncResult,
} = useSync()

let syncErrorTimeout: ReturnType<typeof setTimeout> | null = null
watch(
    lastFinishedResult,
    (result) => {
        if (syncErrorTimeout) {
            clearTimeout(syncErrorTimeout)
            syncErrorTimeout = null
        }
        if (result && !result.success) {
            syncErrorTimeout = setTimeout(() => {
                dismissSyncResult()
            }, 2000)
        }
    },
    { immediate: true }
)

onUnmounted(() => {
    if (syncErrorTimeout) {
        clearTimeout(syncErrorTimeout)
    }
})

const THEME_CLASSES: Record<string, string[]> = {
    light: [],
    paper: ['paper'],
    dark: ['dark'],
    oled: ['dark', 'oled'],
    night: ['dark', 'night'],
    sepia: ['sepia'],
    'sepia-dark': ['dark', 'sepia-dark'],
    eink: ['eink'],
    'eink-dark': ['dark', 'eink-dark'],
}

const ALL_THEME_CLASSES = ['dark', 'sepia', 'paper', 'oled', 'night', 'sepia-dark', 'eink', 'eink-dark']

watchEffect(() => {
    if (!loaded.value) return

    const root = document.documentElement
    root.classList.remove(...ALL_THEME_CLASSES)
    root.classList.add(...(THEME_CLASSES[theme.value] ?? []))
})

export type ViewType = 'home' | 'library' | 'settings' | 'reader' | 'stats'

const currentView = ref<ViewType>('home')
const previousView = ref<ViewType>('home')
const selectedBook = ref<BookDto | null>(null)
const libraryLayout = ref<'grid' | 'list'>('grid')

function setView(view: ViewType) {
    if (currentView.value !== 'reader') {
        previousView.value = currentView.value
    }
    currentView.value = view
    if (view !== 'reader') {
        selectedBook.value = null
    }
}

function onSelectBook(book: BookDto) {
    if (currentView.value !== 'reader') {
        previousView.value = currentView.value
    }
    selectedBook.value = book
    currentView.value = 'reader'
}

const isDraggingOver = ref(false)
const isMac = ref(typeof navigator !== 'undefined' && /macintosh|mac os x/i.test(navigator.userAgent))
let unlistenDragEnter: UnlistenFn | null = null
let unlistenDragLeave: UnlistenFn | null = null
let unlistenDragDrop: UnlistenFn | null = null

onMounted(async () => {
    try {
        unlistenDragEnter = await listen("tauri://drag-enter", () => {
            if (currentView.value === 'home' || currentView.value === 'library') {
                isDraggingOver.value = true
            }
        })
        unlistenDragLeave = await listen("tauri://drag-leave", () => {
            isDraggingOver.value = false
        })
        unlistenDragDrop = await listen<{ paths?: string[] }>("tauri://drag-drop", async (event) => {
            isDraggingOver.value = false
            if (currentView.value !== 'home' && currentView.value !== 'library') return
            const paths = event.payload?.paths
            if (paths && paths.length > 0) {
                const validPaths = paths.filter(p => {
                    const ext = p.split('.').pop()?.toLowerCase()
                    return ext === 'epub' || ext === 'pdf'
                })
                for (const path of validPaths) {
                    try {
                        await importBook(path)
                    } catch (err) {
                        console.error(`Failed to import book at ${path}:`, err)
                    }
                }
            }
        })
    } catch (err) {
        console.error("Failed to setup drag & drop listeners in App:", err)
    }
})

onUnmounted(() => {
    if (unlistenDragEnter) unlistenDragEnter()
    if (unlistenDragLeave) unlistenDragLeave()
    if (unlistenDragDrop) unlistenDragDrop()
})
</script>

<template>
    <div class="h-full overflow-hidden flex flex-col relative bg-(--bg-app) text-(--text-primary)">
        <DragDropOverlay :visible="isDraggingOver" />

        <div
            v-if="isMac"
            data-tauri-drag-region
            class="fixed top-0 right-0 z-[9999]"
            :style="{ left: '80px', height: '30px', backgroundColor: 'rgba(0, 0, 0, 0.001)' }"
        ></div>

        <main
            class="flex-1 min-h-0 transition-all duration-300 flex justify-center w-full"
            :class="currentView === 'reader'
                ? 'h-full md:px-12 md:pb-8 overflow-hidden items-stretch'
                : 'overflow-y-auto p-6 pt-[calc(1.5rem+env(safe-area-inset-top,0px))] pb-36 items-start scroll-smooth md:p-8 md:pt-12 md:pb-36'"
            style="touch-action: pan-y; -webkit-overflow-scrolling: touch;"
        >
            <div :class="currentView === 'reader' ? 'w-full max-w-4xl mx-auto h-full' : 'w-full max-w-3xl'">
                <div
                    v-if="currentView === 'home'"
                    class="w-full"
                >
                    <HomeView @select-book="onSelectBook" @navigate="setView" />
                </div>

                <div
                    v-else-if="currentView === 'library'"
                    class="w-full"
                >
                    <LibraryView v-model:layout="libraryLayout" @select-book="onSelectBook" />
                </div>

                <div
                    v-else-if="currentView === 'stats'"
                    class="w-full"
                >
                    <StatsView @select-book="onSelectBook" />
                </div>

                <div
                    v-else-if="currentView === 'settings'"
                    class="w-full"
                >
                    <SettingsView />
                </div>

                <div
                    v-else-if="currentView === 'reader' && selectedBook"
                    class="w-full h-full"
                >
                    <ReaderView
                        :book="selectedBook"
                        @back-to-library="setView(previousView === 'library' ? 'library' : 'home')"
                    />
                </div>
            </div>
        </main>

        <BottomNavigationBar
            v-if="currentView !== 'reader'"
            :current-view="currentView"
            @navigate="setView"
        />

        <ToastNotifications
            v-if="currentView !== 'reader'"
            :syncing="syncing"
            :progressMessage="progressMessage"
            :progressFraction="progressFraction"
            :importing="importing"
            :importMessage="importMessage"
            :importFraction="importFraction"
            :lastFinishedResult="lastFinishedResult"
            @dismiss-sync="dismissSyncResult"
        />

        <ClickZonePreviewOverlay
            :visible="showClickZonePreview"
            :clickZoneSize="clickZoneSize"
        />
    </div>
</template>
