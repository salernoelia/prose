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
    <ul class="flex flex-col gap-0.5">
        <li
            v-for="(item, index) in items"
            :key="item.href + ':' + index"
            class="flex flex-col"
        >
            <button
                @click="emit('select', item.href)"
                class="group w-full text-left py-2 px-2.5 rounded-lg text-xs font-medium text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--bg-card) transition-all cursor-pointer flex items-center gap-2"
                :style="{ paddingLeft: `${0.75 + (level ?? 0) * 1}rem` }"
                :title="item.label"
            >
                <span
                    v-if="!level"
                    class="w-1.5 h-1.5 rounded-full bg-(--border-color) group-hover:bg-(--accent-color) transition-colors shrink-0"
                ></span>
                <span
                    v-else
                    class="w-1 h-1 rounded-full bg-(--border-color) shrink-0"
                ></span>
                <span class="truncate flex-1">{{ item.label || 'Untitled Chapter' }}</span>
                <span class="material-symbols-outlined text-xs text-(--text-tertiary) opacity-0 group-hover:opacity-100 transition-opacity">
                    arrow_forward
                </span>
            </button>
            <ReaderTocList
                v-if="item.subitems && item.subitems.length"
                :items="item.subitems"
                :level="(level ?? 0) + 1"
                @select="emit('select', $event)"
            />
        </li>
    </ul>
</template>
