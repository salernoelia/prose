/**
 * Smart dark-mode inversion for the PDF canvas (FR-READ themes).
 *
 * pdf.js executes a page's drawing operators against a 2D context: text and
 * vector graphics are painted with `fillStyle`/`strokeStyle`, while photographs
 * arrive through `drawImage`. We shadow the two color accessors on the context
 * so every color is luminance-inverted on the way in, and leave `drawImage`
 * untouched. The result inverts text, rules, and backgrounds for a dark surface
 * while leaving images in their true colors.
 *
 * Inversion flips HSL lightness only, preserving hue and chroma, so a blue
 * heading stays blue instead of turning orange the way a flat RGB invert would.
 *
 * Only the top-level page context is wrapped. pdf.js renders transparency
 * groups and soft masks to its own internal canvases and composites them back
 * via `drawImage`; leaving those alone keeps soft-mask luminance (which drives
 * alpha) intact, at the cost of the occasional grouped element staying
 * non-inverted.
 */

/** Cache from raw color input to its inverted form; pages re-set colors often. */
const invertCache = new Map<string, string>()

/** A throwaway context used to normalize any CSS color form into rgb(a). */
let normalizer: CanvasRenderingContext2D | null = null

/** Shadow `fillStyle`/`strokeStyle` on one context so string colors invert. */
export function installColorInvert(ctx: CanvasRenderingContext2D): void {
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
        set.call(this, typeof value === 'string' ? invertColor(value) : value)
      },
    })
  }
}

function invertColor(input: string): string {
  let cached = invertCache.get(input)
  if (cached === undefined) {
    cached = computeInvert(input)
    invertCache.set(input, cached)
  }
  return cached
}

function computeInvert(input: string): string {
  normalizer ??= document.createElement('canvas').getContext('2d')
  if (!normalizer) return input
  // Let the browser parse any CSS color (hex, named, hsl, rgb) into rgb(a).
  // An unparseable value leaves the previous fillStyle in place; seed a known
  // one so we fall back predictably rather than inverting stale state.
  normalizer.fillStyle = '#000000'
  normalizer.fillStyle = input
  const [r, g, b, a] = parseColor(normalizer.fillStyle)
  const [ir, ig, ib] = invertLightness(r, g, b)
  if (a < 1) return `rgba(${ir}, ${ig}, ${ib}, ${a})`
  return `#${hex(ir)}${hex(ig)}${hex(ib)}`
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

/** Flip HSL lightness (L -> 1 - L) while keeping hue and saturation. */
function invertLightness(r: number, g: number, b: number): [number, number, number] {
  const rn = r / 255
  const gn = g / 255
  const bn = b / 255
  const max = Math.max(rn, gn, bn)
  const min = Math.min(rn, gn, bn)
  const l = (max + min) / 2
  const nl = 1 - l
  if (max === min) {
    const v = Math.round(nl * 255)
    return [v, v, v]
  }
  // Saturation is independent of lightness given the flip, so reuse it as-is.
  const d = max - min
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
  let h: number
  if (max === rn) h = (gn - bn) / d + (gn < bn ? 6 : 0)
  else if (max === gn) h = (bn - rn) / d + 2
  else h = (rn - gn) / d + 4
  h /= 6
  return hslToRgb(h, s, nl)
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s
  const p = 2 * l - q
  return [
    Math.round(hue(p, q, h + 1 / 3) * 255),
    Math.round(hue(p, q, h) * 255),
    Math.round(hue(p, q, h - 1 / 3) * 255),
  ]
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
  return n.toString(16).padStart(2, '0')
}
