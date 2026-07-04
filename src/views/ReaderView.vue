<script
    setup
    lang="ts"
>
import { computed, onMounted, onUnmounted, ref, toRef, watch } from "vue";
import { useSettings } from "../composables/useSettings";
import { useReader } from "../composables/useReader";
import { useAnnotations } from "../composables/useAnnotations";
import ReaderClickZones from "../components/reader/ReaderClickZones.vue";
import ReaderDock from "../components/reader/ReaderDock.vue";
import ReaderTocDrawer from "../components/reader/ReaderTocDrawer.vue";
import ReaderAnnotationsDrawer from "../components/reader/ReaderAnnotationsDrawer.vue";
import ReaderAnnotationPopover from "../components/reader/ReaderAnnotationPopover.vue";
import ReaderDefinitionPopover from "../components/reader/ReaderDefinitionPopover.vue";
import ReaderQuickSettings from "../components/reader/ReaderQuickSettings.vue";
import { useDictionary } from "../composables/useDictionary";
import { useSync } from "../composables/useSync";
import { startSession, endSession } from "../composables/useReadingTracker";
import {
    googleSearchUrl,
    openExternal,
    translateUrl,
    wikipediaUrl,
} from "../lib/externalLookup";
import type { BookDto, BookmarkDto, HighlightDto } from "../ipc/types";

const props = defineProps<{
    book: BookDto;
}>();

const emit = defineEmits<{
    (e: "back-to-library"): void;
}>();

const { clickZoneSize, translationLanguage } = useSettings();

