<script
    setup
    lang="ts"
>
import { ref, watchEffect, watch, onUnmounted, onMounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useSettings } from './composables/useSettings'
import { useLibrary } from './composables/useLibrary'
import { useSync } from './composables/useSync'
import SettingsView from './views/SettingsView.vue'
import LibraryView from './views/LibraryView.vue'
import ReaderView from './views/ReaderView.vue'
import BottomNavigationBar from './components/BottomNavigationBar.vue'
import StatsView from './views/StatsView.vue'
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

// Auto-dismiss sync error after 2 seconds
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

// Each theme maps to the classes applied on <html>. Dark-family themes keep the
// base `dark` class so Tailwind `dark:` variants still resolve, plus a variant
// class that overrides the CSS color tokens (its rules sit after `.dark`).
const THEME_CLASSES: Record<string, string[]> = {
    light: [],
    paper: ['paper'],
    dark: ['dark'],
    oled: ['dark', 'oled'],
    sepia: ['sepia'],
    'sepia-dark': ['dark', 'sepia-dark'],
    eink: ['eink'],
    'eink-dark': ['dark', 'eink-dark'],
}

const ALL_THEME_CLASSES = ['dark', 'sepia', 'paper', 'oled', 'sepia-dark', 'eink', 'eink-dark']

watchEffect(() => {
    if (!loaded.value) return

    const root = document.documentElement
    root.classList.remove(...ALL_THEME_CLASSES)
    root.classList.add(...(THEME_CLASSES[theme.value] ?? []))
})

type ViewType = 'library' | 'settings' | 'reader' | 'stats'

const currentView = ref<ViewType>('library')
const selectedBook = ref<BookDto | null>(null)
const libraryLayout = ref<'grid' | 'list'>('grid')

function setView(view: ViewType) {
    currentView.value = view
    if (view !== 'reader') {
        selectedBook.value = null
    }
}

function onSelectBook(book: BookDto) {
    selectedBook.value = book
    currentView.value = 'reader'
}

// Drag and drop of books (epub, pdf) to import
const isDraggingOver = ref(false)
const isMac = ref(typeof navigator !== 'undefined' && /macintosh|mac os x/i.test(navigator.userAgent))
let unlistenDragEnter: UnlistenFn | null = null
let unlistenDragLeave: UnlistenFn | null = null
let unlistenDragDrop: UnlistenFn | null = null

