<script
    setup
    lang="ts"
>
import { computed, onUnmounted, ref, watch } from 'vue'
import type { ViewportRect } from '../../readers'

const props = defineProps<{
    rect: ViewportRect | null
}>()

// Approximate toolbar height, used only to decide whether a placement would run
// off the top or bottom of the viewport.
const TOOLBAR_HEIGHT = 48

// The toolbar's measured half-width, kept current so the horizontal clamp uses
// the real size. The set of actions (and so the width) changes with the
// selection, and a stale estimate would let a wide toolbar clip off the edge.
const toolbarEl = ref<HTMLDivElement | null>(null)
const halfWidth = ref(120)
let observer: ResizeObserver | null = null

watch(toolbarEl, (el) => {
    observer?.disconnect()
    observer = null
    if (!el || typeof ResizeObserver === 'undefined') return
    observer = new ResizeObserver(() => {
        const width = el.offsetWidth
        if (width) halfWidth.value = width / 2
    })
    observer.observe(el)
})

onUnmounted(() => observer?.disconnect())

// Float the toolbar over the target rect, nudged back inside the viewport
// horizontally so it never clips off-screen on a near-edge selection. Always
// prefer placing below the selection: the OS selection menu (Copy / Look Up /
// Translate) sits above the selection, and a toolbar there would be covered.
// Fall back to above only when there is no room below.
const placement = computed(() => {
    const rect = props.rect
    if (!rect) return null
    const centerX = rect.x + rect.width / 2
    const margin = halfWidth.value + 8
    const left = Math.min(Math.max(centerX, margin), window.innerWidth - margin)

    const below = rect.y + rect.height + 12
    const fitsBelow = below + TOOLBAR_HEIGHT <= window.innerHeight

    if (fitsBelow) {
        return { left, top: below, above: false }
    }
    return { left, top: rect.y - 12, above: true }
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
                ref="toolbarEl"
                class="flex items-center gap-0.5 rounded-full bg-(--bg-card) border border-(--border-color) shadow-md px-1 py-1 select-none"
            >
                <slot />
            </div>
        </div>
    </div>
</template>
