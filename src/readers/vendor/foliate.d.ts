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