onMounted(async () => {
    try {
        unlistenDragEnter = await listen("tauri://drag-enter", () => {
            if (currentView.value === 'library') {
                isDraggingOver.value = true
            }
        })
        unlistenDragLeave = await listen("tauri://drag-leave", () => {
            isDraggingOver.value = false
        })
        unlistenDragDrop = await listen("tauri://drag-drop", async (event: any) => {
            isDraggingOver.value = false
            if (currentView.value !== 'library') return
            const paths = event.payload?.paths as string[]
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
    <div
        class="h-full overflow-hidden flex flex-col relative bg-(--bg-app) text-(--text-primary)"
    >
        <!-- Drag & Drop Overlay -->
        <div
            v-if="isDraggingOver"
            class="fixed inset-0 z-50 flex items-center justify-center bg-(--bg-app)/60 backdrop-blur-sm transition-all duration-300 pointer-events-none"
        >
            <div class="p-8 max-w-xs rounded-2xl bg-(--bg-card) border border-(--border-color) shadow-xl flex flex-col items-center justify-center text-center animate-fade-in pointer-events-auto">
                <div class="p-4 rounded-full bg-(--accent-color-light) border border-(--border-color) shadow-md flex items-center justify-center mb-4 animate-bounce">
                    <span class="material-symbols-outlined text-3xl text-(--text-primary) select-none">
                        upload_file
                    </span>
                </div>
                <h2 class="text-lg font-bold font-serif text-(--text-primary) mb-1.5">Import Books</h2>
                <p class="text-xs text-(--text-secondary) font-serif">Drop EPUB or PDF files here to add them to your library</p>
            </div>
        </div>

        <!-- Draggable top area for borderless window (macOS overlay titlebar) -->
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
                : 'overflow-y-auto p-6 pt-[calc(1.5rem+env(safe-area-inset-top,0px))] pb-28 items-start scroll-smooth md:p-8 md:pt-12 md:pb-24'"
            style="touch-action: pan-y; -webkit-overflow-scrolling: touch;"
        >
            <div :class="currentView === 'reader' ? 'w-full max-w-4xl mx-auto h-full' : 'w-full max-w-3xl'">
                <div
                    v-if="currentView === 'settings'"
                    class="w-full"
                >
                    <SettingsView />
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
                    <StatsView />
                </div>

                <div
                    v-else-if="currentView === 'reader' && selectedBook"
                    class="w-full h-full"
                >
                    <ReaderView
                        :book="selectedBook"
                        @back-to-library="setView('library')"
                    />
                </div>
            </div>
        </main>

        <BottomNavigationBar
            v-if="currentView !== 'reader'"
            :current-view="currentView"
            @navigate="setView"
        />

        <!-- Floating Toast Notifications (Sonner-like) above the bottom dock -->
        <div
            v-if="currentView !== 'reader'"
            class="fixed bottom-[calc(6.5rem+env(safe-area-inset-bottom,0px))] md:bottom-28 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 w-[calc(100%-2rem)] max-w-sm pointer-events-none"
        >
            <!-- Sync Progress Indicator (Floating) -->
            <Transition name="fade-slide">
                <div
                    v-if="syncing"
                    class="pointer-events-auto p-4 rounded-xl border border-(--border-color) bg-(--bg-card)/90 backdrop-blur-md shadow-lg text-xs text-(--text-primary) w-full"
                >
                    <div class="flex justify-between items-center mb-2">
                        <span class="font-medium flex items-center gap-1.5">
                            <span class="material-symbols-outlined text-sm animate-spin select-none">sync</span>
                            {{ progressMessage || "Syncing library..." }}
                        </span>
                        <span class="tabular-nums font-semibold">{{ Math.round(progressFraction * 100) }}%</span>
                    </div>
                    <div class="w-full h-1 bg-(--border-color) rounded overflow-hidden">
                        <div
                            class="h-full bg-(--text-primary) transition-all duration-300"
                            :style="{ width: progressFraction * 100 + '%' }"
                        ></div>
                    </div>
                </div>
            </Transition>

            <!-- Import Progress Indicator (Floating) -->
            <Transition name="fade-slide">
                <div
                    v-if="importing"
                    class="pointer-events-auto p-4 rounded-xl border border-(--border-color) bg-(--bg-card)/90 backdrop-blur-md shadow-lg text-xs text-(--text-primary) w-full"
                >
                    <div class="flex justify-between items-center mb-2">
                        <span class="font-medium flex items-center gap-1.5">
                            <span class="material-symbols-outlined text-sm animate-pulse select-none">cloud_upload</span>
                            {{ importMessage }}
                        </span>
                        <span class="tabular-nums font-semibold">{{ Math.round(importFraction * 100) }}%</span>
                    </div>
                    <div class="w-full h-1 bg-(--border-color) rounded overflow-hidden">
                        <div
                            class="h-full bg-(--text-primary) transition-all duration-300"
                            :style="{ width: importFraction * 100 + '%' }"
                        ></div>
                    </div>
                </div>
            </Transition>

            <!-- Sync Error Alert (Floating, Auto-dismisses) -->
            <Transition name="fade-slide">
                <div
                    v-if="lastFinishedResult && !lastFinishedResult.success"
                    class="pointer-events-auto p-4 rounded-xl border border-red-200 dark:border-red-950/40 bg-red-50/90 dark:bg-red-950/20 backdrop-blur-md shadow-lg text-xs text-red-700 dark:text-red-400 flex justify-between items-center gap-3 w-full"
                >
                    <div class="flex items-center gap-1.5 min-w-0">
                        <span class="material-symbols-outlined text-sm shrink-0 select-none text-red-500">error</span>
                        <span class="min-w-0 break-words font-medium">Sync failed: {{ lastFinishedResult.message }}</span>
                    </div>
                    <button
                        @click="dismissSyncResult"
                        class="shrink-0 text-xs font-semibold hover:underline cursor-pointer border-0 bg-transparent text-red-700 dark:text-red-400"
                    >
                        Dismiss
                    </button>
                </div>
            </Transition>
        </div>


        <div
            v-if="showClickZonePreview"
            class="fixed inset-0 z-50 pointer-events-none transition-all duration-300 animate-fade-in"
        >
            <div
                class="absolute left-0 top-0 bottom-0 bg-red-500/10 border-r border-dashed border-red-500/30 flex items-center justify-center transition-all duration-150"
                :style="{ width: clickZoneSize + 'vw' }"
            >
                <span
                    class="text-[10px] uppercase tracking-widest font-semibold text-white px-2 py-1 rounded select-none shadow"
                >Prev</span>
            </div>
            <div
                class="absolute right-0 top-0 bottom-0 bg-red-500/10 border-l border-dashed border-red-500/30 flex items-center justify-center transition-all duration-150"
                :style="{ width: clickZoneSize + 'vw' }"
            >
                <span
                    class="text-[10px] uppercase tracking-widest font-semibold text-white px-2 py-1 rounded select-none shadow"
                >Next</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-slide-enter-from {
    opacity: 0;
    transform: translateY(12px) scale(0.95);
}
.fade-slide-leave-to {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
}
</style>
