<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

defineProps<{
    /** Width of each side turn-zone, in viewport-width units. */
    zoneSize: number
}>()

const emit = defineEmits<{
    (e: 'prev'): void
    (e: 'next'): void
    (e: 'toggle'): void
}>()

let touchstartX = 0
let touchstartY = 0
let touchstartTime = 0

function handleTouchStart(e: TouchEvent) {
    touchstartX = e.changedTouches[0].screenX
    touchstartY = e.changedTouches[0].screenY
    touchstartTime = Date.now()
}

function handleTouchEnd(e: TouchEvent) {
    const touchendX = e.changedTouches[0].screenX
    const touchendY = e.changedTouches[0].screenY
    const deltaTime = Date.now() - touchstartTime

    // Only recognize swipe if it is quick (under 500ms) to avoid issues with slow panning or text selection
    if (deltaTime < 500) {
        const deltaX = touchendX - touchstartX
        const deltaY = touchendY - touchstartY

        // Horizontal swipe (next page: swipe left, prev page: swipe right)
        // Require horizontal distance of at least 50px and vertical deviation under 40px
        if (deltaX < -50 && Math.abs(deltaY) < 40) {
            emit('next')
        } else if (deltaX > 50 && Math.abs(deltaY) < 40) {
            emit('prev')
        }
    }
}

onMounted(() => {
    window.addEventListener('touchstart', handleTouchStart, { passive: true })
    window.addEventListener('touchend', handleTouchEnd, { passive: true })
})

onUnmounted(() => {
    window.removeEventListener('touchstart', handleTouchStart)
    window.removeEventListener('touchend', handleTouchEnd)
})
</script>

<template>
    <!-- LEFT page turn click zone -->
    <div
        @click.stop="emit('prev')"
        class="fixed left-0 top-0 bottom-0 z-20 bg-transparent transition-all duration-200"
        :style="{ width: zoneSize + 'vw' }"
        style="cursor: w-resize"
        title="Previous Page"
    >
        <div class="w-1 h-full bg-(--accent-color) opacity-0 hover:opacity-5 transition-opacity"></div>
    </div>

    <!-- RIGHT page turn click zone -->
    <div
        @click.stop="emit('next')"
        class="fixed right-0 top-0 bottom-0 z-20 bg-transparent transition-all duration-200"
        :style="{ width: zoneSize + 'vw' }"
        style="cursor: e-resize"
        title="Next Page"
    >
        <div class="w-1 h-full bg-(--accent-color) right-0 absolute opacity-0 hover:opacity-5 transition-opacity">
        </div>
    </div>


</template>
