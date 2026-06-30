/**
 * Ambient declaration for the vendored foliate-js modules, which ship as plain
 * ESM with no types. We only consume the `<foliate-view>` custom element and a
 * handful of its methods, typed locally in `EpubRenderer.ts`; importing the
 * module here is purely for its side effect of registering the element.
 */
declare module '*/foliate-js/view.js'

/**
 * The overlayer module exposes the `Overlayer` class whose static draw methods
 * (e.g. `highlight`) we pass to foliate's `draw-annotation` callback. We treat
 * the draw functions as opaque tokens, so a minimal shape is enough.
 */
declare module '*/foliate-js/overlayer.js' {
  export const Overlayer: {
    highlight: unknown
    underline: unknown
    outline: unknown
  }
}

/**
 * The epubcfi module parses and compares CFIs. We use it to test whether a saved
 * location still belongs to the current page after re-pagination. `collapse`
 * reduces a range CFI to one of its endpoints; `compare` orders two CFIs.
 */
declare module '*/foliate-js/epubcfi.js' {
  export function collapse(cfi: string, toEnd?: boolean): string
  export function compare(a: string, b: string): number
}
