<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SessionEntry } from '../../composables/useReadingStats'

const props = defineProps<{
    sessions: SessionEntry[]
    formatDuration: (s: number) => string
}>()

const emit = defineEmits<{
    (e: 'delete-session', id: string): void
}>()

const showAll = ref(false)
const deletingId = ref<string | null>(null)

const displayedSessions = computed(() => {
    if (showAll.value || props.sessions.length <= 6) {
        return props.sessions
    }
    return props.sessions.slice(0, 6)
})

function formatSessionDate(ms: number): string {
    const d = new Date(ms)
    const now = new Date()
    const isToday = d.toDateString() === now.toDateString()
    const yesterday = new Date(now)
    yesterday.setDate(now.getDate() - 1)
    const isYesterday = d.toDateString() === yesterday.toDateString()

    const timeStr = d.toLocaleTimeString(undefined, {
        hour: 'numeric',
        minute: '2-digit',
    })

    if (isToday) return `Today at ${timeStr}`
    if (isYesterday) return `Yesterday at ${timeStr}`

    return `${d.toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        year: d.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    })} at ${timeStr}`
}

function confirmDelete(id: string) {
    if (deletingId.value === id) {
        emit('delete-session', id)
        deletingId.value = null
    } else {
        deletingId.value = id
        setTimeout(() => {
            if (deletingId.value === id) deletingId.value = null
        }, 3000)
    }
}
</script>

<template>
    <div class="w-full">
        <!-- Header -->
        <div class="flex items-center justify-between gap-2 mb-4">
            <div>
                <span class="text-[11px] font-sans font-medium uppercase tracking-wider text-(--text-tertiary) select-none block">
                    Session Log
                </span>
                <h3 class="text-lg sm:text-xl font-serif font-bold text-(--text-primary) mt-0.5">
                    Reading History
                </h3>
            </div>

            <button
                v-if="sessions.length > 6"
                @click="showAll = !showAll"
                class="text-xs font-sans font-semibold text-(--accent-color) hover:underline cursor-pointer select-none px-2 py-1"
            >
                {{ showAll ? 'Show less' : `Show all (${sessions.length})` }}
            </button>
        </div>

        <!-- Sessions List -->
        <div class="flex flex-col divide-y divide-(--border-color)/40 dark:divide-white/10">
            <div
                v-for="session in displayedSessions"
                :key="session.id"
                class="py-3 first:pt-0 last:pb-0 flex items-center justify-between gap-3 group hover:bg-(--text-primary)/4 dark:hover:bg-white/5 px-2 -mx-2 rounded-xl transition-all"
            >
                <div class="min-w-0 flex-1">
                    <p class="text-sm sm:text-base font-semibold font-serif text-(--text-primary) truncate">
                        {{ session.bookTitle }}
                    </p>
                    <p class="text-xs font-sans text-(--text-secondary) mt-0.5">
                        {{ formatSessionDate(session.startedAt) }}
                    </p>
                </div>

                <div class="flex items-center gap-2 shrink-0">
                    <span class="px-3 py-1 rounded-full bg-(--text-primary)/5 dark:bg-white/10 border border-(--border-color)/60 dark:border-white/15 text-xs font-sans font-bold tabular-nums text-(--text-primary)">
                        {{ formatDuration(session.durationSeconds) }}
                    </span>

                    <button
                        @click="confirmDelete(session.id)"
                        class="flex items-center justify-center w-7 h-7 rounded-full transition-all focus-ring-minimal cursor-pointer"
                        :class="deletingId === session.id
                            ? 'bg-red-500 text-white shadow-xs'
                            : 'text-(--text-tertiary) hover:text-red-500 hover:bg-(--text-primary)/5 dark:hover:bg-white/10'"
                        :title="deletingId === session.id ? 'Click again to confirm deletion' : 'Delete session'"
                        :aria-label="deletingId === session.id ? 'Confirm delete session' : 'Delete session'"
                    >
                        <span class="material-symbols-outlined text-base select-none">
                            {{ deletingId === session.id ? 'check' : 'delete' }}
                        </span>
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>
