<script setup lang="ts">
defineProps<{
    visible: boolean;
}>();
</script>

<template>
    <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-(--bg-app)/60 backdrop-blur-sm transition-all duration-300 pointer-events-none"
    >
        <div class="p-8 max-w-xs rounded-2xl bg-(--bg-card) border border-(--border-color) shadow-xl flex flex-col items-center justify-center text-center animate-fade-in pointer-events-auto">
            <div class="p-4 rounded-full bg-(--accent-color-light) border border-(--border-color) shadow-md flex items-center justify-center mb-4 animate-bounce">
                <span class="material-symbols-outlined text-3xl text-(--text-primary) select-none">
                    upload_file
                </span>
            </div>
            <h2 class="text-lg font-bold font-serif text-(--text-primary) mb-1.5">Import Books</h2>
            <p class="text-xs text-(--text-secondary) font-serif">Drop EPUB or PDF files here to add them to your library</p>
        </div>
    </div>
</template>
