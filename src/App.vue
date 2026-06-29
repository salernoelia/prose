<script
    setup
    lang="ts"
>
import { ref, watchEffect } from 'vue'
import { useSettings } from './composables/useSettings'
import SettingsView from './views/SettingsView.vue'
import LibraryView from './views/LibraryView.vue'
import ReaderView from './views/ReaderView.vue'
import BottomNavigationBar from './components/BottomNavigationBar.vue'
import type { BookDto } from './ipc/types'

const { theme, loaded, showClickZonePreview, clickZoneSize } = useSettings()

watchEffect(() => {
    if (!loaded.value) return

    const root = document.documentElement
    root.classList.remove('dark', 'sepia')

    if (theme.value === 'dark') {
        root.classList.add('dark')
    } else if (theme.value === 'sepia') {
        root.classList.add('sepia')
    }
})

type ViewType = 'library' | 'settings' | 'reader'

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
</script>

<template>
    <div
        class="h-full overflow-hidden flex flex-col relative bg-(--bg-app) text-(--text-primary)"
    >
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
