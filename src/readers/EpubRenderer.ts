/**
 * ePub rendering over foliate-js (architecture section 6, FR-READ-01/04).
 *
 * Wraps the vendored `<foliate-view>` custom element. The whole container is
 * fetched once from its `prose://` URL and unzipped in the WebView; page turns
 * stay renderer-local so they never touch the IPC path (NFR-P-03).
 */
import './vendor/foliate-js/view.js'
import { Overlayer } from './vendor/foliate-js/overlayer.js'
import * as CFI from './vendor/foliate-js/epubcfi.js'
import type {
  Annotatable,
  BookRenderer,
  JumpHistory,
  Locator,
  ReadingStyle,
  TextSelection,
  TocItem,
  ViewportRect,
} from './types'
import { DARK_THEMES, THEME_TOKENS } from './themes'
import { readerFontFaceCss, readerFontStack } from './fonts'

/** Default highlight color, also used as the annotation tint foliate draws. */
const DEFAULT_HIGHLIGHT_COLOR = '#f6c945'

/** The relocate event payload foliate emits on every position change. */
interface RelocateDetail {
  fraction?: number
  cfi?: string
}

/** The subset of foliate's `View` element we drive. */
interface FoliateView extends HTMLElement {
  book: {
    toc?: FoliateTocItem[]
    sections?: FoliateSection[]
    landmarks?: FoliateLandmark[]
  }
  lastLocation?: { cfi?: string }
  history: EventTarget & {
    back(): void
    clear(): void
    pushState(state: unknown): void
    readonly canGoBack: boolean
  }
  open(source: string): Promise<void>
  init(options: { lastLocation?: string; showTextStart?: boolean }): Promise<void>
  goTo(target: string | number): Promise<unknown>
  next(): Promise<void>
  prev(): Promise<void>
  getCFI(index: number, range: Range): string
  addAnnotation(annotation: { value: string; color?: string }): Promise<unknown>
  deleteAnnotation(annotation: { value: string }): Promise<unknown>
  renderer: {
    setStyles?(css: string): void
    setAttribute(name: string, value: string): void
  }
}

/** Detail of foliate's `draw-annotation` event: we supply the draw call. */
interface DrawAnnotationDetail {
  draw(func: unknown, options?: Record<string, unknown>): void
  annotation: { value: string; color?: string }
}

interface FoliateTocItem {
  label?: string
  href?: string
  subitems?: FoliateTocItem[]
}

/** A spine section; `id` is the section's resource href, `linear` its spine flag. */
interface FoliateSection {
  id?: string
  linear?: string
}

/** A landmark from the nav doc or OPF guide; `type` carries the epub:type roles. */
interface FoliateLandmark {
  href?: string
  type?: string | string[]
}

/**
 * Foliate injects this stylesheet into each section. It does two jobs: apply the
 * reader's typography and theme, and normalize away author styling that hurts
 * legibility (hard-coded colors, embedded fonts, exaggerated spacing, artificial
 * centering and indents). The `!important` rules win over all but author `!important`.
 */
