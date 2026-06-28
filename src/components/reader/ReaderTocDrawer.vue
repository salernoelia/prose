<script
    setup
    lang="ts"
>
import Drawer from 'primevue/drawer'
import ReaderTocList from './ReaderTocList.vue'
import type { TocItem } from '../../readers'

defineProps<{
    visible: boolean
    items: TocItem[]
}>()

const emit = defineEmits<{
    (e: 'update:visible', value: boolean): void
    (e: 'select', href: string): void
}>()

function onSelect(href: string) {
    emit('select', href)
    emit('update:visible', false)
}
</script>

<template>
    <Drawer
        :visible="visible"
        @update:visible="emit('update:visible', $event)"
        position="right"
        :modal="true"
        :show-close-icon="false"
        class="!w-80 !max-w-[85vw] !bg-(--bg-app) !border-l !border-(--border-color)"
    >
        <template #container="{ closeCallback }">
            <div class="flex flex-col h-full">
                <header
                    class="flex items-center justify-between px-4 py-3 border-b border-(--border-color)"
                >
                    <span class="text-sm font-semibold tracking-wide text-(--text-primary) select-none">
                        Contents
                    </span>
                    <button
                        @click="closeCallback"
                        class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                        title="Close"
                        aria-label="Close"
                    >
                        <span class="material-symbols-outlined text-xl leading-none select-none">close</span>
                    </button>
                </header>

                <nav class="flex-1 overflow-y-auto px-2 py-2">
                    <ReaderTocList
                        v-if="items.length"
                        :items="items"
                        @select="onSelect"
                    />
                    <p
                        v-else
                        class="px-3 py-4 text-sm text-(--text-tertiary) select-none"
                    >
                        No table of contents available.
                    </p>
                </nav>
            </div>
        </template>
    </Drawer>
</template>
