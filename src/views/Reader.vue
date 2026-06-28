<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useSettings } from '../composables/useSettings'
import type { Book } from './Library.vue'

const props = defineProps<{
  book: Book
}>()

const emit = defineEmits<{
  (e: 'back-to-library'): void
}>()

const { settings, clickZoneSize } = useSettings()

const showDock = ref(true)
const isBookmarked = ref(false)

const currentPageIndex = ref(0)
const pages = ref<string[]>([])

const mainBodyText = ref<HTMLDivElement | null>(null)
const displayContainer = ref<HTMLDivElement | null>(null)
const measuringContainer = ref<HTMLDivElement | null>(null)

// Reset page index when book changes
watch(() => props.book.id, () => {
  currentPageIndex.value = 0
})

// Dynamic reflow pagination algorithm
// Binary searches the text segments to ensure content NEVER overflows the page boundaries
const repaginate = async () => {
  await nextTick()
  const measureEl = measuringContainer.value
  const displayEl = displayContainer.value
  
  if (!measureEl || !displayEl) return

  // Available height for text content (flex-1 height)
  const maxHeight = displayEl.clientHeight
  
  // If the browser hasn't calculated the container size yet, wait and retry
  if (maxHeight < 40) {
    setTimeout(repaginate, 50)
    return
  }

  const fullText = props.book.text
  const calculatedPages: string[] = []
  let startIndex = 0

  // Helper to check if a specific chunk overflows the clientHeight
  const checkOverflow = (content: string) => {
    measureEl.textContent = content
    return measureEl.scrollHeight > maxHeight
  }

  while (startIndex < fullText.length) {
    let low = 0
    let high = fullText.length - startIndex
    let bestEnd = startIndex

    // Binary search to find the maximum fit
    while (low <= high) {
      const mid = Math.floor((low + high) / 2)
      const testContent = fullText.slice(startIndex, startIndex + mid)
      
      if (testContent && checkOverflow(testContent)) {
        high = mid - 1
      } else {
        bestEnd = startIndex + mid
        low = mid + 1
      }
    }

    // Back off to the last space or newline to prevent chopping words
    if (bestEnd < fullText.length) {
      let tempEnd = bestEnd
      while (tempEnd > startIndex && fullText[tempEnd] !== ' ' && fullText[tempEnd] !== '\n') {
        tempEnd--
      }
      if (tempEnd > startIndex) {
        bestEnd = tempEnd
      }
    }

    const chunk = fullText.slice(startIndex, bestEnd).trim()
    if (chunk) {
      calculatedPages.push(chunk)
    } else {
      // Fallback if a single word is wider/taller than the container (e.g. extreme font size)
      // Grab at least 1 character to guarantee forward progress and avoid infinite loops
      calculatedPages.push(fullText.slice(startIndex, startIndex + 1))
      bestEnd = startIndex + 1
    }

    startIndex = bestEnd
  }

  pages.value = calculatedPages
  
  // Bound check the index
  if (currentPageIndex.value >= pages.value.length) {
    currentPageIndex.value = Math.max(0, pages.value.length - 1)
  }
}

// Attach listeners for screen resizing and settings updates
onMounted(() => {
  repaginate()
  window.addEventListener('resize', repaginate)
})

onUnmounted(() => {
  window.removeEventListener('resize', repaginate)
})

// Repaginate reactively whenever text or settings change
watch(
  [
    () => props.book.text,
    () => settings.value.fontSize,
    () => settings.value.lineHeight,
    () => settings.value.margin,
    () => settings.value.fontFamily
  ],
  () => {
    repaginate()
  }
)

const totalPages = computed(() => pages.value.length)
const currentPageContent = computed(() => pages.value[currentPageIndex.value] || '')

const progressPercentage = computed(() => {
  if (totalPages.value <= 1) return 100
  return Math.round((currentPageIndex.value / (totalPages.value - 1)) * 100)
})

function prevPage() {
  if (currentPageIndex.value > 0) {
    currentPageIndex.value--
  }
}

function nextPage() {
  if (currentPageIndex.value < totalPages.value - 1) {
    currentPageIndex.value++
  }
}

function toggleDock() {
  showDock.value = !showDock.value
}

function showOutlineAlert() {
  alert('Outline navigation will display the book Table of Contents in the future.')
}
</script>

