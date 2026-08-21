<script setup lang="ts">
defineProps<{
    label: string;
    icon?: string;
    value: string | number;
    unit?: string;
    subtitle?: string;
    badge?: string;
    variant?: 'default' | 'accent' | 'warm';
}>();
</script>

<template>
    <div
        class="border rounded-2xl p-4 sm:p-5 flex flex-col justify-between shadow-xs transition-all relative overflow-hidden"
        :class="[
            variant === 'accent'
                ? 'bg-(--accent-color) text-(--accent-ink) border-transparent'
                : variant === 'warm'
                    ? 'bg-(--accent-color-light)/30 border-(--border-color) text-(--text-primary)'
                    : 'bg-(--bg-card) border-(--border-color) text-(--text-primary)'
        ]"
    >
        <div class="flex items-center justify-between gap-2 mb-2">
            <div
                class="flex items-center gap-1.5 text-[11px] font-sans font-medium uppercase tracking-wider select-none"
                :class="variant === 'accent' ? 'text-(--accent-ink)/80' : 'text-(--text-tertiary)'"
            >
                <span
                    v-if="icon"
                    class="material-symbols-outlined text-sm select-none"
                    :class="variant === 'accent' ? 'text-(--accent-ink)' : 'text-(--accent-color)'"
                >{{ icon }}</span>
                <span>{{ label }}</span>
            </div>

            <span
                v-if="badge"
                class="text-[10px] font-sans font-semibold px-2 py-0.5 rounded-full uppercase tracking-wider select-none shrink-0"
                :class="variant === 'accent'
                    ? 'bg-black/20 text-(--accent-ink)'
                    : 'bg-(--accent-color-light) text-(--accent-color) border border-(--border-color)'"
            >
                {{ badge }}
            </span>
        </div>

        <div class="flex items-baseline gap-1.5 my-1">
            <span
                class="text-3xl sm:text-4xl font-bold tabular-nums leading-none font-serif tracking-tight"
                :class="variant === 'accent' ? 'text-(--accent-ink)' : 'text-(--text-primary)'"
            >
                {{ value }}
            </span>
            <span
                v-if="unit"
                class="text-xs font-sans font-medium"
                :class="variant === 'accent' ? 'text-(--accent-ink)/80' : 'text-(--text-secondary)'"
            >{{ unit }}</span>
        </div>

        <p
            v-if="subtitle"
            class="text-[11px] mt-1.5 truncate font-sans"
            :class="variant === 'accent' ? 'text-(--accent-ink)/75' : 'text-(--text-tertiary)'"
        >
            {{ subtitle }}
        </p>
    </div>
</template>