const shortAuthor = computed(() => {
    const author = props.book.author;
    if (!author) return "";
    const parts = author.trim().split(/\s+/);
    if (parts.length <= 1) return author;

    // If the last part is a common suffix, try the second-to-last part
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

const {
    host,
    loading,
    error,
    locator,
    progress,
    toc,
    hasToc,
    canZoom,
    canUndoJump,
    ready,
    annotatable,
    next,
    prev,
    goToHref,
    goToLocator,
    undoJump,
    zoomIn,
    zoomOut,
} = useReader(toRef(props, "book"));

const {
    bookmarks,
    highlights,
    selection,
    activeHighlight,
    isBookmarked,
    toggleBookmark,
    removeBookmark,
    highlightSelection,
    removeHighlight,
    dismissSelection,
    dismissActiveHighlight,
} = useAnnotations(toRef(props, "book"), locator, annotatable, ready);

const {
    word: definitionWord,
    definitions,
    rect: definitionRect,
    loading: definitionLoading,
    lookup: lookupWord,
    clear: clearDefinition,
} = useDictionary();

// Offer a definition only for a single selected word; multi-word selections are
// for highlighting, not lookup.
const isSingleWord = computed(() => {
    const text = selection.value?.text.trim() ?? "";
    return text.length > 0 && !/\s/.test(text);
});

// Mirror the selection popover for an existing highlight: a single word can be
// defined, a longer passage is searched instead.
const isActiveHighlightSingleWord = computed(() => {
    const text = activeHighlight.value?.highlight.text.trim() ?? "";
    return text.length > 0 && !/\s/.test(text);
});

const showDock = ref(true);
const showToc = ref(false);
const showAnnotations = ref(false);
const showQuickSettings = ref(false);

const canPrev = computed(() => progress.value > 0);
const canNext = computed(() => progress.value < 100);

const { configured, syncing, triggerSync } = useSync();

function toggleDock() {
    showDock.value = !showDock.value;
}

function onSelectToc(href: string) {
    void goToHref(href);
}

function onSelectBookmark(bookmark: BookmarkDto) {
    void goToLocator(bookmark.locator);
    showAnnotations.value = false;
}

function onSelectHighlight(highlight: HighlightDto) {
    void goToLocator(highlight.locator);
    showAnnotations.value = false;
}

function onHighlight() {
    void highlightSelection();
}

const tempHighlightCfi = ref<string | null>(null);

function handleClearDefinition() {
    if (tempHighlightCfi.value && annotatable.value) {
        annotatable.value.removeHighlight(tempHighlightCfi.value);
        tempHighlightCfi.value = null;
    }
    clearDefinition();
}

function onDefine() {
    const current = selection.value;
    if (!current) return;

    // A define popover and the highlight-remove popover must never stack: the
    // remove button would sit invisibly under the definition card and a tap
    // there would silently delete the highlight.
    dismissActiveHighlight();

    // The temporary blue highlight is only a visual cue for the looked-up word.
    // Never draw it over a real highlight at the same range: clearing the temp
    // would also erase the real one (both are keyed by payload in the renderer).
    const alreadyHighlighted = highlights.value.some(
        (h) => h.locator.payload === current.payload,
    );
    if (annotatable.value && !alreadyHighlighted) {
        tempHighlightCfi.value = current.payload;
        annotatable.value.addHighlight(current.payload, "#3b82f6");
    }

    void lookupWord(current.text, current.rect);
    dismissSelection();
}

function onTranslate() {
    const text = selection.value?.text.trim();
    if (text) void openExternal(translateUrl(text, translationLanguage.value));
    dismissSelection();
}

function onSearch() {
    const text = selection.value?.text.trim();
    if (text) void openExternal(googleSearchUrl(text));
    dismissSelection();
}

function onWikipedia() {
    const text = selection.value?.text.trim();
    if (text) void openExternal(wikipediaUrl(text));
    dismissSelection();
}

// Automatically clear active definition popover and temporary highlights on page navigation
watch(locator, () => {
    handleClearDefinition();
});

function onRemoveActiveHighlight() {
    const active = activeHighlight.value;
    if (active) void removeHighlight(active.highlight.id);
}

function onDefineActiveHighlight() {
    const active = activeHighlight.value;
    if (!active) return;
    // The text is already highlighted, so look it up without touching any
    // highlight; defining a mark must never remove it.
    void lookupWord(active.highlight.text.trim(), active.rect);
    dismissActiveHighlight();
}

function onSearchActiveHighlight() {
    const active = activeHighlight.value;
    if (!active) return;
    void openExternal(googleSearchUrl(active.highlight.text.trim()));
    dismissActiveHighlight();
}

function onTranslateActiveHighlight() {
    const active = activeHighlight.value;
    if (!active) return;
    void openExternal(
        translateUrl(active.highlight.text.trim(), translationLanguage.value),
    );
    dismissActiveHighlight();
}

function onWikipediaActiveHighlight() {
    const active = activeHighlight.value;
    if (!active) return;
    void openExternal(wikipediaUrl(active.highlight.text.trim()));
    dismissActiveHighlight();
}

function handleBack() {
    emit("back-to-library");
    if (configured.value && !syncing.value) {
        void triggerSync();
    }
}

const handleKeyDown = (e: KeyboardEvent) => {
    const target = e.target as HTMLElement | null;
    if (
        target &&
        (target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.tagName === "SELECT" ||
            target.isContentEditable)
    ) {
        return;
    }

    if (e.key === "ArrowRight") {
        next();
    } else if (e.key === "ArrowLeft") {
        prev();
    }
};

function handleRendererClick(e: Event) {
    const customEvent = e as CustomEvent<{ target: Node; x?: number }>;
    if (definitionWord.value) {
        const target = customEvent.detail?.target || e.target;
        const popoverEl = document.querySelector(".reader-definition-popover");
        if (popoverEl && popoverEl.contains(target as Node)) {
            return;
        }
        handleClearDefinition();
        return;
    }

    // A tap in a side turn-zone flips the page; a tap in the middle toggles the
    // dock. Zones are measured from the viewport edges, the same width the
    // settings preview shows. The renderer reports the tap's x in window space.
    const x = customEvent.detail?.x;
    if (typeof x === "number") {
        const zoneWidth = (window.innerWidth * clickZoneSize.value) / 100;
        if (x < zoneWidth) {
            prev();
            return;
        }
        if (x > window.innerWidth - zoneWidth) {
            next();
            return;
        }
    }
    toggleDock();
}

const handleOutsideClick = (e: MouseEvent) => {
    if (!definitionWord.value) return;
    const popoverEl = document.querySelector(".reader-definition-popover");
    if (popoverEl && popoverEl.contains(e.target as Node)) {
        return;
    }
    const defineBtn = (e.target as HTMLElement).closest('[title="Define"]');
    if (defineBtn) {
        return;
    }
    handleClearDefinition();
};

onMounted(() => {
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("click", handleOutsideClick);
    startSession(props.book);
});

onUnmounted(() => {
    window.removeEventListener("keydown", handleKeyDown);
    window.removeEventListener("click", handleOutsideClick);
    endSession();
});
</script>

<template>
    <div class="w-full relative h-full flex flex-col justify-between select-none">
        <ReaderClickZones
            @prev="prev"
            @next="next"
        />

        <!-- Non-Scrolling Reading Canvas (Overflow hidden, flex-1, with fade-in) -->
        <div
            class="relative z-0 w-full flex-1 overflow-hidden select-text transition-all duration-300 flex flex-col animate-fade-in">
            <!-- Book Header Info (Subtle) -->
            <header
                class="mb-2 pb-2 border-b border-(--border-color) flex justify-between items-center text-xs text-(--text-tertiary) select-none whitespace-nowrap overflow-hidden"
                :style="{
                    paddingLeft: '1.5rem',
                    paddingRight: '1.5rem',
                    paddingTop: 'calc(0.5rem + env(safe-area-inset-top, 0px))',
                }"
            >
                <span class="truncate flex-1 min-w-0 pr-4 text-left">{{
                    book.title
                    }}</span>
                <span class="shrink-0 text-right">{{ shortAuthor }}</span>
            </header>

            <!-- Renderer host: foliate-js (ePub) or pdf.js (PDF) mounts here.
                 A bottom inset keeps the last line clear of the dock (which now
                 floats over the page) and the device's safe area. -->
            <div class="relative flex-1 overflow-hidden">
                <div
                    ref="host"
                    class="absolute inset-x-0 top-0"
                    style="bottom: calc(2.5rem + env(safe-area-inset-bottom, 0px))"
                    @renderer-click="handleRendererClick"
                ></div>

                <!-- Loading state -->
                <div
                    v-if="loading"
                    class="absolute inset-0 flex items-center justify-center text-sm text-(--text-tertiary) select-none animate-fade-in"
                >
                    <span class="material-symbols-outlined animate-spin mr-2">progress_activity</span>
                    Opening book
                </div>

                <!-- Error state -->
                <div
                    v-else-if="error"
                    class="absolute inset-0 flex flex-col items-center justify-center gap-2 px-8 text-center select-none"
                >
                    <span class="material-symbols-outlined text-3xl text-(--text-tertiary)">error</span>
                    <p class="text-sm text-(--text-secondary)">{{ error }}</p>
                </div>
            </div>
        </div>

        <ReaderDock
            :visible="showDock"
            :progress="progress"
            :can-prev="canPrev"
            :can-next="canNext"
            :bookmarked="isBookmarked"
            :has-toc="hasToc"
            :can-zoom="canZoom"
            :can-undo-jump="canUndoJump"
            @back="handleBack"
            @undo-jump="undoJump"
            @toc="showToc = true"
            @annotations="showAnnotations = true"
            @toggle-bookmark="toggleBookmark"
            @prev="prev"
            @next="next"
            @zoom-in="zoomIn"
            @zoom-out="zoomOut"
            @quick-settings="showQuickSettings = !showQuickSettings"
            @show="showDock = true"
        />

        <ReaderQuickSettings
            :visible="showQuickSettings"
            @close="showQuickSettings = false"
        />

        <ReaderTocDrawer
            v-model:visible="showToc"
            :items="toc"
            @select="onSelectToc"
        />

        <ReaderAnnotationsDrawer
            v-model:visible="showAnnotations"
            :bookmarks="bookmarks"
            :highlights="highlights"
            @select-bookmark="onSelectBookmark"
            @delete-bookmark="removeBookmark"
            @select-highlight="onSelectHighlight"
            @delete-highlight="removeHighlight"
        />

        <!-- Floating action over a fresh text selection -->
        <ReaderAnnotationPopover :rect="selection?.rect ?? null">
            <button
                @click="onHighlight"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Highlight"
                aria-label="Highlight"
            >
                <span class="material-symbols-outlined text-xl">format_ink_highlighter</span>
            </button>
            <button
                v-if="isSingleWord"
                @click="onDefine"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Define"
                aria-label="Define"
            >
                <span class="material-symbols-outlined text-xl">dictionary</span>
            </button>
            <button
                @click="onTranslate"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Translate"
                aria-label="Translate"
            >
                <span class="material-symbols-outlined text-xl">translate</span>
            </button>
            <button
                @click="onSearch"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Search"
                aria-label="Search"
            >
                <span class="material-symbols-outlined text-xl">search</span>
            </button>
            <button
                @click="onWikipedia"
                class="group flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Wikipedia"
                aria-label="Wikipedia"
            >
                <img
                    src="/wikipedia.webp"
                    alt=""
                    class="h-[22px] w-[22px] object-contain select-none opacity-55 group-hover:opacity-100 transition-opacity"
                    style="filter: invert(var(--icon-invert))"
                />
            </button>
        </ReaderAnnotationPopover>

        <!-- Offline dictionary definition card -->
        <ReaderDefinitionPopover
            :word="definitionWord"
            :definitions="definitions"
            :rect="definitionRect"
            :loading="definitionLoading"
            @close="handleClearDefinition"
        />

        <!-- Floating action over an existing highlight. Hidden while a definition
             is open so its Remove button can never sit under the definition card. -->
        <ReaderAnnotationPopover :rect="definitionWord ? null : (activeHighlight?.rect ?? null)">
            <button
                @click="onRemoveActiveHighlight"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Remove highlight"
                aria-label="Remove highlight"
            >
                <span class="material-symbols-outlined text-xl">delete</span>
            </button>
            <button
                v-if="isActiveHighlightSingleWord"
                @click="onDefineActiveHighlight"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Define"
                aria-label="Define"
            >
                <span class="material-symbols-outlined text-xl">dictionary</span>
            </button>
            <button
                @click="onTranslateActiveHighlight"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Translate"
                aria-label="Translate"
            >
                <span class="material-symbols-outlined text-xl">translate</span>
            </button>
            <button
                @click="onSearchActiveHighlight"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Search"
                aria-label="Search"
            >
                <span class="material-symbols-outlined text-xl">search</span>
            </button>
            <button
                @click="onWikipediaActiveHighlight"
                class="group flex items-center justify-center w-10 h-10 rounded-full text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--accent-color-light) transition-colors focus-ring-minimal"
                title="Wikipedia"
                aria-label="Wikipedia"
            >
                <img
                    src="/wikipedia.webp"
                    alt=""
                    class="h-[22px] w-[22px] object-contain select-none opacity-55 group-hover:opacity-100 transition-opacity"
                    style="filter: invert(var(--icon-invert))"
                />
            </button>
            <button
                @click="dismissActiveHighlight"
                class="flex items-center justify-center w-10 h-10 rounded-full text-(--text-tertiary) hover:text-(--text-primary) transition-colors focus-ring-minimal"
                title="Dismiss"
                aria-label="Dismiss"
            >
                <span class="material-symbols-outlined text-xl">close</span>
            </button>
        </ReaderAnnotationPopover>
    </div>
</template>
