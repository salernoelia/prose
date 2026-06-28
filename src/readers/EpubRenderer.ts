/**
 * ePub rendering over foliate-js (architecture section 6, FR-READ-01/04).
 *
 * Wraps the vendored `<foliate-view>` custom element. The whole container is
 * fetched once from its `prose://` URL and unzipped in the WebView; page turns
 * stay renderer-local so they never touch the IPC path (NFR-P-03).
 */
import './vendor/foliate-js/view.js'
import type { BookRenderer, Locator, ReadingStyle, TocItem } from './types'

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
  renderer: {
    setStyles?(css: string): void
    setAttribute(name: string, value: string): void
  }
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

export class EpubRenderer implements BookRenderer {
  #view: FoliateView | null = null
  #style: ReadingStyle | null = null
  #locationListeners: Array<(locator: Locator) => void> = []

  async mount(container: HTMLElement): Promise<void> {
    const view = document.createElement('foliate-view') as FoliateView
    view.style.width = '100%'
    view.style.height = '100%'
    view.addEventListener('relocate', (event) => {
      const detail = (event as CustomEvent<RelocateDetail>).detail
      this.#emitLocation(detail)
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
