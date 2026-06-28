<script
    setup
    lang="ts"
>
import { useSettings } from '../composables/useSettings'
import Select from 'primevue/select'
import Slider from 'primevue/slider'
import type { Theme } from '../ipc/types'

const { settings, loaded, theme, fontFamily, fontSize, lineHeight, margin, clickZoneSize } =
    useSettings()

const themeOptions = [
    { label: 'Light', value: 'light' as Theme },
    { label: 'Dark', value: 'dark' as Theme },
    { label: 'Sepia', value: 'sepia' as Theme },
]

const fontOptions = [
    { label: 'Literata', value: 'Literata' },
    { label: 'Georgia', value: 'Georgia' },
    { label: 'Inter', value: 'Inter' },
    { label: 'Outfit', value: 'Outfit' },
]
</script>

<template>
    <div class="w-full animate-fade-in">
        <!-- Typography-driven Header (No Icons) -->
        <header class="pb-6">
            <h1 class="text-xl font-semibold tracking-tight text-(--text-primary)">Settings</h1>
        </header>

        <!-- Form Controls (No Icons, Minimal Labels) -->
        <div
            v-if="loaded"
            class="flex flex-col gap-6"
        >
            <!-- Theme Selection -->
            <div class="flex flex-col gap-1.5">
                <label
                    for="theme-select"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Theme
                </label>
                <Select
                    id="theme-select"
                    v-model="theme"
                    :options="themeOptions"
                    optionLabel="label"
                    optionValue="value"
                    class="w-full focus-ring-minimal"
                />
            </div>

            <!-- Font Family Selection -->
            <div class="flex flex-col gap-1.5">
                <label
                    for="font-family-select"
                    class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
                >
                    Typeface
                </label>
                <Select
                    id="font-family-select"
                    v-model="fontFamily"
                    :options="fontOptions"
                    optionLabel="label"
                    optionValue="value"
                    class="w-full focus-ring-minimal"
                />
            </div>

            <!-- Typography Preview -->
            <div class="mt-4 flex flex-col gap-1.5">
                <span class="text-xs font-medium uppercase tracking-wider text-(--text-tertiary)">Preview</span>
                <div
                    class="overflow-hidden border border-(--border-color) rounded-lg bg-(--bg-card) shadow-inner"
                    :style="{ height: '300px' }"
                >
                    <div
                        class="h-full overflow-y-auto select-none p-6"
                        :style="{
                            fontFamily: settings.fontFamily,
                            fontSize: settings.fontSize + 'px',
                            lineHeight: settings.lineHeight,
                            paddingLeft: settings.margin * 12 + 'px',
                            paddingRight: settings.margin * 12 + 'px',
                        }"
                    >
                        <h2 class="font-semibold mb-2 text-[1.1em] tracking-tight">
                            Chapter I: Down the Rabbit-Hole
                        </h2>
                        <p class="text-left text-[0.95em]">
                            Alice was beginning to get very tired of sitting by her sister on the bank, and of
                            having nothing to do: once or twice she had peeped into the book her sister was
                            reading, but it had no pictures or conversations in it, “and what is the use of a
                            book,” thought Alice “without pictures or conversations?”
                        </p>
                    </div>
                </div>
            </div>

            <!-- Font Size Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="font-size-slider">Size</label>
                    <span class="text-(--text-tertiary)">{{ fontSize }}px</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="font-size-slider"
                        v-model="fontSize"
                        :min="12"
                        :max="48"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Line Height Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="line-height-slider">Spacing</label>
                    <span class="text-(--text-tertiary)">{{ lineHeight.toFixed(1) }}x</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="line-height-slider"
                        v-model="lineHeight"
                        :min="1.0"
                        :max="3.0"
                        :step="0.1"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Margin Slider -->
            <div class="flex flex-col gap-1.5">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="margin-slider">Margin</label>
                    <span class="text-(--text-tertiary)">{{ margin.toFixed(1) }}x</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="margin-slider"
                        v-model="margin"
                        :min="0.5"
                        :max="3.0"
                        :step="0.1"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>

            <!-- Click Zone Slider -->
            <div class="flex flex-col gap-1.5 pb-8">
                <div
                    class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                    <label for="click-zone-slider">Page-Turn Zone</label>
                    <span class="text-(--text-tertiary)">{{ clickZoneSize }}%</span>
                </div>
                <div class="py-2">
                    <Slider
                        id="click-zone-slider"
                        v-model="clickZoneSize"
                        :min="10"
                        :max="45"
                        class="w-full focus-ring-minimal"
                    />
                </div>
            </div>


        </div>

        <!-- Loading State -->
        <div
            v-else
            class="flex flex-col items-center justify-center py-16 gap-3"
        >
            <div class="w-6 h-6 rounded-full border border-(--border-color) border-t-(--accent-color) animate-spin">
            </div>
            <p class="text-xs text-(--text-secondary) font-medium">Loading</p>
        </div>
    </div>
</template>
