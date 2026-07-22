<script setup lang="ts">
import Select from 'primevue/select'
import Slider from 'primevue/slider'
import { TRANSLATION_LANGUAGES } from '../../lib/externalLookup'

defineProps<{
    clickZoneSize: number
    translationLanguage: string
}>()

const emit = defineEmits<{
    (e: 'update:clickZoneSize', value: number): void
    (e: 'update:translationLanguage', value: string): void
}>()

const translationOptions = [...TRANSLATION_LANGUAGES]
</script>

<template>
    <div class="flex flex-col gap-6 pt-4 border-t border-(--border-color)">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-(--text-secondary)">
            Reader Controls
        </h2>

        <div class="flex flex-col gap-1.5">
            <div class="flex justify-between items-center text-xs font-medium uppercase tracking-wider text-(--text-secondary)">
                <label for="click-zone-slider">Page Turn Zone Width</label>
                <span class="text-(--text-tertiary)">{{ clickZoneSize }}%</span>
            </div>
            <div class="py-2">
                <Slider
                    id="click-zone-slider"
                    :modelValue="clickZoneSize"
                    @update:modelValue="(val) => emit('update:clickZoneSize', val as number)"
                    :min="5"
                    :max="45"
                    :step="1"
                    class="w-full focus-ring-minimal"
                />
            </div>
            <p class="text-xs text-(--text-tertiary)">
                Tapping within this percentage from the left or right edge turns pages. Tapping the middle area toggles the dock.
            </p>
        </div>

        <div class="flex flex-col gap-1.5">
            <label
                for="translation-lang-select"
                class="text-xs font-medium uppercase tracking-wider text-(--text-secondary)"
            >
                Translation Target Language
            </label>
            <Select
                id="translation-lang-select"
                :modelValue="translationLanguage"
                @update:modelValue="(val) => emit('update:translationLanguage', val as string)"
                :options="translationOptions"
                optionLabel="label"
                optionValue="value"
                class="w-full focus-ring-minimal"
            />
        </div>
    </div>
</template>
