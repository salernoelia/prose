<script setup lang="ts">
import { computed } from "vue";
import type { BookDto } from "../../ipc/types";

const props = defineProps<{
    book: BookDto;
}>();

const shortAuthor = computed(() => {
    const author = props.book.author;
    if (!author) return "";
    const parts = author.trim().split(/\s+/);
    if (parts.length <= 1) return author;

    let surnameIdx = parts.length - 1;
    const suffixes = ["jr", "jr.", "sr", "sr.", "ii", "iii", "iv"];
    if (surnameIdx > 0 && suffixes.includes(parts[surnameIdx].toLowerCase())) {
        surnameIdx--;
    }

    const firstName = parts[0];
    const firstLetter = firstName.charAt(0).toUpperCase();
    const surname = parts[surnameIdx];
    return `${firstLetter}. ${surname}`;
});
</script>

<template>
    <header
        class="mb-2 pb-2 border-b border-(--border-color) flex justify-between items-center text-xs text-(--text-tertiary) select-none whitespace-nowrap overflow-hidden"
        :style="{
            paddingLeft: '1.5rem',
            paddingRight: '1.5rem',
            paddingTop: 'calc(0.5rem + env(safe-area-inset-top, 0px))',
        }"
    >
        <span class="truncate flex-1 min-w-0 pr-4 text-left">{{ book.title }}</span>
        <span class="shrink-0 text-right">{{ shortAuthor }}</span>
    </header>
</template>
