<script
    setup
    lang="ts"
>
import { ref, watchEffect } from 'vue'
import { useSettings } from './composables/useSettings'
import SettingsView from './views/Settings.vue'
import LibraryView, { type Book } from './views/Library.vue'
import ReaderView from './views/Reader.vue'

const { theme, loaded, showClickZonePreview, clickZoneSize } = useSettings()

// Active theme class toggles on document root
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

// Navigation views
type ViewType = 'library' | 'settings' | 'sync' | 'reader'

const currentView = ref<ViewType>('library')
const isSidebarOpen = ref(false)
const selectedBook = ref<Book | null>(null)

const navItems = [
    { label: 'Library', value: 'library' as ViewType },
    { label: 'Settings', value: 'settings' as ViewType },
    { label: 'Sync', value: 'sync' as ViewType }
]

function setView(view: ViewType) {
    currentView.value = view
    if (view !== 'reader') {
        selectedBook.value = null
    }
    // On smaller screens, auto-close sidebar after selection
    if (window.innerWidth < 768) {
        isSidebarOpen.value = false
    }
}

function onSelectBook(book: Book) {
    selectedBook.value = book
    currentView.value = 'reader'
    isSidebarOpen.value = false // Close sidebar on reader open
}
</script>

<template>
    <div
        class="min-h-screen flex relative overflow-x-hidden bg-[var(--bg-app)] text-[var(--text-primary)]"
        :class="{ 'h-screen overflow-hidden': currentView === 'reader' }"
    >
        <!-- Menu Toggle Button -->
        <div
            v-if="currentView !== 'reader'"
            class="fixed top-6 left-6 z-50"
        >
            <button
                @click="isSidebarOpen = !isSidebarOpen"
                class="flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors focus-ring-minimal p-2 bg-[var(--bg-app)] border border-[var(--border-color)] rounded-md shadow-sm"
                aria-label="Toggle navigation menu"
            >
                <span class="material-symbols-outlined text-lg leading-none select-none">{{ isSidebarOpen ? "close" :
                    "menu" }}</span>
            </button>
        </div>

        <!-- Sidebar Navigation -->
        <aside
            class="fixed top-0 left-0 h-full w-64 bg-[var(--bg-app)] border-r border-[var(--border-color)] transition-transform duration-300 ease-in-out z-40 p-8 pt-24 flex flex-col justify-between shadow-sm"
            :class="isSidebarOpen ? 'translate-x-0' : '-translate-x-full'"
        >
            <div class="flex flex-col gap-8">
                <!-- Logo / Title -->
                <span class="text-xl font-semibold tracking-tight text-[var(--text-primary)]">Prose</span>

                <!-- Navigation Menu -->
                <nav class="flex flex-col gap-3">
                    <button
                        v-for="item in navItems"
                        :key="item.value"
                        @click="setView(item.value)"
                        class="text-left py-1 text-base hover:text-[var(--text-primary)] transition-all focus-ring-minimal font-normal"
                        :class="currentView === item.value ||
                            (item.value === 'library' && currentView === 'reader')
                            ? 'text-[var(--text-primary)] font-semibold translate-x-1'
                            : 'text-[var(--text-secondary)]'
                            "
                    >
                        {{ item.label }}
                    </button>
                </nav>
            </div>

            <!-- Footer Branding -->
            <span class="text-xs text-[var(--text-tertiary)]">Local-first reader</span>
        </aside>

        <!-- Overlay when sidebar is open on mobile size -->
        <div
            v-if="isSidebarOpen"
            @click="isSidebarOpen = false"
            class="fixed inset-0 bg-black/5 dark:bg-black/20 z-30 md:hidden transition-opacity"
        ></div>

        <!-- Main Content Canvas -->
        <main
            class="flex-1 transition-all duration-300 flex justify-center"
            :class="currentView === 'reader'
                ? 'h-full p-6 md:p-12 overflow-hidden items-stretch'
                : isSidebarOpen
                    ? 'min-h-screen p-8 pt-24 md:pl-72 items-start'
                    : 'min-h-screen p-8 pt-24 pl-8 items-start'
                "
        >
            <div :class="currentView === 'reader'
                ? 'w-full max-w-4xl mx-auto h-full'
                : 'w-full max-w-3xl'
                ">
                <!-- Active View Dispatcher -->
                <div
                    v-if="currentView === 'settings'"
                    class="w-full"
                >
                    <SettingsView />
                </div>

                <!-- Real Library View -->
                <div
                    v-else-if="currentView === 'library'"
                    class="w-full"
                >
                    <LibraryView @select-book="onSelectBook" />
                </div>

                <!-- Real Reader View -->
                <div
                    v-else-if="currentView === 'reader' && selectedBook"
                    class="w-full h-full"
                >
                    <ReaderView
                        :book="selectedBook"
                        @back-to-library="setView('library')"
                    />
                </div>

                <!-- Sync Placeholder View (Matches style guide) -->
                <div
                    v-else-if="currentView === 'sync'"
                    class="w-full animate-fade-in"
                >
                    <header class="pb-6 mb-6 border-b border-[var(--border-color)]">
                        <h1 class="text-xl font-semibold tracking-tight text-[var(--text-primary)]">
                            Sync
                        </h1>
                    </header>
                    <div class="py-12 text-left">
                        <p class="text-base text-[var(--text-secondary)] leading-relaxed">
                            Synchronization is not active.
                        </p>
                        <p class="text-sm text-[var(--text-tertiary)] mt-2 leading-relaxed">
                            Configure your WebDAV connection settings to keep books,
                            highlights, and progress synced across devices.
                        </p>
                    </div>
                </div>
            </div>
        </main>

        <!-- Page-Turn Click Zone Visual Overlay Preview -->
        <div
            v-if="showClickZonePreview"
            class="fixed inset-0 z-50 pointer-events-none transition-all duration-300 animate-fade-in"
        >
            <!-- Left side overlay -->
            <div
                class="absolute left-0 top-0 bottom-0 bg-red-500/10 border-r border-dashed border-red-500/30 flex items-center justify-center transition-all duration-150"
                :style="{ width: clickZoneSize + 'vw' }"
            >
                <span
                    class="text-[10px] uppercase tracking-widest font-semibold text-white px-2 py-1 rounded select-none shadow"
                >Prev</span>
            </div>
            <!-- Right side overlay -->
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
