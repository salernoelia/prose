/**
 * The renderer contract shared by every format (architecture section 6).
 *
 * A `BookRenderer` lives entirely in the WebView: it paints content and reports
 * the current position. One implementation per format, selected by the `Format`
 * Rust reports. Book bytes reach it only through `prose://` URLs, never `invoke`.
 */

/** A format-neutral reading position, mirroring the Rust `Locator`. */
export interface Locator {
  /** Opaque, renderer-produced payload (a CFI for ePub, a page index for PDF). */
  payload: string
  /** Fraction of the way through the book, in `[0, 1]`. */
  progression: number
}

/** One table-of-contents entry. `subitems` nest for multi-level outlines. */
export interface TocItem {
  label: string
  /** Opaque destination the producing renderer understands. */
  href: string
  subitems: TocItem[]
}

/** Typography applied to reflowable content. PDF ignores everything but is given the shape. */
export interface ReadingStyle {
  fontFamily: string
  fontSize: number
  lineHeight: number
  margin: number
  /** Active reading theme; drives background and text color inside the ePub iframe. */
  theme: 'light' | 'dark' | 'sepia'
}

/** Renders a single book and reports its position. */
export interface BookRenderer {
  /** Attach to a host element. Call once before {@link load}. */
  mount(container: HTMLElement): Promise<void>
  /** Open the book at a `prose://` URL. */
  load(source: string): Promise<void>
  /** Tear down listeners, workers, and DOM. Idempotent. */
  destroy(): void

  next(): void
  prev(): void

  /** Jump to a previously reported position. */
  goToLocator(locator: Locator): Promise<void>
  /** Jump to a table-of-contents destination. */
  goToHref(href: string): Promise<void>
  /** The current table of contents, available after {@link load}. */
  toc(): TocItem[]

  /** Register a callback fired whenever the position changes. */
  onLocationChange(cb: (locator: Locator) => void): void
  /** Apply typography. A no-op for fixed-layout formats. */
  applyStyle(style: ReadingStyle): void
}

/** Optional capability for fixed-layout renderers that support zoom (FR-READ-05). */
export interface Zoomable {
  /** Set the zoom multiplier, where 1 is the default fit-to-page. */
  setZoom(zoom: number): void
  /** Multiply the current zoom by `factor` (e.g. 1.25 to zoom in). */
  zoomBy(factor: number): void
}

/** Narrow a renderer to {@link Zoomable} when it supports zoom. */
export function isZoomable(renderer: BookRenderer): renderer is BookRenderer & Zoomable {
  return typeof (renderer as Partial<Zoomable>).setZoom === 'function'
}
