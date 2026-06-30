/**
 * Themed dark-mode inversion for the PDF canvas (FR-READ themes).
 *
 * pdf.js executes a page's drawing operators against a 2D context: text and
 * vector graphics are painted with `fillStyle`/`strokeStyle`, while photographs
 * arrive through `drawImage`. We shadow the two color accessors on the context
 * so every string color is remapped onto the active theme, and leave
 * `drawImage` untouched so images keep their true colors.
 *
 * The remap is not a flat invert. Each color's lightness is mapped onto the
 * gradient from the theme's text color (for dark source colors) to its
 * background color (for light source colors), then blended by saturation:
 * near-grayscale content snaps onto that themed ramp, so a white page becomes
 * exactly the theme background and black text becomes exactly the theme text
 * color (with the theme's tint, e.g. sepia warmth). Saturated content keeps its
 * own hue and only has its lightness compressed into the theme's range, so a
 * blue heading stays blue instead of turning orange.
 *
 * Only the top-level page context is wrapped. pdf.js renders transparency
 * groups and soft masks to its own internal canvases and composites them back
 * via `drawImage`; leaving those alone keeps soft-mask luminance (which drives
 * alpha) intact, at the cost of the occasional grouped element staying
 * non-inverted.
 */
import type { ReadingStyle } from './types'
import { THEME_TOKENS } from './themes'

/** Theme endpoints the source grayscale ramp is mapped onto. */
interface Palette {
  bg: [number, number, number]
  fg: [number, number, number]
  /** HSL lightness of `bg` and `fg`, the ends of the remapped lightness range. */
  lbg: number
  lfg: number
}

/** Cache from `theme + input` to the remapped color; pages re-set colors often. */
const remapCache = new Map<string, string>()
const palettes = new Map<string, Palette>()

/** A throwaway context used to normalize any CSS color form into rgb(a). */
let normalizer: CanvasRenderingContext2D | null = null

/** Shadow `fillStyle`/`strokeStyle` on one context so string colors are themed. */
export function installColorInvert(ctx: CanvasRenderingContext2D, theme: ReadingStyle['theme']): void {
  const palette = paletteFor(theme)
  const proto = Object.getPrototypeOf(ctx) as CanvasRenderingContext2D
  for (const prop of ['fillStyle', 'strokeStyle'] as const) {
    const desc = Object.getOwnPropertyDescriptor(proto, prop)
    if (!desc?.get || !desc.set) continue
    const { get, set } = desc
    Object.defineProperty(ctx, prop, {
      configurable: true,
      get() {
        return get.call(this)
      },
      set(value: unknown) {
        // Gradients and patterns are objects, not strings; pass them through.
        set.call(this, typeof value === 'string' ? remapColor(value, theme, palette) : value)
      },
    })
  }
}

function paletteFor(theme: ReadingStyle['theme']): Palette {
  let palette = palettes.get(theme)
  if (!palette) {
    const bg = hexToRgb(THEME_TOKENS[theme].bg)
    const fg = hexToRgb(THEME_TOKENS[theme].fg)
    palette = { bg, fg, lbg: lightness(bg), lfg: lightness(fg) }
    palettes.set(theme, palette)
  }
  return palette
}

function remapColor(input: string, theme: ReadingStyle['theme'], palette: Palette): string {
  const key = `${theme}\n${input}`
  let cached = remapCache.get(key)
  if (cached === undefined) {
    cached = computeRemap(input, palette)
    remapCache.set(key, cached)
  }
  return cached
}

function computeRemap(input: string, palette: Palette): string {
  normalizer ??= document.createElement('canvas').getContext('2d')
  if (!normalizer) return input
  // Let the browser parse any CSS color (hex, named, hsl, rgb) into rgb(a).
  // An unparseable value leaves the previous fillStyle in place; seed a known
  // one so we fall back predictably rather than remapping stale state.
  normalizer.fillStyle = '#000000'
  normalizer.fillStyle = input
  const [r, g, b, a] = parseColor(normalizer.fillStyle)
  const [h, s, l] = rgbToHsl(r, g, b)

  // The achromatic target: where this lightness lands on the fg -> bg ramp.
  // Dark source (l=0) -> text color, light source (l=1) -> background color.
  const themed: [number, number, number] = [
    lerp(palette.fg[0], palette.bg[0], l),
    lerp(palette.fg[1], palette.bg[1], l),
    lerp(palette.fg[2], palette.bg[2], l),
  ]
  // The colored target: same hue and saturation, lightness compressed into the
  // theme's range so the color reads on the dark surface without going neon.
  const colored = hslToRgb(h, s, lerp(palette.lfg, palette.lbg, l))

  // Blend by saturation: grayscale follows the themed ramp, color keeps itself.
  const out: [number, number, number] = [
    Math.round(lerp(themed[0], colored[0], s)),
    Math.round(lerp(themed[1], colored[1], s)),
    Math.round(lerp(themed[2], colored[2], s)),
  ]
  if (a < 1) return `rgba(${out[0]}, ${out[1]}, ${out[2]}, ${a})`
  return `#${hex(out[0])}${hex(out[1])}${hex(out[2])}`
}

function lerp(from: number, to: number, t: number): number {
  return from + (to - from) * t
}

/** Parse the normalized `#rrggbb` or `rgba(r, g, b, a)` a canvas hands back. */
function parseColor(value: string): [number, number, number, number] {
  if (value.startsWith('#')) {
    return [
      parseInt(value.slice(1, 3), 16),
      parseInt(value.slice(3, 5), 16),
      parseInt(value.slice(5, 7), 16),
      1,
    ]
  }
  const parts = value.match(/[\d.]+/g) ?? ['0', '0', '0']
  return [Number(parts[0]), Number(parts[1]), Number(parts[2]), parts[3] !== undefined ? Number(parts[3]) : 1]
}

function hexToRgb(value: string): [number, number, number] {
  return [
    parseInt(value.slice(1, 3), 16),
    parseInt(value.slice(3, 5), 16),
    parseInt(value.slice(5, 7), 16),
  ]
}

/** HSL lightness of an 8-bit color, in 0..1. */
function lightness(rgb: [number, number, number]): number {
  const max = Math.max(rgb[0], rgb[1], rgb[2]) / 255
  const min = Math.min(rgb[0], rgb[1], rgb[2]) / 255
  return (max + min) / 2
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  const rn = r / 255
  const gn = g / 255
  const bn = b / 255
  const max = Math.max(rn, gn, bn)
  const min = Math.min(rn, gn, bn)
  const l = (max + min) / 2
  if (max === min) return [0, 0, l]
  const d = max - min
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
  let h: number
  if (max === rn) h = (gn - bn) / d + (gn < bn ? 6 : 0)
  else if (max === gn) h = (bn - rn) / d + 2
  else h = (rn - gn) / d + 4
  return [h / 6, s, l]
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  if (s === 0) {
    const v = l * 255
    return [v, v, v]
  }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s
  const p = 2 * l - q
  return [hue(p, q, h + 1 / 3) * 255, hue(p, q, h) * 255, hue(p, q, h - 1 / 3) * 255]
}

function hue(p: number, q: number, t: number): number {
  if (t < 0) t += 1
  if (t > 1) t -= 1
  if (t < 1 / 6) return p + (q - p) * 6 * t
  if (t < 1 / 2) return q
  if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
  return p
}

function hex(n: number): string {
  return Math.max(0, Math.min(255, n)).toString(16).padStart(2, '0')
}