<template>
  <div class="w-full relative h-full flex flex-col justify-between select-none">
    
    <!-- LEFT page turn click zone -->
    <div 
      @click.stop="prevPage" 
      class="fixed left-0 top-0 bottom-0 z-20 bg-transparent transition-all duration-200"
      :style="{ width: clickZoneSize + 'vw' }"
      style="cursor: w-resize;"
      title="Previous Page"
    >
      <div class="w-1 h-full bg-[var(--accent-color)] opacity-0 hover:opacity-5 transition-opacity"></div>
    </div>

    <!-- RIGHT page turn click zone -->
    <div 
      @click.stop="nextPage" 
      class="fixed right-0 top-0 bottom-0 z-20 bg-transparent transition-all duration-200"
      :style="{ width: clickZoneSize + 'vw' }"
      style="cursor: e-resize;"
      title="Next Page"
    >
      <div class="w-1 h-full bg-[var(--accent-color)] right-0 absolute opacity-0 hover:opacity-5 transition-opacity"></div>
    </div>

    <!-- CENTER menu toggle zone (between left & right zones) -->
    <div 
      @click="toggleDock" 
      class="fixed top-0 bottom-0 z-10 bg-transparent cursor-pointer"
      :style="{ left: clickZoneSize + 'vw', right: clickZoneSize + 'vw' }"
      title="Toggle Controls"
    ></div>

    <!-- Non-Scrolling Reading Canvas (Overflow hidden, flex-1, with fade-in) -->
    <div 
      class="relative z-0 w-full flex-1 overflow-hidden select-text transition-all duration-300 flex flex-col justify-between animate-fade-in"
      :style="{
        fontFamily: settings.fontFamily,
        fontSize: settings.fontSize + 'px',
        lineHeight: settings.lineHeight,
        paddingLeft: (settings.margin * 1.5) + 'rem',
        paddingRight: (settings.margin * 1.5) + 'rem'
      }"
    >
      <!-- Book Header Info (Subtle) -->
      <header class="mb-3 pb-2 border-b border-[var(--border-color)] flex justify-between items-center text-xs text-[var(--text-tertiary)] select-none">
        <span class="truncate pr-4">{{ book.title }}</span>
        <span>{{ book.author }}</span>
      </header>

      <!-- Main Body Text Canvas wrapper (stretched, relative container) -->
      <div 
        ref="displayContainer"
        class="flex-1 overflow-hidden flex flex-col justify-start text-left pointer-events-auto relative mb-6"
      >
        <!-- Visible Content Panel -->
        <div ref="mainBodyText" class="flex-1 overflow-hidden">
          <p class="leading-relaxed text-left whitespace-pre-wrap">
            {{ currentPageContent }}
          </p>
        </div>

        <!-- Hidden Measuring Container (Identical fonts, matches bounds via absolute inset-0) -->
        <div 
          ref="measuringContainer"
          class="absolute inset-0 pointer-events-none opacity-0 invisible overflow-hidden whitespace-pre-wrap leading-relaxed text-left"
        ></div>
      </div>
    </div>

    <!-- Floating Dock Card (Compact, Icon-based and Mobile-friendly) -->
    <div 
      class="fixed bottom-3 left-1/2 -translate-x-1/2 z-50 transition-all duration-300 ease-in-out pointer-events-auto"
      :class="showDock ? 'translate-y-0 opacity-100' : 'translate-y-16 opacity-0 pointer-events-none'"
    >
      <!-- Small Border Card (Rounded Pill) -->
      <div class="bg-[var(--bg-card)] border border-[var(--border-color)] rounded-full px-4 py-2 shadow-md flex items-center gap-4">
        
        <!-- Back to Library -->
        <button 
          @click="emit('back-to-library')"
          class="flex items-center justify-center w-8 h-8 rounded-full text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors focus-ring-minimal"
          title="Back to Library"
          aria-label="Back to Library"
        >
          <span class="material-symbols-outlined text-xl leading-none select-none">arrow_back</span>
        </button>

        <span class="w-px h-4 bg-[var(--border-color)]"></span>

        <!-- Outline / TOC (Mock action) -->
        <button 
          @click="showOutlineAlert"
          class="flex items-center justify-center w-8 h-8 rounded-full text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors focus-ring-minimal"
          title="Table of Contents"
          aria-label="Table of Contents"
        >
          <span class="material-symbols-outlined text-xl leading-none select-none">toc</span>
        </button>

        <!-- Bookmark Toggler -->
        <button 
          @click="isBookmarked = !isBookmarked"
          class="flex items-center justify-center w-8 h-8 rounded-full transition-colors focus-ring-minimal"
          :class="isBookmarked ? 'text-[var(--accent-color)] bg-[var(--accent-color-light)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'"
          title="Toggle Bookmark"
          aria-label="Toggle Bookmark"
        >
          <span class="material-symbols-outlined text-xl leading-none select-none">
            {{ isBookmarked ? 'bookmark' : 'bookmark_border' }}
          </span>
        </button>

        <span class="w-px h-4 bg-[var(--border-color)]"></span>

        <!-- Page turn indicators in dock -->
        <div class="flex items-center gap-2 text-xs text-[var(--text-secondary)] select-none">
          <button 
            @click="prevPage" 
            :disabled="currentPageIndex === 0"
            class="flex items-center justify-center w-6 h-6 rounded-full disabled:opacity-20 hover:text-[var(--text-primary)]"
          >
            <span class="material-symbols-outlined text-base">chevron_left</span>
          </button>
          <span>{{ progressPercentage }}%</span>
          <button 
            @click="nextPage" 
            :disabled="currentPageIndex === totalPages - 1"
            class="flex items-center justify-center w-6 h-6 rounded-full disabled:opacity-20 hover:text-[var(--text-primary)]"
          >
            <span class="material-symbols-outlined text-base">chevron_right</span>
          </button>
        </div>

        <span class="w-px h-4 bg-[var(--border-color)]"></span>

        <!-- Hide Dock Action -->
        <button 
          @click="showDock = false"
          class="flex items-center justify-center w-8 h-8 rounded-full text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors focus-ring-minimal"
          title="Hide Controls"
          aria-label="Hide Controls"
        >
          <span class="material-symbols-outlined text-xl leading-none select-none">visibility_off</span>
        </button>

      </div>
    </div>

    <!-- Small controls restoration tab (displays when dock is hidden) -->
    <div 
      class="fixed bottom-3 left-1/2 -translate-x-1/2 z-50 transition-all duration-300 ease-in-out pointer-events-auto"
      :class="!showDock ? 'translate-y-0 opacity-100' : 'translate-y-8 opacity-0 pointer-events-none'"
    >
      <button 
        @click="showDock = true"
        class="flex items-center justify-center w-8 h-8 rounded-full bg-[var(--bg-card)] border border-[var(--border-color)] shadow-sm text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors focus-ring-minimal"
        title="Show Controls"
        aria-label="Show Controls"
      >
        <span class="material-symbols-outlined text-lg leading-none select-none">menu</span>
      </button>
    </div>

  </div>
</template>

<style scoped>
/* West/East resize cursor overrides */
cursor-w {
  cursor: w-resize;
}
cursor-e {
  cursor: e-resize;
}
</style>
