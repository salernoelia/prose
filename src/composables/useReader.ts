/**
 * Reader orchestration: open a book by id, mount the right renderer, and expose
 * navigation and position to the view (architecture sections 6, 9).
 *
 * The composable owns the renderer lifecycle. Components stay declarative: they
 * bind a host element, read reactive state, and call navigation methods. Book
 * bytes reach the renderer only through the `prose://` URL resolved here.
 */
import { computed, onUnmounted, ref, shallowRef, watch, type Ref } from 'vue'
import {
  createRenderer,
  isAnnotatable,
  isZoomable,
  type Annotatable,
  type BookRenderer,
  type Locator,
  type TocItem,
} from '../readers'
import { bookResourceUrl } from '../ipc/protocol'
import { readingSavePosition, readingGetPosition } from '../ipc/reading'
import type { BookDto } from '../ipc/types'
import { useSettings } from './useSettings'

export function useReader(book: Ref<BookDto>) {
  const { settings } = useSettings()

  const host = ref<HTMLElement | null>(null)
  const renderer = shallowRef<BookRenderer | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  const locator = ref<Locator | null>(null)
  const toc = ref<TocItem[]>([])
  // Bumps once the renderer has loaded and painted, so annotation logic knows it
  // is safe to draw saved highlights into the freshly created overlayers.
  const ready = ref(0)
  // The active renderer narrowed to its highlight capability, or null for
  // fixed-layout formats (PDF) that do not support highlights.
  const annotatable = shallowRef<(BookRenderer & Annotatable) | null>(null)

  const progress = computed(() => Math.round((locator.value?.progression ?? 0) * 100))
  const hasToc = computed(() => toc.value.length > 0)

  const canZoom = ref(false)
  const ZOOM_FACTOR = 1.25
  let saveTimer: ReturnType<typeof setTimeout> | null = null
  // The latest position not yet written, kept so a pending save can be flushed
  // immediately when the book closes or the app is hidden.
  let pendingSave: { bookId: string; locator: Locator } | null = null

  function flushPendingSave() {
    if (saveTimer) {
      clearTimeout(saveTimer)
      saveTimer = null
    }
    if (pendingSave) {
      void readingSavePosition(pendingSave.bookId, pendingSave.locator)
      pendingSave = null
    }
  }

  // The app being backgrounded or closed should never lose the current page;
  // visibilitychange fires before the window goes away on every platform.
  function onHidden() {
    if (document.visibilityState === 'hidden') flushPendingSave()
  }

  function readingStyle() {
    return {
      fontFamily: settings.value.fontFamily,
      fontSize: settings.value.fontSize,
      lineHeight: settings.value.lineHeight,
      margin: settings.value.margin,
      theme: settings.value.theme,
    }
  }

  async function open() {
    const container = host.value
    if (!container) return
    loading.value = true
    error.value = null
    locator.value = null
    toc.value = []
    try {
      const instance = await createRenderer(book.value.format)
      const bookId = book.value.id

      const saved = await readingGetPosition(bookId)
      let lastSavedPayload = saved?.locator.payload ?? null
      let isInitializing = true

      instance.onLocationChange((next) => {
        locator.value = next
        if (isInitializing) return

        const hasMoved = next.payload !== lastSavedPayload
        if (hasMoved) {
          lastSavedPayload = next.payload
          // Record the move right away, then debounce the write so a burst of
          // page turns coalesces into one. The pending position is flushed on
          // teardown and on app hide, so a quick close never loses the page.
          pendingSave = { bookId, locator: next }
          if (saveTimer) clearTimeout(saveTimer)
          saveTimer = setTimeout(() => {
            saveTimer = null
            if (pendingSave) {
              void readingSavePosition(pendingSave.bookId, pendingSave.locator)
              pendingSave = null
            }
          }, 500)
        }
      })
      await instance.mount(container)
      instance.applyStyle(readingStyle())
      await instance.load(bookResourceUrl(bookId))
      toc.value = instance.toc()
      canZoom.value = isZoomable(instance)
      annotatable.value = isAnnotatable(instance) ? instance : null
      renderer.value = instance
      if (saved) {
        await instance.goToLocator(saved.locator)
      }
      isInitializing = false
      const currentLocator = locator.value as Locator | null
      if (currentLocator) {
        lastSavedPayload = currentLocator.payload
      }
      // Signal that saved highlights can now be drawn into the live overlayers.
      ready.value++
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : 'Failed to open this book.'
    } finally {
      loading.value = false
    }
  }

  function teardown() {
    flushPendingSave()
    renderer.value?.destroy()
    renderer.value = null
    annotatable.value = null
  }

  function next() {
    renderer.value?.next()
  }

  function prev() {
    renderer.value?.prev()
  }

  async function goToHref(href: string) {
    await renderer.value?.goToHref(href)
  }

  async function goToLocator(target: Locator) {
    await renderer.value?.goToLocator(target)
  }

  function zoomIn() {
    const instance = renderer.value
    if (instance && isZoomable(instance)) instance.zoomBy(ZOOM_FACTOR)
  }

  function zoomOut() {
    const instance = renderer.value
    if (instance && isZoomable(instance)) instance.zoomBy(1 / ZOOM_FACTOR)
  }

  // Reopen from scratch whenever the host attaches or the book changes.
  watch([host, () => book.value.id], () => {
    teardown()
    void open()
  })

  // Reflow ePub typography and theme when reading settings change; PDF ignores this.
  watch(
    () => [
      settings.value.fontFamily,
      settings.value.fontSize,
      settings.value.lineHeight,
      settings.value.margin,
      settings.value.theme,
    ],
    () => renderer.value?.applyStyle(readingStyle()),
  )

  document.addEventListener('visibilitychange', onHidden)
  onUnmounted(() => {
    document.removeEventListener('visibilitychange', onHidden)
    teardown()
  })

  return {
    host,
    loading,
    error,
    locator,
    progress,
    toc,
    hasToc,
    canZoom,
    ready,
    annotatable,
    next,
    prev,
    goToHref,
    goToLocator,
    zoomIn,
    zoomOut,
  }
}
