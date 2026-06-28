/**
 * PDF rendering over pdf.js (architecture section 6, FR-READ-02/05).
 *
 * pdf.js fetches pages from the book's `prose://` URL with HTTP range requests,
 * so only the bytes a page needs are streamed. One page is painted at a time,
 * fit to width by default, with a renderer-local zoom. `applyStyle` is a no-op:
 * PDF is fixed-layout (architecture section 6).
 */
import * as pdfjs from 'pdfjs-dist'
import type {
  PDFDocumentLoadingTask,
  PDFDocumentProxy,
  PDFPageProxy,
  RenderTask,
} from 'pdfjs-dist'
import workerSrc from 'pdfjs-dist/build/pdf.worker.mjs?url'
import type { BookRenderer, Locator, TocItem, Zoomable } from './types'

pdfjs.GlobalWorkerOptions.workerSrc = workerSrc

/** A raw pdf.js outline node, narrowed to the fields we resolve. */
interface PdfOutlineNode {
  title: string
  dest: string | unknown[] | null
  items: PdfOutlineNode[]
}

export class PdfRenderer implements BookRenderer, Zoomable {
  #container: HTMLElement | null = null
  /** A stable, non-scrolling ancestor measured for the fit calculation. */
  #viewport: HTMLElement | null = null
  /** Zoom captured at the start of a trackpad pinch gesture. */
  #gestureBaseZoom = 1
  #canvas: HTMLCanvasElement | null = null
  #loadingTask: PDFDocumentLoadingTask | null = null
  #doc: PDFDocumentProxy | null = null
  #renderTask: RenderTask | null = null
  #resizeObserver: ResizeObserver | null = null
  #repaintHandle: number | null = null
  #locationListeners: Array<(locator: Locator) => void> = []
  #toc: TocItem[] = []
  #page = 1
  #pageCount = 1
  /** User zoom multiplier on top of the fit-to-page base scale (FR-READ-05). */
  #zoom = 1

