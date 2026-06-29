/**
 * ePub rendering over foliate-js (architecture section 6, FR-READ-01/04).
 *
 * Wraps the vendored `<foliate-view>` custom element. The whole container is
 * fetched once from its `prose://` URL and unzipped in the WebView; page turns
 * stay renderer-local so they never touch the IPC path (NFR-P-03).
 */
import './vendor/foliate-js/view.js'
import { Overlayer } from './vendor/foliate-js/overlayer.js'
import type {
  Annotatable,
  BookRenderer,
  Locator,
  ReadingStyle,
  TextSelection,
  TocItem,
  ViewportRect,
} from './types'

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

// Theme token maps: background color and text color injected directly into the
// ePub iframe so the reading surface matches the app shell's active theme.
const THEME_TOKENS = {
  light: { bg: '#faf9f5', fg: '#18181b' },
  dark: { bg: '#121212', fg: '#e4e4e7' },
  sepia: { bg: '#d0b580', fg: '#36291b' },
} as const

/**
 * Foliate injects this stylesheet into each section. It does two jobs: apply the
 * reader's typography and theme, and normalize away author styling that hurts
 * legibility (hard-coded colors, embedded fonts, exaggerated spacing, artificial
 * centering and indents). The `!important` rules win over all but author `!important`.
 */
function readingCss(style: ReadingStyle): string {
  const font = style.fontFamily
  const { bg, fg } = THEME_TOKENS[style.theme]
  return `
    html {
      color-scheme: ${style.theme === 'dark' ? 'dark' : 'light'};
      font-size: ${style.fontSize}px;
      background-color: ${bg} !important;
      color: ${fg} !important;
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
       letter/word spacing, left-align body text some books center across the
       whole screen, and drop heavy first-line indents and side gutters. */
    p, li, blockquote, dd, dt, div, section, article {
      line-height: ${style.lineHeight};
      text-align: start !important;
      letter-spacing: normal !important;
      word-spacing: normal !important;
    }
    p { text-indent: 0 !important; }
    body, section, article, div, p, blockquote {
      margin-left: 0 !important;
      margin-right: 0 !important;
      max-width: none !important;
    }
    pre { white-space: pre-wrap !important; }
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

export class EpubRenderer implements BookRenderer, Annotatable {
  #view: FoliateView | null = null
  #style: ReadingStyle | null = null
  #locationListeners: Array<(locator: Locator) => void> = []
  #selectionListeners: Array<(selection: TextSelection | null) => void> = []
  #highlightClickListeners: Array<(payload: string, rect: ViewportRect) => void> = []
  // Drawn highlights, keyed by their CFI payload, so they can be redrawn each
  // time foliate creates a fresh overlayer for a (re)rendered section.
  #highlights = new Map<string, string>()
  // The currently visible section document, used to read and clear selections.
  #currentDoc: Document | null = null
  #clickedHighlight = false

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

      let hadSelection = false
      doc.addEventListener('pointerdown', () => {
        const selection = doc.defaultView?.getSelection()
        hadSelection = selection ? !selection.isCollapsed : false
      })

      doc.addEventListener('click', (e) => {
        setTimeout(() => {
          if (hadSelection) {
            hadSelection = false
            return
          }
          if (this.#clickedHighlight) {
            this.#clickedHighlight = false
            return
          }

          const selection = doc.defaultView?.getSelection()
          if (selection && !selection.isCollapsed) return

          if ((e.target as HTMLElement).closest('a, button, input, textarea, select')) return

          container.dispatchEvent(new CustomEvent('renderer-click', {
            bubbles: true,
            detail: { target: e.target }
          }))
        }, 0)
      })
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
    if (href) await this.#view?.goTo(href)
  }

  toc(): TocItem[] {
    return mapToc(this.#view?.book.toc)
  }

  onLocationChange(cb: (locator: Locator) => void): void {
    this.#locationListeners.push(cb)
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
    this.#currentDoc?.defaultView?.getSelection()?.removeAllRanges()
  }

  /**
   * Watch a section document for text selections. On each selection settle, the
   * range is turned into a CFI (the same payload format used for positions) and
   * its viewport rect is computed by offsetting the in-iframe rect by the
   * iframe's own position, so the UI can anchor a popover over the selection.
   */
  #watchSelection(doc: Document, index: number): void {
    this.#currentDoc = doc
    const report = () => {
      const view = this.#view
      const selection = doc.defaultView?.getSelection()
      if (!view || !selection || selection.rangeCount === 0 || selection.isCollapsed) {
        this.#emitSelection(null)
        return
      }
      const text = selection.toString().trim()
      if (!text) {
        this.#emitSelection(null)
        return
      }
      const range = selection.getRangeAt(0)
      const payload = view.getCFI(index, range)
      this.#emitSelection({ payload, text, rect: this.#rectInViewport(range) })
    }
    // pointerup catches mouse and touch drags; selectionchange clears the popover
    // when the user taps away and the selection collapses.
    doc.addEventListener('pointerup', () => setTimeout(report, 0))
    doc.addEventListener('selectionchange', () => {
      const selection = doc.defaultView?.getSelection()
      if (!selection || selection.isCollapsed) this.#emitSelection(null)
    })
  }

  #emitSelection(selection: TextSelection | null): void {
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
