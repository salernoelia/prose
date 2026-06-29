// `BookRenderer` implementations (foliate-js for ePub, pdf.js for PDF) and the
// registry that selects one by the format Rust reports.
export type {
  Annotatable,
  BookRenderer,
  Locator,
  ReadingStyle,
  TextSelection,
  ViewportRect,
  TocItem,
  Zoomable,
} from './types'
export { isAnnotatable, isZoomable } from './types'
export { createRenderer } from './registry'