  async mount(container: HTMLElement): Promise<void> {
    // The page scrolls only when zoomed past fit; centered otherwise. `margin:
    // auto` inside a flex container centers on both axes while still allowing
    // every edge to be scrolled into view once the canvas exceeds the viewport.
    container.style.overflow = 'auto'
    container.style.display = 'flex'

    const canvas = document.createElement('canvas')
    canvas.style.display = 'block'
    canvas.style.margin = 'auto'
    canvas.style.flex = '0 0 auto'
    container.append(canvas)
    this.#canvas = canvas
    this.#container = container

    // Measure the non-scrolling parent so the fit is immune to the scrollbar
    // the canvas itself may introduce when zoomed (which otherwise feeds back
    // into the observer and triggers the "ResizeObserver loop" warning).
    this.#viewport = container.parentElement ?? container
    this.#resizeObserver = new ResizeObserver(() => this.#scheduleRepaint())
    this.#resizeObserver.observe(this.#viewport)

    // Pinch-to-zoom: a trackpad pinch is a `wheel` with `ctrlKey` set; WebKit
    // also emits `gesture*` events. Both are handled so the canvas zooms in
    // place instead of scrolling the page or zooming the whole WebView.
    container.addEventListener('wheel', this.#onWheel, { passive: false })
    container.addEventListener('gesturestart', this.#onGestureStart as EventListener)
    container.addEventListener('gesturechange', this.#onGestureChange as EventListener)
  }

  async load(source: string): Promise<void> {
    this.#loadingTask = pdfjs.getDocument({ url: source })
    this.#doc = await this.#loadingTask.promise
    this.#pageCount = this.#doc.numPages
    this.#toc = await this.#buildToc()
    await this.#paint()
  }

  destroy(): void {
    if (this.#repaintHandle != null) cancelAnimationFrame(this.#repaintHandle)
    this.#container?.removeEventListener('wheel', this.#onWheel)
    this.#container?.removeEventListener('gesturestart', this.#onGestureStart as EventListener)
    this.#container?.removeEventListener('gesturechange', this.#onGestureChange as EventListener)
    this.#renderTask?.cancel()
    this.#resizeObserver?.disconnect()
    this.#canvas?.remove()
    void this.#loadingTask?.destroy()
    this.#repaintHandle = null
    this.#renderTask = null
    this.#resizeObserver = null
    this.#canvas = null
    this.#container = null
    this.#viewport = null
    this.#loadingTask = null
    this.#doc = null
    this.#locationListeners = []
  }

  next(): void {
    if (this.#page < this.#pageCount) {
      this.#page += 1
      void this.#paint()
    }
  }

  prev(): void {
    if (this.#page > 1) {
      this.#page -= 1
      void this.#paint()
    }
  }

  async goToLocator(locator: Locator): Promise<void> {
    const page = Number.parseInt(locator.payload, 10)
    if (Number.isFinite(page)) await this.#setPage(page)
  }

  async goToHref(href: string): Promise<void> {
    const page = Number.parseInt(href, 10)
    if (Number.isFinite(page)) await this.#setPage(page)
  }

  toc(): TocItem[] {
    return this.#toc
  }

  onLocationChange(cb: (locator: Locator) => void): void {
    this.#locationListeners.push(cb)
  }

  applyStyle(): void {
    // PDF is fixed-layout; typography settings do not apply.
  }

  /** Adjust the zoom multiplier and repaint. 1 is fit-to-page (the whole page). */
  setZoom(zoom: number): void {
    const clamped = Math.max(1, Math.min(zoom, 6))
    if (clamped === this.#zoom) return
    this.#zoom = clamped
    this.#scheduleRepaint()
  }

  zoomBy(factor: number): void {
    this.setZoom(this.#zoom * factor)
  }

  #onWheel = (event: WheelEvent): void => {
    if (!event.ctrlKey) return
    event.preventDefault()
    // ctrlKey + wheel is the trackpad pinch signal; deltaY < 0 means zoom in.
    this.zoomBy(Math.exp(-event.deltaY * 0.01))
  }

  #onGestureStart = (event: Event): void => {
    event.preventDefault()
    this.#gestureBaseZoom = this.#zoom
  }

  #onGestureChange = (event: Event & { scale?: number }): void => {
    event.preventDefault()
    if (event.scale) this.setZoom(this.#gestureBaseZoom * event.scale)
  }

  /** Coalesce repaint requests into one per frame so resize cannot loop. */
  #scheduleRepaint(): void {
    if (this.#repaintHandle != null) return
    this.#repaintHandle = requestAnimationFrame(() => {
      this.#repaintHandle = null
      void this.#paint()
    })
  }

  async #setPage(page: number): Promise<void> {
    const clamped = Math.max(1, Math.min(page, this.#pageCount))
    if (clamped === this.#page) return
    this.#page = clamped
    await this.#paint()
  }

  async #paint(): Promise<void> {
    if (!this.#doc || !this.#canvas || !this.#viewport) return
    this.#renderTask?.cancel()

    const page: PDFPageProxy = await this.#doc.getPage(this.#page)
    const unscaled = page.getViewport({ scale: 1 })
    // Fit the whole page to the viewport (contain), so it is never cut off in
    // wide or short windows; zoom scales up from there (FR-READ-05).
    const fitWidth = this.#viewport.clientWidth || unscaled.width
    const fitHeight = this.#viewport.clientHeight || unscaled.height
    const baseScale = Math.min(fitWidth / unscaled.width, fitHeight / unscaled.height)
    const dpr = window.devicePixelRatio || 1
    const viewport = page.getViewport({ scale: baseScale * this.#zoom })

    const canvas = this.#canvas
    const context = canvas.getContext('2d')
    if (!context) return
    canvas.width = Math.floor(viewport.width * dpr)
    canvas.height = Math.floor(viewport.height * dpr)
    canvas.style.width = `${Math.floor(viewport.width)}px`
    canvas.style.height = `${Math.floor(viewport.height)}px`

    this.#renderTask = page.render({
      canvas,
      canvasContext: context,
      viewport,
      transform: dpr === 1 ? undefined : [dpr, 0, 0, dpr, 0, 0],
    })
    try {
      await this.#renderTask.promise
    } catch (error) {
      // A cancelled render (page turn or resize mid-paint) is expected.
      if (!(error instanceof Error) || error.name !== 'RenderingCancelledException') throw error
      return
    }
    this.#emitLocation()
  }

  async #buildToc(): Promise<TocItem[]> {
    if (!this.#doc) return []
    const outline = (await this.#doc.getOutline()) as PdfOutlineNode[] | null
    if (!outline) return []
    const resolve = async (nodes: PdfOutlineNode[]): Promise<TocItem[]> =>
      Promise.all(
        nodes.map(async (node) => ({
          label: node.title,
          href: String((await this.#pageOf(node.dest)) ?? this.#page),
          subitems: await resolve(node.items ?? []),
        })),
      )
    return resolve(outline)
  }

  /** Resolve a pdf.js destination to a 1-based page number. */
  async #pageOf(dest: string | unknown[] | null): Promise<number | null> {
    if (!this.#doc || dest == null) return null
    const explicit = typeof dest === 'string' ? await this.#doc.getDestination(dest) : dest
    const ref = Array.isArray(explicit) ? explicit[0] : null
    if (!ref) return null
    try {
      return (await this.#doc.getPageIndex(ref as never)) + 1
    } catch {
      return null
    }
  }

  #emitLocation(): void {
    const progression = this.#pageCount > 1 ? (this.#page - 1) / (this.#pageCount - 1) : 1
    const locator: Locator = { payload: String(this.#page), progression }
    for (const listener of this.#locationListeners) listener(locator)
  }
}
