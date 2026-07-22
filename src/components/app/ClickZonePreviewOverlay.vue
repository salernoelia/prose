<script setup lang="ts">
defineProps<{
    visible: boolean;
    clickZoneSize: number;
}>();
</script>

<template>
    <div
        v-if="visible"
        class="fixed inset-0 z-50 pointer-events-none transition-all duration-300 animate-fade-in"
    >
        <div
            class="absolute left-0 top-0 bottom-0 bg-red-500/10 border-r border-dashed border-red-500/30 flex items-center justify-center transition-all duration-150"
            :style="{ width: clickZoneSize + 'vw' }"
        >
            <span class="text-[10px] uppercase tracking-widest font-semibold text-white px-2 py-1 rounded select-none shadow">
                Prev
            </span>
        </div>
        <div
            class="absolute right-0 top-0 bottom-0 bg-red-500/10 border-l border-dashed border-red-500/30 flex items-center justify-center transition-all duration-150"
            :style="{ width: clickZoneSize + 'vw' }"
        >
            <span class="text-[10px] uppercase tracking-widest font-semibold text-white px-2 py-1 rounded select-none shadow">
                Next
            </span>
        </div>
    </div>
</template>
