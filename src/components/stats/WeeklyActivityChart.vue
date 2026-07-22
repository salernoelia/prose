<script setup lang="ts">
export interface WeeklyBar {
    date: string;
    label: string;
    totalSeconds: number;
    height: number;
    active: boolean;
}

defineProps<{
    bars: WeeklyBar[];
    todayISO: string;
}>();
</script>

<template>
    <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 mb-3">
        <p class="text-xs font-medium tracking-wider text-(--text-tertiary) mb-4">This week</p>
        <div
            class="flex items-end justify-between gap-1.5"
            style="height: 80px;"
        >
            <div
                v-for="bar in bars"
                :key="bar.date"
                class="flex-1 flex flex-col items-center justify-end gap-1.5"
            >
                <div
                    class="w-full rounded-full transition-all duration-500"
                    :class="[
                        bar.date === todayISO
                            ? 'bg-(--text-primary)'
                            : bar.active
                                ? 'bg-(--text-secondary)'
                                : 'bg-(--border-color)',
                    ]"
                    :style="{ height: bar.active ? bar.height + 'px' : '4px' }"
                ></div>
                <span
                    class="text-[10px] font-medium select-none tabular-nums"
                    :class="bar.date === todayISO ? 'text-(--text-primary) font-semibold' : 'text-(--text-tertiary)'"
                >
                    {{ bar.label }}
                </span>
            </div>
        </div>
    </div>
</template>
