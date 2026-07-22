<script setup lang="ts">
import type { CSSProperties } from 'vue';

defineProps<{
    visible: boolean;
    dragging: boolean;
    active: boolean;
    progress: number;
    indicatorStyle: CSSProperties | Record<string, string | number>;
}>();
</script>

<template>
    <div
        v-show="visible"
        class="pointer-events-none fixed top-0 left-1/2 z-40"
        :class="dragging ? '' : 'transition-all duration-300 ease-out'"
        :style="indicatorStyle"
    >
        <div class="w-9 h-9 rounded-full bg-(--bg-card) border border-(--border-color) shadow-md flex items-center justify-center">
            <span
                class="material-symbols-outlined text-lg text-(--text-secondary) select-none"
                :class="{ 'animate-spin': active }"
                :style="active ? undefined : { transform: `rotate(${progress * 180}deg)` }"
            >sync</span>
        </div>
    </div>
</template>
