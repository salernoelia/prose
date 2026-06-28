/**
 * Ambient declaration for the vendored foliate-js modules, which ship as plain
 * ESM with no types. We only consume the `<foliate-view>` custom element and a
 * handful of its methods, typed locally in `EpubRenderer.ts`; importing the
 * module here is purely for its side effect of registering the element.
 */
declare module '*/foliate-js/view.js'
