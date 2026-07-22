<script setup lang="ts">
defineProps<{
    syncing: boolean;
    progressMessage: string;
    progressFraction: number;
    importing: boolean;
    importMessage: string;
    importFraction: number;
    lastFinishedResult: { success: boolean; message?: string } | null;
}>();

const emit = defineEmits<{
    (e: 'dismiss-sync'): void;
}>();
</script>

<template>
    <div
        class="fixed bottom-[calc(6.5rem+env(safe-area-inset-bottom,0px))] md:bottom-28 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 w-[calc(100%-2rem)] max-w-sm pointer-events-none"
    >
        <Transition name="fade-slide">
            <div
                v-if="syncing"
                class="pointer-events-auto p-4 rounded-xl border border-(--border-color) bg-(--bg-card)/90 backdrop-blur-md shadow-lg text-xs text-(--text-primary) w-full"
            >
                <div class="flex justify-between items-center mb-2">
                    <span class="font-medium flex items-center gap-1.5">
                        <span class="material-symbols-outlined text-sm animate-spin select-none">sync</span>
                        {{ progressMessage || "Syncing library..." }}
                    </span>
                    <span class="tabular-nums font-semibold">{{ Math.round(progressFraction * 100) }}%</span>
                </div>
                <div class="w-full h-1 bg-(--border-color) rounded overflow-hidden">
                    <div
                        class="h-full bg-(--text-primary) transition-all duration-300"
                        :style="{ width: progressFraction * 100 + '%' }"
                    ></div>
                </div>
            </div>
        </Transition>

        <Transition name="fade-slide">
            <div
                v-if="importing"
                class="pointer-events-auto p-4 rounded-xl border border-(--border-color) bg-(--bg-card)/90 backdrop-blur-md shadow-lg text-xs text-(--text-primary) w-full"
            >
                <div class="flex justify-between items-center mb-2">
                    <span class="font-medium flex items-center gap-1.5">
                        <span class="material-symbols-outlined text-sm animate-pulse select-none">cloud_upload</span>
                        {{ importMessage }}
                    </span>
                    <span class="tabular-nums font-semibold">{{ Math.round(importFraction * 100) }}%</span>
                </div>
                <div class="w-full h-1 bg-(--border-color) rounded overflow-hidden">
                    <div
                        class="h-full bg-(--text-primary) transition-all duration-300"
                        :style="{ width: importFraction * 100 + '%' }"
                    ></div>
                </div>
            </div>
        </Transition>

        <Transition name="fade-slide">
            <div
                v-if="lastFinishedResult && !lastFinishedResult.success"
                class="pointer-events-auto p-4 rounded-xl border border-red-200 dark:border-red-950/40 bg-red-50/90 dark:bg-red-950/20 backdrop-blur-md shadow-lg text-xs text-red-700 dark:text-red-400 flex justify-between items-center gap-3 w-full"
            >
                <div class="flex items-center gap-1.5 min-w-0">
                    <span class="material-symbols-outlined text-sm shrink-0 select-none text-red-500">error</span>
                    <span class="min-w-0 break-words font-medium">Sync failed: {{ lastFinishedResult.message }}</span>
                </div>
                <button
                    @click="emit('dismiss-sync')"
                    class="shrink-0 text-xs font-semibold hover:underline cursor-pointer border-0 bg-transparent text-red-700 dark:text-red-400"
                >
                    Dismiss
                </button>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
    transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.fade-slide-enter-from {
    opacity: 0;
    transform: translateY(12px) scale(0.95);
}
.fade-slide-leave-to {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
}
</style>
