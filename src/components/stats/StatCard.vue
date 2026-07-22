<script setup lang="ts">
defineProps<{
    label: string;
    icon?: string;
    value: string | number;
    unit?: string;
    subtitle?: string;
}>();
</script>

<template>
    <div class="bg-(--bg-card) border border-(--border-color) rounded-2xl p-4 flex flex-col gap-1">
        <div class="flex items-center gap-1.5 text-(--text-tertiary) text-xs font-medium tracking-wider">
            <span
                v-if="icon"
                class="material-symbols-outlined text-sm select-none"
            >{{ icon }}</span>
            <span>{{ label }}</span>
        </div>
        <div class="flex items-end gap-1.5 mt-1">
            <span class="text-3xl font-semibold tabular-nums text-(--text-primary) leading-none">
                {{ value }}
            </span>
            <span
                v-if="unit"
                class="text-sm text-(--text-secondary) mb-0.5"
            >{{ unit }}</span>
        </div>
        <p
            v-if="subtitle"
            class="text-xs text-(--text-tertiary) mt-0.5"
        >
            {{ subtitle }}
        </p>
    </div>
</template>
