/**
 * Annotation orchestration for the open book: bookmarks and highlights
 * (FR-NOTE-01/02).
 *
 * Owns the per-book annotation lists and the calls that mutate them through the
 * IPC boundary, and bridges the renderer's selection and overlay capability to
 * the store. Bookmarks work for every format; highlights need a renderer with
 * selectable text (ePub), so they are a no-op when no `annotatable` renderer is
 * present. Changes apply locally at once and upload on the next sync.
 */
import { computed, ref, watch, type Ref } from 'vue'
import {
  annotationAddBookmark,
  annotationListBookmarks,
  annotationDeleteBookmark,
  annotationAddHighlight,
  annotationListHighlights,
  annotationDeleteHighlight,
} from '../ipc/annotation'
import type { BookDto, BookmarkDto, HighlightDto } from '../ipc/types'
import type { Annotatable, BookRenderer, Locator, TextSelection, ViewportRect } from '../readers'

/** An existing highlight the reader tapped, with where to anchor its menu. */
export interface ActiveHighlight {
  highlight: HighlightDto
  rect: ViewportRect
}

/** Default highlight tint, matching the renderer's fallback color. */
const DEFAULT_HIGHLIGHT_COLOR = '#f6c945'

export function useAnnotations(
  book: Ref<BookDto>,
  locator: Ref<Locator | null>,
  annotatable: Ref<(BookRenderer & Annotatable) | null>,
  ready: Ref<number>,
) {
  const bookmarks = ref<BookmarkDto[]>([])
  const highlights = ref<HighlightDto[]>([])
  // The live text selection reported by the renderer, used to offer a highlight
  // action; null when nothing is selected.
  const selection = ref<TextSelection | null>(null)
  // The highlight the reader tapped, surfaced so the UI can offer to remove it.
  const activeHighlight = ref<ActiveHighlight | null>(null)

  /**
   * Whether a bookmark sits on the page the reader is currently viewing. ePub
   * pages are spans of text whose CFI shifts as the book re-paginates, so the
   * renderer decides membership by range containment; without that capability
   * (PDF), an exact payload match is enough.
   */
  function bookmarkOnPage(bookmark: BookmarkDto, payload: string): boolean {
    const renderer = annotatable.value
    if (renderer) return renderer.samePage(bookmark.locator.payload, payload)
    return bookmark.locator.payload === payload
  }

  /** Whether the current page already carries a bookmark. */
  const isBookmarked = computed(() => {
    const payload = locator.value?.payload
    if (!payload) return false
    return bookmarks.value.some((b) => bookmarkOnPage(b, payload))
  })

  /** Whether the active renderer can carry highlights (selectable text). */
  const canHighlight = computed(() => annotatable.value !== null)

  async function refresh() {
    const id = book.value.id
    const [bm, hl] = await Promise.all([
      annotationListBookmarks(id),
      annotationListHighlights(id),
    ])
    // Guard against a book switch racing an in-flight load.
    if (book.value.id !== id) return
    bookmarks.value = bm
    highlights.value = hl
  }

  /** Bookmark the current page, or remove the bookmarks already on it. */
  async function toggleBookmark() {
    const current = locator.value
    if (!current?.payload) return
    const existing = bookmarks.value.filter((b) => bookmarkOnPage(b, current.payload))
    if (existing.length > 0) {
      const removed = new Set(existing.map((b) => b.id))
      await Promise.all(existing.map((b) => annotationDeleteBookmark(b.id)))
      bookmarks.value = bookmarks.value.filter((b) => !removed.has(b.id))
      return
    }
    const created = await annotationAddBookmark(book.value.id, current)
    bookmarks.value = [...bookmarks.value, created]
  }

  async function removeBookmark(id: string) {
    await annotationDeleteBookmark(id)
    bookmarks.value = bookmarks.value.filter((b) => b.id !== id)
  }

  /** Highlight the current selection in the given color, then draw and persist it. */
  async function highlightSelection(color: string = DEFAULT_HIGHLIGHT_COLOR) {
    const sel = selection.value
    const renderer = annotatable.value
    if (!sel || !renderer) return
    // The highlight is located by the selection's own CFI; its progression
    // tracks the current page so it sorts and merges with the rest.
    const target: Locator = {
      payload: sel.payload,
      progression: locator.value?.progression ?? 0,
    }
    const created = await annotationAddHighlight(book.value.id, target, sel.text, color)
    highlights.value = [...highlights.value, created]
    renderer.addHighlight(sel.payload, color)
    renderer.clearSelection()
    selection.value = null
  }

  async function removeHighlight(id: string) {
    const target = highlights.value.find((h) => h.id === id)
    await annotationDeleteHighlight(id)
    highlights.value = highlights.value.filter((h) => h.id !== id)
    if (target) annotatable.value?.removeHighlight(target.locator.payload)
    if (activeHighlight.value?.highlight.id === id) activeHighlight.value = null
  }

  function dismissSelection() {
    annotatable.value?.clearSelection()
    selection.value = null
  }

  function dismissActiveHighlight() {
    activeHighlight.value = null
  }

  // Wire renderer callbacks each time a new annotatable renderer is created.
  watch(annotatable, (renderer) => {
    selection.value = null
    activeHighlight.value = null
    if (!renderer) return
    renderer.onSelection((sel) => {
      selection.value = sel
      if (sel) activeHighlight.value = null
    })
    renderer.onHighlightClick((payload, rect) => {
      const highlight = highlights.value.find((h) => h.locator.payload === payload)
      activeHighlight.value = highlight ? { highlight, rect } : null
    })
  })

  // Draw saved highlights once the renderer has painted; foliate clears its
  // overlayers between renders, so this re-seeds them whenever the view reloads
  // or the highlight list arrives.
  watch([ready, () => highlights.value.length], () => {
    const renderer = annotatable.value
    if (!renderer || ready.value === 0) return
    for (const hl of highlights.value) {
      renderer.addHighlight(hl.locator.payload, hl.color ?? DEFAULT_HIGHLIGHT_COLOR)
    }
  })

  // Reload whenever the book changes; clear first so stale lists never show.
  watch(
    () => book.value.id,
    () => {
      bookmarks.value = []
      highlights.value = []
      selection.value = null
      activeHighlight.value = null
      void refresh()
    },
    { immediate: true },
  )

  // Clear any active text selection and highlight popovers when navigating/changing page
  watch(locator, () => {
    dismissSelection()
    dismissActiveHighlight()
  })

  return {
    bookmarks,
    highlights,
    selection,
    activeHighlight,
    isBookmarked,
    canHighlight,
    refresh,
    toggleBookmark,
    removeBookmark,
    highlightSelection,
    removeHighlight,
    dismissSelection,
    dismissActiveHighlight,
  }
}
