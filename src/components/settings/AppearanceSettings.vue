<script setup lang="ts">
import type { Theme, SettingsDto } from '../../ipc/types'

defineProps<{
    theme: Theme
    fontFamily: string
    settings: SettingsDto
}>()

const emit = defineEmits<{
    (e: 'update:theme', value: Theme): void
    (e: 'update:fontFamily', value: string): void
}>()

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
</script>

<template>
    <div class="flex flex-col gap-6">
        <!-- Live Book Preview Card (Always visible at the top of settings) -->
        <div class="flex flex-col gap-2">
            <span class="text-[11px] font-sans font-medium uppercase tracking-widest text-(--text-tertiary) select-none">
                Live Preview
            </span>
            <div
                class="overflow-hidden border border-(--border-color) rounded-2xl bg-(--bg-card) shadow-sm transition-all duration-300"
                :style="{ minHeight: '190px' }"
            >
                <div
                    class="h-full select-none p-6 transition-all duration-200"
                    :style="{
                        fontFamily: settings.fontFamily,
                        fontSize: `${settings.fontSize}px`,
                        lineHeight: settings.lineHeight,
                        paddingLeft: `${Math.max(16, settings.margin * 16)}px`,
                        paddingRight: `${Math.max(16, settings.margin * 16)}px`,
                        textAlign: settings.textAlign,
                    }"
                >
                    <h2 class="font-semibold mb-3 text-[1.15em] tracking-tight font-serif">
                        Chapter I: Down the Rabbit-Hole
                    </h2>
                    <p class="text-[0.95em] leading-relaxed">
                        Alice was beginning to get very tired of sitting by her sister on the bank, and of
                        having nothing to do: once or twice she had peeped into the book her sister was
                        reading, but it had no pictures or conversations in it, &ldquo;and what is the use of a book,&rdquo;
                        thought Alice, &ldquo;without pictures or conversations?&rdquo;
                    </p>
                </div>
            </div>
        </div>

        <!-- Theme Selection Grid -->
        <div class="flex flex-col gap-2">
            <span class="text-[11px] font-sans font-medium uppercase tracking-widest text-(--text-tertiary) select-none">
                Theme
            </span>
            <div class="grid grid-cols-3 sm:grid-cols-6 gap-2">
                <button
                    v-for="tc in themeCards"
                    :key="tc.value"
                    @click="emit('update:theme', tc.value)"
                    class="flex flex-col items-center justify-center py-2.5 px-2 rounded-xl transition-all cursor-pointer shadow-xs active:scale-95"
                    :style="{
                        backgroundColor: tc.bg,
                        color: tc.fg,
                        border: `1.5px solid ${theme === tc.value ? 'var(--accent-color)' : tc.border}`,
                    }"
                >
                    <span class="text-sm font-serif font-bold">Aa</span>
                    <span class="text-[10px] font-sans font-medium opacity-80 mt-1">{{ tc.label }}</span>
                </button>
            </div>
        </div>

        <!-- Typeface Selection Grid -->
        <div class="flex flex-col gap-2">
            <span class="text-[11px] font-sans font-medium uppercase tracking-widest text-(--text-tertiary) select-none">
                Typeface
            </span>
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                <button
                    v-for="fo in fontOptions"
                    :key="fo.value"
                    @click="emit('update:fontFamily', fo.value)"
                    class="py-2.5 px-3 rounded-xl border text-xs transition-all cursor-pointer text-center active:scale-95 shadow-xs"
                    :class="fontFamily === fo.value
                        ? 'border-(--text-primary) bg-(--bg-card) text-(--text-primary) font-bold ring-1 ring-(--text-primary)'
                        : 'border-(--border-color) bg-(--bg-card) text-(--text-secondary) hover:text-(--text-primary) hover:border-(--border-color-hover)'"
                    :style="{ fontFamily: fo.value }"
                >
                    {{ fo.label }}
                </button>
            </div>
        </div>
    </div>
</template>
