/**
 * PDF rendering over pdf.js (architecture section 6, FR-READ-02/05).
 *
 * The document is fetched once from the book's `prose://` URL and handed to
 * pdf.js as a buffer. pdf.js only uses `fetch` for `http(s)` URLs; for a custom
 * scheme it falls back to an XHR stream the WebView rejects ("Failed to fetch"),
 * so we fetch the bytes ourselves the same way the ePub renderer does. One page
 * is painted at a time, fit to width by default, with a renderer-local zoom.
 * `applyStyle` is a no-op: PDF is fixed-layout (architecture section 6).
 */
import * as pdfjs from 'pdfjs-dist'
import type {
  PDFDocumentLoadingTask,
  PDFDocumentProxy,
  PDFPageProxy,
  RenderTask,
} from 'pdfjs-dist'
import workerSrc from 'pdfjs-dist/build/pdf.worker.mjs?url'
import type { BookRenderer, Locator, ReadingStyle, TocItem, Zoomable } from './types'
import { isDarkTheme } from './themes'
import { installColorInvert } from './darkInvert'

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

  #baseWidth = 0
  #baseHeight = 0
  #debounceTimeoutHandle: number | null = null
  #lastPaintedPage = 1
  /** Active reading theme; dark themes invert the page (FR-READ themes). */
  #theme: ReadingStyle['theme'] = 'light'
  /** Per-page invert strategy, cached so zoom and page-flips never recompute. */
  #pageModes = new Map<number, 'smart' | 'scanned'>()

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

    // Fit to the host's own box, not its parent: the host is inset from the
    // parent (it clears the floating dock), so measuring the parent oversizes
    // the canvas and makes it overflow vertically. Observe and measure the
    // border-box, which a scrollbar never changes, so the fit cannot feed back
    // into the observer and trigger the "ResizeObserver loop" warning.
    this.#viewport = container
    this.#resizeObserver = new ResizeObserver(() => this.#scheduleRepaint())
    this.#resizeObserver.observe(this.#viewport, { box: 'border-box' })

    // Pinch-to-zoom: a trackpad pinch is a `wheel` with `ctrlKey` set; WebKit
    // also emits `gesture*` events. Both are handled so the canvas zooms in
    // place instead of scrolling the page or zooming the whole WebView.
    container.addEventListener('wheel', this.#onWheel, { passive: false })
    container.addEventListener('gesturestart', this.#onGestureStart as EventListener, { passive: false })
    container.addEventListener('gesturechange', this.#onGestureChange as EventListener, { passive: false })

    let startX = 0
    let startY = 0
    container.addEventListener('mousedown', (e) => {
      startX = e.clientX
      startY = e.clientY
    })
    container.addEventListener('click', (e) => {
      if (e.target !== container && e.target !== canvas) return
      const diffX = Math.abs(e.clientX - startX)
      const diffY = Math.abs(e.clientY - startY)
      if (diffX < 5 && diffY < 5) {
        container.dispatchEvent(new CustomEvent('renderer-click', {
          bubbles: true,
          detail: { target: e.target, x: e.clientX }
        }))
      }
    })
  }

  async load(source: string): Promise<void> {
    const response = await fetch(source)
    if (!response.ok) throw new Error(`Failed to load PDF (${response.status}).`)
    const data = new Uint8Array(await response.arrayBuffer())
    this.#loadingTask = pdfjs.getDocument({ data })
    this.#doc = await this.#loadingTask.promise
    this.#pageCount = this.#doc.numPages
    this.#toc = await this.#buildToc()
    await this.#paint()
  }

  destroy(): void {
    if (this.#repaintHandle != null) cancelAnimationFrame(this.#repaintHandle)
    if (this.#debounceTimeoutHandle != null) {
      clearTimeout(this.#debounceTimeoutHandle)
      this.#debounceTimeoutHandle = null
    }
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

  applyStyle(style: ReadingStyle): void {
    // PDF is fixed-layout, so typography is ignored; only the theme matters,
    // and only to decide whether the page is inverted for dark reading.
    if (style.theme === this.#theme) return
    this.#theme = style.theme
    if (this.#doc) void this.#paint()
  }

  /** Adjust the zoom multiplier and repaint. 1 is fit-to-page (the whole page). */
  setZoom(zoom: number, anchor?: { clientX: number; clientY: number }): void {
    const clamped = Math.max(1, Math.min(zoom, 6))
    if (clamped === this.#zoom) return
    const oldZoom = this.#zoom
    this.#zoom = clamped

    const container = this.#container
    const canvas = this.#canvas
    if (!container || !canvas || this.#baseWidth === 0 || this.#baseHeight === 0) {
      this.#scheduleDebouncedRepaint()
      return
    }

    // Determine client anchor point
    let clientX = 0
    let clientY = 0
    if (anchor) {
      clientX = anchor.clientX
      clientY = anchor.clientY
    } else {
      const rect = container.getBoundingClientRect()
      clientX = rect.left + rect.width / 2
      clientY = rect.top + rect.height / 2
    }

    const containerRect = container.getBoundingClientRect()
    const offsetX = clientX - containerRect.left
    const offsetY = clientY - containerRect.top

    // Current scroll positions
    const scrollLeft = container.scrollLeft
    const scrollTop = container.scrollTop

    // Calculate new layout size
    const newWidth = Math.floor(this.#baseWidth * this.#zoom)
    const newHeight = Math.floor(this.#baseHeight * this.#zoom)

    // Update canvas style dimensions immediately (smooth CSS scaling)
    canvas.style.width = `${newWidth}px`
    canvas.style.height = `${newHeight}px`

    // Dynamically adjust margin to center canvas or align to start to avoid flexbox cropping
    const containerWidth = container.clientWidth
    const containerHeight = container.clientHeight
    const marginX = newWidth < containerWidth ? 'auto' : '0'
    const marginY = newHeight < containerHeight ? 'auto' : '0'
    canvas.style.margin = `${marginY} ${marginX}`

    // Adjust scroll positions so anchor point stays in place
    const ratio = this.#zoom / oldZoom
    container.scrollLeft = (scrollLeft + offsetX) * ratio - offsetX
    container.scrollTop = (scrollTop + offsetY) * ratio - offsetY

    // Schedule debounced high-resolution repaint
    this.#scheduleDebouncedRepaint()
  }

  zoomBy(factor: number, anchor?: { clientX: number; clientY: number }): void {
    this.setZoom(this.#zoom * factor, anchor)
  }

  #onWheel = (event: WheelEvent): void => {
    if (!event.ctrlKey) return
    event.preventDefault()
    const anchor = { clientX: event.clientX, clientY: event.clientY }
    // ctrlKey + wheel is the trackpad pinch signal; deltaY < 0 means zoom in.
    this.zoomBy(Math.exp(-event.deltaY * 0.01), anchor)
  }

  #onGestureStart = (event: any): void => {
    event.preventDefault()
    this.#gestureBaseZoom = this.#zoom
  }

  #onGestureChange = (event: any): void => {
    event.preventDefault()
    if (event.scale) {
      const anchor =
        event.clientX !== undefined && event.clientY !== undefined
          ? { clientX: event.clientX, clientY: event.clientY }
          : undefined
      this.setZoom(this.#gestureBaseZoom * event.scale, anchor)
    }
  }

  /** Coalesce repaint requests into one per frame so resize cannot loop. */
  #scheduleRepaint(): void {
    if (this.#repaintHandle != null) return
    this.#repaintHandle = requestAnimationFrame(() => {
      this.#repaintHandle = null
      void this.#paint()
    })
  }

  #scheduleDebouncedRepaint(): void {
    if (this.#debounceTimeoutHandle != null) {
      clearTimeout(this.#debounceTimeoutHandle)
    }
    this.#debounceTimeoutHandle = window.setTimeout(() => {
      this.#debounceTimeoutHandle = null
      void this.#paint()
    }, 200) // 200ms debounce
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

    if (this.#debounceTimeoutHandle != null) {
      clearTimeout(this.#debounceTimeoutHandle)
      this.#debounceTimeoutHandle = null
    }

    const page: PDFPageProxy = await this.#doc.getPage(this.#page)

    // Dark themes invert the page. Born-digital pages are inverted per-color so
    // photos keep their true colors (`smart`); a page that is just a scanned
    // image has no such structure, so the whole canvas is inverted instead.
    const dark = isDarkTheme(this.#theme)
    const mode = dark ? await this.#pageMode(page) : 'smart'

    const unscaled = page.getViewport({ scale: 1 })
    // Fit the whole page to the viewport (contain), so it is never cut off in
    // wide or short windows; zoom scales up from there (FR-READ-05). The
    // border-box from getBoundingClientRect is immune to any scrollbar.
    const box = this.#viewport ? this.#viewport.getBoundingClientRect() : { width: 0, height: 0 }
    const fitWidth = box.width || window.innerWidth || unscaled.width
    const fitHeight = box.height || window.innerHeight || unscaled.height

    // On mobile devices, let the PDF fill up the full width instead of fitting to page height.
    const isMobile = window.innerWidth < 768
    const baseScale = isMobile
      ? (fitWidth / unscaled.width)
      : Math.min(fitWidth / unscaled.width, fitHeight / unscaled.height)

    // Save base dimensions for layout scaling
    this.#baseWidth = unscaled.width * baseScale
    this.#baseHeight = unscaled.height * baseScale

    const dpr = window.devicePixelRatio || 1
    const viewport = page.getViewport({ scale: baseScale * this.#zoom })

    const canvas = this.#canvas
    const container = this.#container

    // Reset scroll positions if the page actually changed since last paint
    if (container && this.#page !== this.#lastPaintedPage) {
      this.#lastPaintedPage = this.#page
      container.scrollLeft = 0
      container.scrollTop = 0
    }

    // Render to an offscreen canvas first to prevent flicker or blank screen.
    const offscreenCanvas = document.createElement('canvas')
    const offscreenContext = offscreenCanvas.getContext('2d')
    if (!offscreenContext) return

    offscreenCanvas.width = Math.floor(viewport.width * dpr)
    offscreenCanvas.height = Math.floor(viewport.height * dpr)

    // Smart invert remaps each color as pdf.js paints it onto the active theme;
    // images, drawn directly, are left alone. A fresh offscreen context each
    // paint needs no teardown.
    if (dark && mode === 'smart') installColorInvert(offscreenContext, this.#theme)

    this.#renderTask = page.render({
      canvas: offscreenCanvas,
      canvasContext: offscreenContext,
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

    // Update main canvas size and copy content in a single frame.
    canvas.width = offscreenCanvas.width
    canvas.height = offscreenCanvas.height
    canvas.style.width = `${Math.floor(viewport.width)}px`
    canvas.style.height = `${Math.floor(viewport.height)}px`

    // A scanned page carries no per-color structure, so flip the whole canvas
    // at display time; `hue-rotate` keeps any color tint roughly true.
    canvas.style.filter = dark && mode === 'scanned' ? 'invert(1) hue-rotate(180deg)' : ''

    const context = canvas.getContext('2d')
    if (context) {
      context.drawImage(offscreenCanvas, 0, 0)
    }

    // Dynamic margin calculation to center canvas if smaller than container,
    // and align to start (0) if larger, avoiding flexbox cropping.
    if (canvas && container) {
      const canvasWidth = Math.floor(viewport.width)
      const canvasHeight = Math.floor(viewport.height)
      const containerWidth = container.clientWidth
      const containerHeight = container.clientHeight

      const marginX = canvasWidth < containerWidth ? 'auto' : '0'
      const marginY = canvasHeight < containerHeight ? 'auto' : '0'
      canvas.style.margin = `${marginY} ${marginX}`
    }

    this.#emitLocation()
  }

  /**
   * Decide how a page inverts under a dark theme. A page that paints raster
   * images but shows no text is a scan, which must be inverted whole; anything
   * with text is born-digital and inverts per-color so its images stay true.
   */
  async #pageMode(page: PDFPageProxy): Promise<'smart' | 'scanned'> {
    const cached = this.#pageModes.get(page.pageNumber)
    if (cached) return cached
    const { fnArray } = await page.getOperatorList()
    let images = 0
    let texts = 0
    for (const fn of fnArray) {
      if (fn === pdfjs.OPS.paintImageXObject || fn === pdfjs.OPS.paintInlineImageXObject) images += 1
      else if (fn === pdfjs.OPS.showText) texts += 1
    }
    const mode = images > 0 && texts === 0 ? 'scanned' : 'smart'
    this.#pageModes.set(page.pageNumber, mode)
    return mode
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
