<script
    setup
    lang="ts"
>
import { useSettings } from '../../composables/useSettings'
import type { Theme, TextAlign } from '../../ipc/types'

defineProps<{
    visible: boolean
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const { theme, fontFamily, fontSize, lineHeight, textAlign } = useSettings()

const alignOptions: { value: TextAlign; icon: string; label: string }[] = [
    { value: 'left', icon: 'format_align_left', label: 'Left' },
    { value: 'justify', icon: 'format_align_justify', label: 'Justify' },
    { value: 'center', icon: 'format_align_center', label: 'Center' },
    { value: 'right', icon: 'format_align_right', label: 'Right' },
]

const themeCards: { label: string; value: Theme; bg: string; fg: string; border: string }[] = [
    { label: 'Light', value: 'light', bg: '#F7EDDA', fg: '#1C1917', border: '#E8DCC8' },
    { label: 'Paper', value: 'paper', bg: '#ffffff', fg: '#111111', border: '#e5e5e5' },
    { label: 'Sepia', value: 'sepia', bg: '#E4D7BE', fg: '#2E2218', border: '#D2C1A5' },
    { label: 'Sepia Dark', value: 'sepia-dark', bg: '#1C1611', fg: '#F7EDDA', border: '#382E25' },
    { label: 'Dark', value: 'dark', bg: '#09332C', fg: '#F7EDDA', border: '#1D4B42' },
    { label: 'OLED', value: 'oled', bg: '#000000', fg: '#F7EDDA', border: '#1e1e1e' },
]

const fontOptions = [
    { label: 'Literata', value: 'Literata' },
    { label: 'Georgia', value: 'Georgia' },
    { label: 'Inter', value: 'Inter' },
    { label: 'Outfit', value: 'Outfit' },
]

const FONT_MIN = 12
const FONT_MAX = 48
const LINE_MIN = 1.0
const LINE_MAX = 3.0

const clamp = (value: number, lo: number, hi: number) => Math.min(Math.max(value, lo), hi)

function stepFont(delta: number) {
    fontSize.value = clamp(fontSize.value + delta, FONT_MIN, FONT_MAX)
}

function stepLine(delta: number) {
    lineHeight.value = clamp(Math.round((lineHeight.value + delta) * 10) / 10, LINE_MIN, LINE_MAX)
}
</script>

<template>
    <!-- Tap-outside backdrop -->
    <div
        v-if="visible"
        class="fixed inset-0 z-40 bg-black/10 backdrop-blur-xs"
        @click="emit('close')"
    ></div>

    <div
        v-if="visible"
        class="fixed left-1/2 -translate-x-1/2 z-50 w-80 max-w-[92vw] animate-fade-in
               bottom-[calc(4.5rem+env(safe-area-inset-bottom,0px))] md:bottom-18"
    >
        <div class="flex flex-col gap-4 rounded-3xl bg-(--bg-card) border border-(--border-color) shadow-xl p-5 font-serif select-none">
            <!-- Header with Close -->
            <div class="flex items-center justify-between">
                <span class="text-xs font-semibold tracking-wide text-(--text-primary)">
                    Reading Appearance
                </span>
                <button
                    @click="emit('close')"
                    class="w-6 h-6 rounded-full flex items-center justify-center text-(--text-tertiary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors cursor-pointer"
                >
                    <span class="material-symbols-outlined text-base">close</span>
                </button>
            </div>

            <!-- Theme Swatches (Apple Books inspired) -->
            <div class="grid grid-cols-3 gap-2">
                <button
                    v-for="tc in themeCards"
                    :key="tc.value"
                    @click="theme = tc.value"
                    class="flex flex-col items-center justify-center py-2 px-1 rounded-xl transition-all cursor-pointer relative"
                    :style="{
                        backgroundColor: tc.bg,
                        color: tc.fg,
                        border: `1.5px solid ${theme === tc.value ? 'var(--accent-color)' : tc.border}`,
                    }"
                >
                    <span class="text-xs font-serif font-bold">Aa</span>
                    <span class="text-[10px] font-sans font-medium opacity-80 mt-0.5">{{ tc.label }}</span>
                </button>
            </div>

            <!-- Typeface Picker -->
            <div class="flex flex-col gap-1.5">
                <span class="text-[10px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary)">
                    Typeface
                </span>
                <div class="grid grid-cols-4 gap-1 bg-(--accent-color-light)/60 p-1 rounded-xl border border-(--border-color)/50">
                    <button
                        v-for="fo in fontOptions"
                        :key="fo.value"
                        @click="fontFamily = fo.value"
                        class="py-1 px-1.5 rounded-lg text-xs transition-all cursor-pointer truncate text-center"
                        :class="fontFamily === fo.value
                            ? 'bg-(--bg-card) text-(--text-primary) font-semibold shadow-xs'
                            : 'text-(--text-secondary) hover:text-(--text-primary)'"
                        :style="{ fontFamily: fo.value }"
                    >
                        {{ fo.label }}
                    </button>
                </div>
            </div>

            <!-- Font Size & Line Spacing Steppers -->
            <div class="grid grid-cols-2 gap-2">
                <!-- Size -->
                <div class="flex items-center justify-between bg-(--accent-color-light)/40 border border-(--border-color)/50 rounded-xl px-2.5 py-1.5">
                    <button
                        @click="stepFont(-1)"
                        :disabled="fontSize <= FONT_MIN"
                        class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-secondary) hover:text-(--text-primary) active:scale-90 transition-all disabled:opacity-20 cursor-pointer"
                        title="Smaller text"
                    >
                        <span class="text-xs font-serif font-semibold">A</span>
                    </button>
                    <span class="text-xs font-medium tabular-nums text-(--text-primary)">{{ fontSize }}</span>
                    <button
                        @click="stepFont(1)"
                        :disabled="fontSize >= FONT_MAX"
                        class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-secondary) hover:text-(--text-primary) active:scale-90 transition-all disabled:opacity-20 cursor-pointer"
                        title="Larger text"
                    >
                        <span class="text-base font-serif font-bold">A</span>
                    </button>
                </div>

                <!-- Spacing -->
                <div class="flex items-center justify-between bg-(--accent-color-light)/40 border border-(--border-color)/50 rounded-xl px-2.5 py-1.5">
                    <button
                        @click="stepLine(-0.1)"
                        :disabled="lineHeight <= LINE_MIN"
                        class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-secondary) hover:text-(--text-primary) active:scale-90 transition-all disabled:opacity-20 cursor-pointer"
                        title="Tighter spacing"
                    >
                        <span class="material-symbols-outlined text-sm">density_medium</span>
                    </button>
                    <span class="text-xs font-medium tabular-nums text-(--text-primary)">{{ lineHeight.toFixed(1) }}</span>
                    <button
                        @click="stepLine(0.1)"
                        :disabled="lineHeight >= LINE_MAX"
                        class="w-7 h-7 rounded-full flex items-center justify-center text-(--text-secondary) hover:text-(--text-primary) active:scale-90 transition-all disabled:opacity-20 cursor-pointer"
                        title="Looser spacing"
                    >
                        <span class="material-symbols-outlined text-sm">density_large</span>
                    </button>
                </div>
            </div>

            <!-- Alignment -->
            <div class="flex items-center justify-between gap-1 bg-(--accent-color-light)/60 p-1 rounded-xl border border-(--border-color)/50">
                <button
                    v-for="opt in alignOptions"
                    :key="opt.value"
                    @click="textAlign = opt.value"
                    :class="[
                        'flex-1 py-1 flex items-center justify-center rounded-lg transition-all duration-100 cursor-pointer',
                        textAlign === opt.value
                            ? 'bg-(--bg-card) text-(--text-primary) shadow-xs'
                            : 'text-(--text-secondary) hover:text-(--text-primary)',
                    ]"
                    :title="opt.label"
                    :aria-label="opt.label"
                >
                    <span class="material-symbols-outlined text-base leading-none">{{ opt.icon }}</span>
                </button>
            </div>
        </div>
    </div>
</template>
