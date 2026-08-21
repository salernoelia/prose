<script setup lang="ts">
type ViewType = 'home' | 'library' | 'settings' | 'reader' | 'stats'

interface NavItem {
    label: string
    value: ViewType
    icon: string
}

defineProps<{
    currentView: ViewType
}>()

const emit = defineEmits<{
    (e: 'navigate', view: ViewType): void
}>()

const navItems: NavItem[] = [
    { label: 'Home', value: 'home', icon: 'auto_stories' },
    { label: 'Library', value: 'library', icon: 'local_library' },
    { label: 'Stats', value: 'stats', icon: 'bar_chart' },
    { label: 'Settings', value: 'settings', icon: 'settings' },
]
</script>

<template>
    <nav
        class="fixed z-40 font-serif select-none transition-all duration-200 bottom-0 left-0 right-0 h-[calc(4.75rem+env(safe-area-inset-bottom,0px))] pb-[calc(0.5rem+env(safe-area-inset-bottom,0px))] pt-2 bg-(--bg-app)/92 dark:bg-(--bg-app)/95 backdrop-blur-xl border-t border-(--border-color) flex items-center justify-around px-2 md:bottom-8 md:left-1/2 md:right-auto md:-translate-x-1/2 md:h-14 md:w-auto md:min-w-[360px] md:rounded-full md:border md:border-(--border-color) md:bg-(--bg-card)/90 md:shadow-lg md:p-1.5 md:gap-1 md:pb-1.5 md:pt-1.5"
    >
        <button
            v-for="item in navItems"
            :key="item.value"
            @click="emit('navigate', item.value)"
            class="relative flex flex-col md:flex-row items-center justify-center gap-1 md:gap-2 flex-1 md:flex-initial h-full px-2.5 md:px-4 py-1 rounded-xl md:rounded-full transition-all duration-150 cursor-pointer active:scale-95 text-(--text-secondary) hover:text-(--text-primary)"
            :class="[
                currentView === item.value || (item.value === 'home' && currentView === 'reader')
                    ? 'text-(--text-primary) font-semibold'
                    : 'font-normal'
            ]"
            :aria-label="item.label"
        >
            <!-- Active pill background -->
            <div
                v-if="currentView === item.value || (item.value === 'home' && currentView === 'reader')"
                class="absolute inset-0 rounded-xl md:rounded-full bg-(--text-primary)/8 dark:bg-white/10 -z-10 transition-all duration-200"
            ></div>

            <span
                class="material-symbols-outlined text-[23px] md:text-[19px] leading-none flex items-center justify-center select-none shrink-0"
            >
                {{ item.icon }}
            </span>
            <span class="text-[11px] md:text-xs font-serif leading-none flex items-center tracking-normal">
                {{ item.label }}
            </span>
        </button>
    </nav>
</template>
