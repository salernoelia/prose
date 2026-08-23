/**
 * The typefaces the reader offers, re-declared for the section iframe.
 *
 * foliate renders each section in its own iframe document, and `@font-face`
 * rules are per-document: the families the app loads for its own UI do not
 * exist inside that document. Every bundled family is therefore declared again
 * in the stylesheet injected into each section. The URLs are absolutized
 * because the section document's base URL is a `blob:` URL, against which the
 * bundler's relative asset paths do not resolve.
 */
import literataRegular from '@fontsource/literata/files/literata-latin-400-normal.woff2?url'
import literataItalic from '@fontsource/literata/files/literata-latin-400-italic.woff2?url'
import literataBold from '@fontsource/literata/files/literata-latin-700-normal.woff2?url'
import literataBoldItalic from '@fontsource/literata/files/literata-latin-700-italic.woff2?url'
import interRegular from '@fontsource/inter/files/inter-latin-400-normal.woff2?url'
import interItalic from '@fontsource/inter/files/inter-latin-400-italic.woff2?url'
import interBold from '@fontsource/inter/files/inter-latin-700-normal.woff2?url'
import interBoldItalic from '@fontsource/inter/files/inter-latin-700-italic.woff2?url'
import outfitRegular from '@fontsource/outfit/files/outfit-latin-400-normal.woff2?url'
import outfitBold from '@fontsource/outfit/files/outfit-latin-700-normal.woff2?url'

interface FontFace {
  family: string
  weight: 400 | 700
  style: 'normal' | 'italic'
  url: string
}

const FACES: FontFace[] = [
  { family: 'Literata', weight: 400, style: 'normal', url: literataRegular },
  { family: 'Literata', weight: 400, style: 'italic', url: literataItalic },
  { family: 'Literata', weight: 700, style: 'normal', url: literataBold },
  { family: 'Literata', weight: 700, style: 'italic', url: literataBoldItalic },
  { family: 'Inter', weight: 400, style: 'normal', url: interRegular },
  { family: 'Inter', weight: 400, style: 'italic', url: interItalic },
  { family: 'Inter', weight: 700, style: 'normal', url: interBold },
  { family: 'Inter', weight: 700, style: 'italic', url: interBoldItalic },
  // Outfit ships no italic; the browser slants the upright cut instead.
  { family: 'Outfit', weight: 400, style: 'normal', url: outfitRegular },
  { family: 'Outfit', weight: 700, style: 'normal', url: outfitBold },
]

/**
 * Fallbacks per bundled family, so a section still renders in something close
 * while the face loads. Georgia and other system families are used as they are.
 */
const STACKS: Record<string, string> = {
  Literata: `'Literata', Georgia, serif`,
  Inter: `'Inter', system-ui, sans-serif`,
  Outfit: `'Outfit', system-ui, sans-serif`,
  Georgia: `Georgia, serif`,
}

function absolute(url: string): string {
  return new URL(url, document.baseURI).href
}

/** The `@font-face` block to inject into a section document. */
export function readerFontFaceCss(): string {
  return FACES.map(
    (face) => `
    @font-face {
      font-family: '${face.family}';
      font-style: ${face.style};
      font-weight: ${face.weight};
      font-display: swap;
      src: url('${absolute(face.url)}') format('woff2');
    }`,
  ).join('')
}

/** The font stack for a settings font family, quoted and ready for CSS. */
export function readerFontStack(fontFamily: string): string {
  return STACKS[fontFamily] ?? `'${fontFamily}', serif`
}
