<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'

const emit = defineEmits<{
    (e: 'prev'): void
    (e: 'next'): void
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
    // The ePub renderer (foliate) handles its own panning, swiping, and text
    // selection inside its iframe; stepping in here would double-turn the page
    // and fight selection. Only drive paging for formats that do not, like PDF.
    const target = e.target as HTMLElement | null
    if (target?.closest('foliate-view')) return

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
    <!-- Renders nothing visible on purpose. A covering overlay would block text
         selection near the edges and turn the page on finger lift. Tap paging is
         driven by `renderer-click`, which the reader maps to a turn zone by the
         tap's x; swipes on fixed-layout pages are handled by the listeners above. -->
    <div
        class="hidden"
        aria-hidden="true"
    ></div>
</template>
