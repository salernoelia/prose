<script setup lang="ts">
import Slider from 'primevue/slider'
import type { TextAlign } from '../../ipc/types'

defineProps<{
    fontSize: number
    lineHeight: number
    margin: number
    textAlign: TextAlign
}>()

const emit = defineEmits<{
    (e: 'update:fontSize', value: number): void
    (e: 'update:lineHeight', value: number): void
    (e: 'update:margin', value: number): void
    (e: 'update:textAlign', value: TextAlign): void
}>()

const alignOptions: { value: TextAlign; icon: string; label: string }[] = [
    { value: 'left', icon: 'format_align_left', label: 'Left' },
    { value: 'justify', icon: 'format_align_justify', label: 'Justify' },
    { value: 'center', icon: 'format_align_center', label: 'Center' },
    { value: 'right', icon: 'format_align_right', label: 'Right' },
]
</script>

<template>
    <div class="flex flex-col gap-6 pt-4 border-t border-(--border-color)">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-(--text-secondary)">
            Typography & Layout
        </h2>

        <!-- Font Size Slider -->
        <div class="flex flex-col gap-1.5">
            <div class="flex justify-between items-center text-xs font-medium text-(--text-secondary)">
                <label for="font-size-slider">Font Size</label>
                <span class="text-(--text-tertiary) tabular-nums font-mono text-[11px]">{{ fontSize }}px</span>
            </div>
            <div class="py-1">
                <Slider
                    id="font-size-slider"
                    :modelValue="fontSize"
                    @update:modelValue="(val) => emit('update:fontSize', val as number)"
                    :min="12"
                    :max="48"
                    class="w-full focus-ring-minimal"
                />
            </div>
        </div>

        <!-- Line Spacing Slider -->
        <div class="flex flex-col gap-1.5">
            <div class="flex justify-between items-center text-xs font-medium text-(--text-secondary)">
                <label for="line-height-slider">Line Spacing</label>
                <span class="text-(--text-tertiary) tabular-nums font-mono text-[11px]">{{ lineHeight.toFixed(1) }}x</span>
            </div>
            <div class="py-1">
                <Slider
                    id="line-height-slider"
                    :modelValue="lineHeight"
                    @update:modelValue="(val) => emit('update:lineHeight', val as number)"
                    :min="1.0"
                    :max="3.0"
                    :step="0.1"
                    class="w-full focus-ring-minimal"
                />
            </div>
        </div>

        <!-- Margin Slider -->
        <div class="flex flex-col gap-1.5">
            <div class="flex justify-between items-center text-xs font-medium text-(--text-secondary)">
                <label for="margin-slider">Page Margins</label>
                <span class="text-(--text-tertiary) tabular-nums font-mono text-[11px]">{{ margin.toFixed(1) }}x</span>
            </div>
            <div class="py-1">
                <Slider
                    id="margin-slider"
                    :modelValue="margin"
                    @update:modelValue="(val) => emit('update:margin', val as number)"
                    :min="0.5"
                    :max="3.0"
                    :step="0.1"
                    class="w-full focus-ring-minimal"
                />
            </div>
        </div>

        <!-- Text Alignment -->
        <div class="flex flex-col gap-2">
            <span class="text-xs font-medium text-(--text-secondary)">
                Text Alignment
            </span>
            <div class="grid grid-cols-4 gap-2">
                <button
                    v-for="opt in alignOptions"
                    :key="opt.value"
                    @click="emit('update:textAlign', opt.value)"
                    class="py-2 px-3 rounded-xl border text-xs font-medium flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-xs active:scale-95"
                    :class="textAlign === opt.value
                        ? 'border-(--text-primary) bg-(--bg-card) text-(--text-primary) font-bold ring-1 ring-(--text-primary)'
                        : 'border-(--border-color) bg-(--bg-card) text-(--text-secondary) hover:text-(--text-primary) hover:border-(--border-color-hover)'"
                >
                    <span class="material-symbols-outlined text-base select-none">{{ opt.icon }}</span>
                    <span>{{ opt.label }}</span>
                </button>
            </div>
        </div>
    </div>
</template>
