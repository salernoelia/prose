<script setup lang="ts">
import Select from 'primevue/select'
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

const fontOptions = [
    { label: 'Georgia', value: 'Georgia' },
    { label: 'Literata', value: 'Literata' },
    { label: 'Inter', value: 'Inter' },
    { label: 'Outfit', value: 'Outfit' },
]
</script>

<template>
    <div class="flex flex-col gap-6">
        <div class="flex flex-col gap-1.5">
            <label
                for="theme-select"
                class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
            >
                Theme
            </label>
            <Select
                id="theme-select"
                :modelValue="theme"
                @update:modelValue="(val) => emit('update:theme', val as Theme)"
                :options="themeOptions"
                optionLabel="label"
                optionValue="value"
                class="w-full focus-ring-minimal"
            />
        </div>

        <div class="flex flex-col gap-1.5">
            <label
                for="font-family-select"
                class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
            >
                Typeface
            </label>
            <Select
                id="font-family-select"
                :modelValue="fontFamily"
                @update:modelValue="(val) => emit('update:fontFamily', val as string)"
                :options="fontOptions"
                optionLabel="label"
                optionValue="value"
                class="w-full focus-ring-minimal"
            />
        </div>

        <div class="mt-2 flex flex-col gap-1.5">
            <span class="text-xs font-medium uppercase tracking-wider text-(--text-tertiary)">Preview</span>
            <div
                class="overflow-hidden border border-(--border-color) rounded-lg bg-(--bg-card) shadow-inner"
                :style="{ height: '220px' }"
            >
                <div
                    class="h-full overflow-hidden select-none p-6"
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
                        reading, but it had no pictures or conversations in it.
                    </p>
                </div>
            </div>
        </div>
    </div>
</template>