function readingCss(style: ReadingStyle): string {
  const font = readerFontStack(style.fontFamily)
  const { bg, fg } = THEME_TOKENS[style.theme]
  return `
    ${readerFontFaceCss()}
    html {
      color-scheme: ${DARK_THEMES.has(style.theme) ? 'dark' : 'light'};
      font-size: ${style.fontSize}px;
      background-color: ${bg} !important;
      color: ${fg} !important;
      -webkit-touch-callout: default !important;
      -webkit-user-select: text !important;
      user-select: text !important;
      /* Kill the browser's ~300ms double-tap-zoom wait before click fires, so
         a tap in a turn zone flips the page immediately. Zoom is off in the
         reader anyway; word double-tap-select is a separate gesture and stays. */
      touch-action: manipulation !important;
    }
    /* Force the reader's font everywhere, overriding embedded fonts, but leave
       genuine monospace content (which a more specific rule re-asserts). */
    * { font-family: ${font} !important; }
    pre, pre *, code, code *, kbd, samp, tt {
      font-family: ui-monospace, SFMono-Regular, Menlo, monospace !important;
    }
    /* Collapse author colors to one readable color: many ePubs hard-code dark
       text that is unreadable on a dark background. Inheriting from the root lets
       the theme color set above propagate; links stay marked by an underline. */
    body, p, li, blockquote, dd, dt, div, span, h1, h2, h3, h4, h5, h6,
    a, em, strong, b, i, u, small, sub, sup, td, th, figcaption, cite, q, mark {
      color: inherit !important;
      background-color: transparent !important;
    }
    body { background-color: ${bg} !important; }
    a { text-decoration: underline; }
    /* Normalize spacing and layout the author may have exaggerated: collapse odd
       letter/word spacing, force the reader's chosen alignment over the book's
       own (some books center or right-align across the whole screen), and drop
       heavy first-line indents and side gutters. */
    p, li, blockquote, dd, dt, div, section, article {
      line-height: ${style.lineHeight} !important;
      text-align: ${style.textAlign} !important;
      letter-spacing: normal !important;
      word-spacing: normal !important;
    }
    /* Headings and captions follow the chosen alignment too, but keep their
       own line-height. */
    h1, h2, h3, h4, h5, h6, figcaption, caption {
      text-align: ${style.textAlign} !important;
    }
    p { text-indent: 0 !important; }
    body, section, article, div, p, blockquote {
      margin-left: 0 !important;
      margin-right: 0 !important;
      max-width: none !important;
    }
    pre { white-space: pre-wrap !important; }
    /* Back every image with a light gray so transparent figures and line art
       (usually authored dark-on-transparent for a white page) stay readable on
       dark themes. On opaque images the backing sits behind the pixels, so it is
       invisible and adds no frame. */
    img, svg, image { background-color: #919191 !important; }
  `
}

function mapToc(items: FoliateTocItem[] | undefined): TocItem[] {
  if (!items) return []
  return items.map((item) => ({
    label: (item.label ?? '').trim(),
    href: item.href ?? '',
    subitems: mapToc(item.subitems),
  }))
}

export class EpubRenderer implements BookRenderer, Annotatable, JumpHistory {
  #view: FoliateView | null = null
  #style: ReadingStyle | null = null
  #locationListeners: Array<(locator: Locator) => void> = []
  #jumpHistoryListeners: Array<(canUndo: boolean) => void> = []
  #selectionListeners: Array<(selection: TextSelection | null) => void> = []
  #highlightClickListeners: Array<(payload: string, rect: ViewportRect) => void> = []
  // Drawn highlights, keyed by their CFI payload, so they can be redrawn each
  // time foliate creates a fresh overlayer for a (re)rendered section.
  #highlights = new Map<string, string>()
  // The currently visible section document, used to read and clear selections.
  #currentDoc: Document | null = null
  #clickedHighlight = false
  #hasActiveSelection = false
  #lastEmittedSelection: TextSelection | null = null

