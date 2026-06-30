<script
    setup
    lang="ts"
>
import { computed } from 'vue'
import type { ViewportRect } from '../../readers'

const props = defineProps<{
    rect: ViewportRect | null
}>()

// On touch devices the OS draws its own selection menu (Copy / Paste / Select
// All) directly above the selection, which covers a toolbar placed there. Detect
// a coarse pointer and drop our toolbar below the selection instead.
const isTouch =
    typeof window !== 'undefined' && !!window.matchMedia?.('(pointer: coarse)').matches

// Approximate toolbar height, used only to decide whether a below placement
// would run off the bottom of the viewport.
const TOOLBAR_HEIGHT = 48

// Float the toolbar over the target rect, nudged back inside the viewport
// horizontally so it never clips off-screen on a near-edge selection. Touch
// devices prefer a below placement to clear the native selection menu.
const placement = computed(() => {
    const rect = props.rect
    if (!rect) return null
    const centerX = rect.x + rect.width / 2
    const left = Math.min(Math.max(centerX, 80), window.innerWidth - 80)
    const below = rect.y + rect.height + 12
    const fitsBelow = below + TOOLBAR_HEIGHT <= window.innerHeight

    if (isTouch && fitsBelow) {
        return { left, top: below, above: false }
    }
    return { left, top: Math.max(rect.y - 12, 12), above: true }
})

const style = computed(() => {
    const p = placement.value
    if (!p) return { display: 'none' }
    return { left: `${p.left}px`, top: `${p.top}px` }
})
</script>

<template>
    <div
        v-if="placement"
        class="fixed z-50"
        :class="placement.above ? '-translate-x-1/2 -translate-y-full' : '-translate-x-1/2'"
        :style="style"
    >
        <div class="animate-fade-in">
            <div
                class="flex items-center gap-1 rounded-full bg-(--bg-card) border border-(--border-color) shadow-md px-1 py-1 select-none"
            >
                <slot />
            </div>
        </div>
    </div>
</template>
