<script
    setup
    lang="ts"
>
import type { TocItem } from '../../readers'

defineProps<{
    items: TocItem[]
    /** Nesting depth, used only for indentation. */
    level?: number
}>()

const emit = defineEmits<{
    (e: 'select', href: string): void
}>()
</script>

<template>
    <ul class="flex flex-col">
        <li
            v-for="(item, index) in items"
            :key="item.href + ':' + index"
        >
            <button
                @click="emit('select', item.href)"
                class="w-full text-left py-2 rounded-md text-sm text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal truncate"
                :style="{ paddingLeft: 0.75 + (level ?? 0) * 1 + 'rem', paddingRight: '0.75rem' }"
                :title="item.label"
            >
                {{ item.label || 'Untitled' }}
            </button>
            <ReaderTocList
                v-if="item.subitems.length"
                :items="item.subitems"
                :level="(level ?? 0) + 1"
                @select="emit('select', $event)"
            />
        </li>
    </ul>
</template>