  async mount(container: HTMLElement): Promise<void> {
    const view = document.createElement('foliate-view') as FoliateView
    view.style.width = '100%'
    view.style.height = '100%'
    view.addEventListener('relocate', (event) => {
      const detail = (event as CustomEvent<RelocateDetail>).detail
      this.#emitLocation(detail)
    })
    // foliate hands back a `draw` callback for each annotation; paint a highlight
    // in its stored color (falling back to the default tint).
    view.addEventListener('draw-annotation', (event) => {
      const { draw, annotation } = (event as CustomEvent<DrawAnnotationDetail>).detail
      draw(Overlayer.highlight, { color: annotation.color ?? DEFAULT_HIGHLIGHT_COLOR })
    })
    // A tap on an existing highlight reports its payload so the UI can offer to
    // remove it.
    view.addEventListener('show-annotation', (event) => {
      const { value, range } = (event as CustomEvent<{ value: string; range?: Range }>).detail
      const rect = this.#rectInViewport(range)
      this.#clickedHighlight = true
      for (const listener of this.#highlightClickListeners) listener(value, rect)
    })
    // Each freshly rendered section gets a new overlayer; redraw our highlights
    // into it. addAnnotation resolves each CFI to its section and only draws the
    // ones that belong to the section just created.
    view.addEventListener('create-overlay', () => {
      for (const [value, color] of this.#highlights) {
        void this.#view?.addAnnotation({ value, color })
      }
    })
    view.addEventListener('load', (event) => {
      const { doc, index } = (event as CustomEvent<{ doc: Document; index: number }>).detail
      this.#watchSelection(doc, index)
      doc.addEventListener('keydown', (e) => {
        const target = e.target as HTMLElement | null
        if (target && (
          target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.tagName === 'SELECT' ||
          target.isContentEditable
        )) {
          return
        }
        if (e.key === 'ArrowRight') {
          this.next()
        } else if (e.key === 'ArrowLeft') {
          this.prev()
        }
      })

      let hadSelectionOnPointerDown = false
      // The click action fires immediately: a delay here made every page turn
      // feel laggy. An existing selection still cancels the turn via the guards
      // below (checked at fire time), so long-press and drag selection are safe;
      // a pending pointerdown also clears any in-flight action.
      let pendingClickAction: ReturnType<typeof setTimeout> | null = null
      const clickActionDelay = 0

      doc.addEventListener('pointerdown', () => {
        if (pendingClickAction !== null) {
          clearTimeout(pendingClickAction)
          pendingClickAction = null
        }
        const selection = doc.defaultView?.getSelection()
        hadSelectionOnPointerDown = Boolean((selection && !selection.isCollapsed) || this.#hasActiveSelection)
      })

      doc.addEventListener('click', (e) => {
        pendingClickAction = setTimeout(() => {
          pendingClickAction = null
          const selection = doc.defaultView?.getSelection()
          const isCurrentlySelected = Boolean(selection && !selection.isCollapsed)

          if (this.#hasActiveSelection || isCurrentlySelected || hadSelectionOnPointerDown) {
            hadSelectionOnPointerDown = false
            return
          }

          if (this.#clickedHighlight) {
            this.#clickedHighlight = false
            return
          }

          if ((e.target as HTMLElement).closest('a, button, input, textarea, select')) return

          const img = (e.target as HTMLElement).closest('img, svg image')
          if (img) {
            const src =
              (img as HTMLImageElement).currentSrc ||
              (img as HTMLImageElement).src ||
              img.getAttribute('href') ||
              img.getAttribute('xlink:href') ||
              ''
            if (src) {
              container.dispatchEvent(new CustomEvent('image-click', {
                bubbles: true,
                detail: { src }
              }))
              return
            }
          }

          const frame = doc.defaultView?.frameElement?.getBoundingClientRect()
          const x = (e as MouseEvent).clientX + (frame?.left ?? 0)

          container.dispatchEvent(new CustomEvent('renderer-click', {
            bubbles: true,
            detail: { target: e.target, x }
          }))
        }, clickActionDelay)
      })
    })
    // foliate's history records every jump (goTo for TOC, links, annotations)
    // and folds plain page turns into the top entry via replaceState, so
    // canGoBack flips exactly when there is a jump to undo.
    view.history.addEventListener('index-change', () => {
      for (const listener of this.#jumpHistoryListeners) listener(view.history.canGoBack)
    })

    // On a wide screen foliate caps the text column and leaves gutters that
    // belong to its own shadow DOM, so a tap near the screen edge never reaches
    // the section document and the page would not turn. Clicks inside the frame
    // stay in the frame's document, so anything arriving here landed on a
    // gutter and can be reported straight away, in window coordinates.
    let gutterDownX = 0
    let gutterDownY = 0
    container.addEventListener('pointerdown', (e) => {
      gutterDownX = e.clientX
      gutterDownY = e.clientY
    })
    container.addEventListener('click', (e) => {
      if (Math.abs(e.clientX - gutterDownX) > 5 || Math.abs(e.clientY - gutterDownY) > 5) return
      const selection = this.#currentDoc?.defaultView?.getSelection()
      if (this.#hasActiveSelection || (selection && !selection.isCollapsed)) return
      container.dispatchEvent(new CustomEvent('renderer-click', {
        bubbles: true,
        detail: { target: e.target, x: e.clientX }
      }))
    })

    container.append(view)
    this.#view = view
  }

  async load(source: string): Promise<void> {
    if (!this.#view) throw new Error('EpubRenderer.load called before mount')
    await this.#view.open(source)
    // Exclude a cover page from the reading flow before the first render, so
    // foliate's text-start skips it and next/prev can never land back on it.
    this.#excludeCover()
    // Always paginate as a single page; never spread into two columns.
    this.#view.renderer.setAttribute('max-column-count', '1')
    if (this.#style) this.applyStyle(this.#style)
    await this.#view.init({ showTextStart: true })
  }

  destroy(): void {
    this.#view?.remove()
    this.#view = null
    this.#locationListeners = []
    this.#jumpHistoryListeners = []
    this.#selectionListeners = []
    this.#highlightClickListeners = []
    this.#highlights.clear()
  }

  next(): void {
    // foliate rejects when there is nowhere left to go; that is not an error.
    this.#view?.next().catch(() => {})
  }

  prev(): void {
    this.#view?.prev().catch(() => {})
  }

  async goToLocator(locator: Locator): Promise<void> {
    if (!this.#view) return
    if (locator.payload) await this.#view.goTo(locator.payload)
  }

  async goToHref(href: string): Promise<void> {
    if (!href) return
    await this.#view?.goTo(href)
    // The section just shown can still reflow as its web fonts and our injected
    // styles settle, which shifts where the outline's anchor sits and can leave
    // the wrong page on screen. Re-resolve the same target once the layout is
    // stable so it lands correctly regardless of viewport (font size, margin,
    // window width).
    void this.#fontsReady().then(() => {
      this.#view?.goTo(href).catch(() => {})
    })
  }

  /** Resolve once the current section's fonts have loaded, or right away if none. */
  #fontsReady(): Promise<unknown> {
    const fonts = this.#currentDoc?.fonts
    if (!fonts) return Promise.resolve()
    return fonts.ready.catch(() => {})
  }

  toc(): TocItem[] {
    return mapToc(this.#view?.book.toc)
  }

  onLocationChange(cb: (locator: Locator) => void): void {
    this.#locationListeners.push(cb)
  }

  undoJump(): void {
    this.#view?.history.back()
  }

  onJumpHistoryChange(cb: (canUndo: boolean) => void): void {
    this.#jumpHistoryListeners.push(cb)
  }

  resetJumpHistory(): void {
    const view = this.#view
    if (!view) return
    view.history.clear()
    // Re-seed the history with the current page so the first jump after the
    // reset has an entry to return to (back() needs a predecessor, and page
    // turns keep this entry current via replaceState).
    const cfi = view.lastLocation?.cfi
    if (cfi) view.history.pushState(cfi)
    for (const listener of this.#jumpHistoryListeners) listener(false)
  }

  onSelection(cb: (selection: TextSelection | null) => void): void {
    this.#selectionListeners.push(cb)
  }

  onHighlightClick(cb: (payload: string, rect: ViewportRect) => void): void {
    this.#highlightClickListeners.push(cb)
  }

  addHighlight(payload: string, color: string): void {
    this.#highlights.set(payload, color)
    void this.#view?.addAnnotation({ value: payload, color })
  }

  removeHighlight(payload: string): void {
    this.#highlights.delete(payload)
    void this.#view?.deleteAnnotation({ value: payload })
  }

  clearSelection(): void {
    this.#hasActiveSelection = false
    this.#lastEmittedSelection = null
    this.#currentDoc?.defaultView?.getSelection()?.removeAllRanges()
    this.#emitSelection(null)
  }

  /**
   * Whether a saved CFI lies within the page foliate currently reports. The
   * reported page payload is a range CFI spanning the visible content, so a
   * bookmark matches when its anchor falls between that range's endpoints. This
   * survives re-pagination, where exact CFI equality would not. Falls back to
   * equality if either CFI cannot be parsed.
   */
  samePage(payload: string, pagePayload: string): boolean {
    if (payload === pagePayload) return true
    try {
      const pageStart = CFI.collapse(pagePayload)
      const pageEnd = CFI.collapse(pagePayload, true)
      const anchor = CFI.collapse(payload)
      return CFI.compare(pageStart, anchor) <= 0 && CFI.compare(anchor, pageEnd) <= 0
    } catch {
      return false
    }
  }

  /**
   * Watch a section document for text selections. On each selection settle, the
   * range is turned into a CFI (the same payload format used for positions) and
   * its viewport rect is computed by offsetting the in-iframe rect by the
   * iframe's own position, so the UI can anchor a popover over the selection.
   */
  #watchSelection(doc: Document, index: number): void {
    this.#currentDoc = doc
    // WebKit fires transient collapsed selectionchange events while a drag
    // handle is grabbed or moved. Clearing on the first one flashes the popover
    // off mid-drag, so a clear only lands once the collapse has held briefly.
    let clearTimer: ReturnType<typeof setTimeout> | null = null
    const scheduleClear = () => {
      if (clearTimer !== null) return
      clearTimer = setTimeout(() => {
        clearTimer = null
        const selection = doc.defaultView?.getSelection()
        if (!selection || selection.isCollapsed || !selection.toString().trim()) {
          this.#hasActiveSelection = false
          this.#emitSelection(null)
        }
      }, 200)
    }
    const report = () => {
      const view = this.#view
      const selection = doc.defaultView?.getSelection()
      if (!view || !selection || selection.rangeCount === 0 || selection.isCollapsed) {
        scheduleClear()
        return
      }
      const text = selection.toString().trim()
      if (!text) {
        scheduleClear()
        return
      }
      if (clearTimer !== null) {
        clearTimeout(clearTimer)
        clearTimer = null
      }
      this.#hasActiveSelection = true
      const range = selection.getRangeAt(0)
      const payload = view.getCFI(index, range)
      this.#emitSelection({ payload, text, rect: this.#rectInViewport(range) })
    }

    doc.addEventListener('selectionchange', report)
  }

  #emitSelection(selection: TextSelection | null): void {
    if (selection === null && this.#lastEmittedSelection === null) return
    if (selection !== null && this.#lastEmittedSelection !== null) {
      if (
        selection.payload === this.#lastEmittedSelection.payload &&
        selection.text === this.#lastEmittedSelection.text &&
        Math.abs(selection.rect.x - this.#lastEmittedSelection.rect.x) < 2 &&
        Math.abs(selection.rect.y - this.#lastEmittedSelection.rect.y) < 2 &&
        Math.abs(selection.rect.width - this.#lastEmittedSelection.rect.width) < 2 &&
        Math.abs(selection.rect.height - this.#lastEmittedSelection.rect.height) < 2
      ) {
        return
      }
    }
    this.#lastEmittedSelection = selection
    for (const listener of this.#selectionListeners) listener(selection)
  }

  /**
   * A range's bounding box in the outer viewport. Range rects are relative to
   * their section iframe; offsetting by the iframe's own position lifts them into
   * the coordinate space the popover is positioned in.
   */
  #rectInViewport(range: Range | undefined): ViewportRect {
    if (!range) return { x: 0, y: 0, width: 0, height: 0 }
    const rect = range.getBoundingClientRect()
    const frame = range.startContainer.ownerDocument?.defaultView?.frameElement?.getBoundingClientRect()
    return {
      x: rect.x + (frame?.x ?? 0),
      y: rect.y + (frame?.y ?? 0),
      width: rect.width,
      height: rect.height,
    }
  }

  applyStyle(style: ReadingStyle): void {
    this.#style = style
    const renderer = this.#view?.renderer
    if (!renderer) return
    renderer.setStyles?.(readingCss(style))
    // Margin is foliate's outer gutter in pixels; scale our unit-less setting.
    renderer.setAttribute('margin', `${Math.round(style.margin * 24)}px`)
  }

  #emitLocation(detail: RelocateDetail): void {
    const locator: Locator = {
      payload: detail.cfi ?? '',
      progression: detail.fraction ?? 0,
    }
    for (const listener of this.#locationListeners) listener(locator)
  }

  /**
   * Mark the cover section non-linear. Some books place the cover in the spine
   * as `linear="yes"` with an absolute, full-viewport layout that foliate's
   * paginator cannot render or page past. Excluding it from the linear flow is
   * how foliate is told to skip a section: text-start lands after it and
   * next/prev step over it (paginator skips `linear === 'no'`).
   */
  #excludeCover(): void {
    const book = this.#view?.book
    if (!book) return
    const coverHref = (book.landmarks ?? []).find((mark) => hasRole(mark, 'cover'))?.href
    if (!coverHref) return
    for (const section of book.sections ?? []) {
      if (sameDocument(section.id, coverHref)) section.linear = 'no'
    }
  }
}

/** Whether a landmark carries the given epub:type role. */
function hasRole(mark: FoliateLandmark, role: string): boolean {
  const types = Array.isArray(mark.type) ? mark.type : [mark.type]
  return types.some((type) => typeof type === 'string' && type.includes(role))
}

/**
 * Whether two hrefs point at the same document. Compares the full path without
 * its fragment, and falls back to the filename so a section href and a landmark
 * href that resolve their directories differently still match.
 */
function sameDocument(a: string | undefined, b: string | undefined): boolean {
  if (!a || !b) return false
  const path = (href: string) => href.split('#')[0]
  const file = (href: string) => path(href).split('/').pop()
  return path(a) === path(b) || file(a) === file(b)
}
