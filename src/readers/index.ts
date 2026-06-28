// `BookRenderer` implementations (foliate-js for ePub, pdf.js for PDF) and the
// registry that selects one by the format Rust reports.
export type { BookRenderer, Locator, ReadingStyle, TocItem, Zoomable } from './types'
export { isZoomable } from './types'
export { createRenderer } from './registry'
