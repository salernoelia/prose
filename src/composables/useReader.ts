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
  isZoomable,
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

  const progress = computed(() => Math.round((locator.value?.progression ?? 0) * 100))
  const hasToc = computed(() => toc.value.length > 0)

  const canZoom = ref(false)
  const ZOOM_FACTOR = 1.25
  let saveTimer: ReturnType<typeof setTimeout> | null = null

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
      instance.onLocationChange((next) => {
        locator.value = next
        if (saveTimer) clearTimeout(saveTimer)
        saveTimer = setTimeout(() => {
          void readingSavePosition(bookId, next)
          saveTimer = null
        }, 500)
      })
      await instance.mount(container)
      instance.applyStyle(readingStyle())
      await instance.load(bookResourceUrl(bookId))
      toc.value = instance.toc()
      canZoom.value = isZoomable(instance)
      renderer.value = instance
      const saved = await readingGetPosition(bookId)
      if (saved) {
        await instance.goToLocator(saved.locator)
      }
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : 'Failed to open this book.'
    } finally {
      loading.value = false
    }
  }

  function teardown() {
    if (saveTimer) {
      clearTimeout(saveTimer)
      saveTimer = null
    }
    renderer.value?.destroy()
    renderer.value = null
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

  onUnmounted(teardown)

  return {
    host,
    loading,
    error,
    locator,
    progress,
    toc,
    hasToc,
    canZoom,
    next,
    prev,
    goToHref,
    zoomIn,
    zoomOut,
  }
}
