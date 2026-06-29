<script
    setup
    lang="ts"
>
import { computed } from 'vue'
import type { ViewportRect } from '../../readers'

const props = defineProps<{
    rect: ViewportRect | null
}>()

// Float the toolbar centered just above the target rect, nudged back inside the
// viewport horizontally so it never clips off-screen on a near-edge selection.
const style = computed(() => {
    const rect = props.rect
    if (!rect) return { display: 'none' }
    const centerX = rect.x + rect.width / 2
    const clampedX = Math.min(Math.max(centerX, 80), window.innerWidth - 80)
    const top = Math.max(rect.y - 12, 12)
    return {
        left: `${clampedX}px`,
        top: `${top}px`,
    }
})
</script>

<template>
    <div
        v-if="rect"
        class="fixed z-50 -translate-x-1/2 -translate-y-full animate-fade-in"
        :style="style"
    >
        <div
            class="flex items-center gap-1 rounded-full bg-(--bg-card) border border-(--border-color) shadow-md px-1 py-1 select-none"
        >
            <slot />
        </div>
    </div>
</template>
