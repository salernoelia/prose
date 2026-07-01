<script
    setup
    lang="ts"
>
import { useSettings } from '../../composables/useSettings'
import Select from 'primevue/select'
import type { Theme } from '../../ipc/types'

defineProps<{
    visible: boolean
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const { theme, fontSize, lineHeight } = useSettings()

const themeOptions = [
    { label: 'Light', value: 'light' as Theme },
    { label: 'Paper', value: 'paper' as Theme },
    { label: 'Dark', value: 'dark' as Theme },
    { label: 'OLED Black', value: 'oled' as Theme },
    { label: 'Night', value: 'night' as Theme },
    { label: 'Sepia', value: 'sepia' as Theme },
    { label: 'Sepia Dark', value: 'sepia-dark' as Theme },
    { label: 'E-Ink Light', value: 'eink' as Theme },
    { label: 'E-Ink Dark', value: 'eink-dark' as Theme },
]

// Match the ranges and steps used by the full settings view.
const FONT_MIN = 12
const FONT_MAX = 48
const LINE_MIN = 1.0
const LINE_MAX = 3.0

const clamp = (value: number, lo: number, hi: number) => Math.min(Math.max(value, lo), hi)

function stepFont(delta: number) {
    fontSize.value = clamp(fontSize.value + delta, FONT_MIN, FONT_MAX)
}

function stepLine(delta: number) {
    // Round to one decimal to avoid floating-point drift across steps.
    lineHeight.value = clamp(Math.round((lineHeight.value + delta) * 10) / 10, LINE_MIN, LINE_MAX)
}
</script>

<template>
    <!-- Tap-outside backdrop -->
    <div
        v-if="visible"
        class="fixed inset-0 z-40"
        @click="emit('close')"
    ></div>

    <div
        v-if="visible"
        class="fixed left-1/2 -translate-x-1/2 z-50 w-64 animate-fade-in
               bottom-[calc(4.5rem+env(safe-area-inset-bottom,0px))] md:bottom-20"
    >
        <div class="flex flex-col gap-3 rounded-2xl bg-(--bg-card) border border-(--border-color) shadow-md p-4">
            <!-- Theme -->
            <div class="flex flex-col gap-1.5">
                <span class="text-[11px] font-medium uppercase tracking-wider text-(--text-tertiary) select-none">
                    Theme
                </span>
                <Select
                    v-model="theme"
                    :options="themeOptions"
                    optionLabel="label"
                    optionValue="value"
                    class="w-full focus-ring-minimal"
                />
            </div>

            <!-- Text size -->
            <div class="flex items-center justify-between gap-2">
                <span class="text-[11px] font-medium uppercase tracking-wider text-(--text-tertiary) select-none">
                    Text size
                </span>
                <div class="flex items-center gap-1.5">
                    <button
                        @click="stepFont(-1)"
                        :disabled="fontSize <= FONT_MIN"
                        class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal disabled:opacity-20 disabled:active:scale-100"
                        title="Smaller text"
                        aria-label="Smaller text"
                    >
                        <span class="material-symbols-outlined text-base">remove</span>
                    </button>
                    <span class="w-9 text-center text-sm tabular-nums text-(--text-primary) select-none">{{ fontSize }}</span>
                    <button
                        @click="stepFont(1)"
                        :disabled="fontSize >= FONT_MAX"
                        class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal disabled:opacity-20 disabled:active:scale-100"
                        title="Larger text"
                        aria-label="Larger text"
                    >
                        <span class="material-symbols-outlined text-base">add</span>
                    </button>
                </div>
            </div>

            <!-- Line spacing -->
            <div class="flex items-center justify-between gap-2">
                <span class="text-[11px] font-medium uppercase tracking-wider text-(--text-tertiary) select-none">
                    Line spacing
                </span>
                <div class="flex items-center gap-1.5">
                    <button
                        @click="stepLine(-0.1)"
                        :disabled="lineHeight <= LINE_MIN"
                        class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal disabled:opacity-20 disabled:active:scale-100"
                        title="Tighter line spacing"
                        aria-label="Tighter line spacing"
                    >
                        <span class="material-symbols-outlined text-base">remove</span>
                    </button>
                    <span class="w-9 text-center text-sm tabular-nums text-(--text-primary) select-none">{{ lineHeight.toFixed(1) }}</span>
                    <button
                        @click="stepLine(0.1)"
                        :disabled="lineHeight >= LINE_MAX"
                        class="flex items-center justify-center w-8 h-8 rounded-full text-(--text-secondary) hover:text-(--text-primary) transition-all duration-100 active:scale-90 active:opacity-80 focus-ring-minimal disabled:opacity-20 disabled:active:scale-100"
                        title="Looser line spacing"
                        aria-label="Looser line spacing"
                    >
                        <span class="material-symbols-outlined text-base">add</span>
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>
