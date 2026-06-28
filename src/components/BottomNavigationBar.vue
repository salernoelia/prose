<script
    setup
    lang="ts"
>
type ViewType = 'library' | 'settings' | 'reader'

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
    { label: 'Library', value: 'library', icon: 'local_library' },
    { label: 'Settings', value: 'settings', icon: 'settings' }
]
</script>

<template>
    <nav
        class="fixed z-40 flex items-center justify-around bg-(--bg-app) border border-(--border-color) shadow-lg transition-all duration-300
               bottom-0 left-0 right-0 h-18 px-6 border-x-0 border-b-0 safe-bottom
               md:bottom-6 md:left-1/2 md:right-auto md:-translate-x-1/2 md:h-16 md:w-56 md:rounded-full md:px-4 md:border">
        <button
            v-for="item in navItems"
            :key="item.value"
            @click="emit('navigate', item.value)"
            class="flex flex-col items-center justify-center gap-1 min-w-16 h-full text-xs transition-colors focus-ring-minimal md:flex-row md:gap-2 md:px-3 md:rounded-full"
            :class="currentView === item.value || (item.value === 'library' && currentView === 'reader')
                ? 'text-(--text-primary) font-semibold'
                : 'text-(--text-secondary)'
                "
        >
            <span class="material-symbols-outlined text-lg leading-none select-none">
                {{ item.icon }}
            </span>
            <span>{{ item.label }}</span>
        </button>
    </nav>
</template>

<style scoped>
    @media (max-width: 767px) {
        .safe-bottom {
            padding-bottom: env(safe-area-inset-bottom, 0px);
            height: calc(4.5rem + env(safe-area-inset-bottom, 0px));
        }
    }
</style>