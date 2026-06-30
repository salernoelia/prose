<script
    setup
    lang="ts"
>
import { computed } from 'vue'
import type { DefinitionDto } from '../../ipc/types'
import type { ViewportRect } from '../../readers'
import {
    googleSearchUrl,
    openExternal,
    wikipediaUrl,
    wiktionaryUrl,
} from '../../lib/externalLookup'

const props = defineProps<{
    word: string | null
    definitions: DefinitionDto[]
    rect: ViewportRect | null
    loading: boolean
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

// Open the active word in an external reference. The popover already has the
// looked-up word, so each source just builds its own URL.
function openWikipedia() {
    if (props.word) void openExternal(wikipediaUrl(props.word))
}
function openWiktionary() {
    if (props.word) void openExternal(wiktionaryUrl(props.word))
}
function openGoogle() {
    if (props.word) void openExternal(googleSearchUrl(props.word))
}

// Anchor the card below or above the selection, centered and clamped within the viewport
// so the definition stays fully visible.
const placement = computed(() => {
    const rect = props.rect
    if (!rect) {
        return {
            popoverStyle: { display: 'none' },
            bodyStyle: {},
        }
    }
    const centerX = rect.x + rect.width / 2
    const halfWidth = Math.min(160, window.innerWidth / 2 - 16)
    const clampedX = Math.min(Math.max(centerX, halfWidth + 16), window.innerWidth - halfWidth - 16)

    const spaceBelow = window.innerHeight - (rect.y + rect.height + 16)
    const spaceAbove = rect.y - 16
    const headerHeight = 44
    const minBodyHeight = 100

    // Prefer placing below, switch to above if it fits better there
    const placeAbove = spaceBelow < 320 && spaceAbove > spaceBelow

    if (placeAbove) {
        const bottom = window.innerHeight - rect.y + 8
        const maxBodyHeight = Math.max(minBodyHeight, spaceAbove - headerHeight)
        return {
            popoverStyle: {
                left: `${clampedX}px`,
                bottom: `${bottom}px`,
                top: 'auto',
            },
            bodyStyle: {
                maxHeight: `${Math.min(256, maxBodyHeight)}px`,
            }
        }
    } else {
        const top = rect.y + rect.height + 8
        const maxBodyHeight = Math.max(minBodyHeight, spaceBelow - headerHeight)
        return {
            popoverStyle: {
                left: `${clampedX}px`,
                top: `${top}px`,
                bottom: 'auto',
            },
            bodyStyle: {
                maxHeight: `${Math.min(256, maxBodyHeight)}px`,
            }
        }
    }
})
</script>

<template>
    <div
        v-if="word"
        class="fixed z-50 -translate-x-1/2 w-80 max-w-[90vw] animate-fade-in reader-definition-popover"
        :style="placement.popoverStyle"
    >
        <div
            class="rounded-xl bg-(--bg-card) border border-(--border-color) shadow-lg overflow-hidden"
        >
            <header
                class="flex items-center justify-between gap-2 px-4 py-2.5 border-b border-(--border-color)"
            >
                <span class="min-w-0 truncate text-base font-semibold text-(--text-primary) select-text">{{ word }}</span>
                <div class="flex items-center shrink-0">
                    <button
                        @click="openWikipedia"
                        class="group flex items-center justify-center w-7 h-7 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                        title="Search Wikipedia"
                        aria-label="Search Wikipedia"
                    >
                        <img
                            src="/wikipedia.webp"
                            alt=""
                            class="h-[18px] w-[18px] object-contain select-none opacity-55 group-hover:opacity-100 transition-opacity"
                        />
                    </button>
                    <button
                        @click="openWiktionary"
                        class="flex items-center justify-center w-7 h-7 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                        title="Search Wiktionary"
                        aria-label="Search Wiktionary"
                    >
                        <span class="material-symbols-outlined text-lg leading-none select-none">dictionary</span>
                    </button>
                    <button
                        @click="openGoogle"
                        class="flex items-center justify-center w-7 h-7 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                        title="Search Google"
                        aria-label="Search Google"
                    >
                        <span class="material-symbols-outlined text-lg leading-none select-none">search</span>
                    </button>
                    <button
                        @click="emit('close')"
                        class="flex items-center justify-center w-7 h-7 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                        title="Close"
                        aria-label="Close"
                    >
                        <span class="material-symbols-outlined text-lg leading-none select-none">close</span>
                    </button>
                </div>
            </header>

            <div
                class="overflow-y-auto px-4 py-3"
                :style="placement.bodyStyle"
            >
                <div
                    v-if="loading"
                    class="flex items-center gap-2 text-sm text-(--text-tertiary) select-none"
                >
                    <span class="material-symbols-outlined animate-spin text-base">progress_activity</span>
                    Looking up
                </div>

                <p
                    v-else-if="!definitions.length"
                    class="text-sm text-(--text-tertiary) select-none"
                >
                    No definition found.
                </p>

                <ol
                    v-else
                    class="flex flex-col gap-3"
                >
                    <li
                        v-for="(sense, index) in definitions"
                        :key="index"
                        class="select-text"
                    >
                        <span class="text-xs italic text-(--accent-color) mr-1">{{ sense.partOfSpeech }}</span>
                        <span class="text-sm text-(--text-secondary)">{{ sense.gloss }}</span>
                        <p
                            v-for="(example, exIndex) in sense.examples"
                            :key="exIndex"
                            class="mt-0.5 text-xs italic text-(--text-tertiary)"
                        >
                            &ldquo;{{ example }}&rdquo;
                        </p>
                        <p
                            v-if="sense.synonyms.length"
                            class="mt-0.5 text-xs text-(--text-tertiary)"
                        >
                            Synonyms: {{ sense.synonyms.join(', ') }}
                        </p>
                    </li>
                </ol>
            </div>
        </div>
    </div>
</template>
