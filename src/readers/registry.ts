/**
 * Selects a `BookRenderer` by the `Format` Rust reports (architecture section 6).
 *
 * Adding a format is one entry here plus its renderer; nothing else in the UI
 * shell changes. The renderers are imported lazily so a session that only opens
 * ePubs never pays to load pdf.js, and vice versa.
 */
import type { Format } from '../ipc/types'
import type { BookRenderer } from './types'

export async function createRenderer(format: Format): Promise<BookRenderer> {
  switch (format) {
    case 'epub': {
      const { EpubRenderer } = await import('./EpubRenderer')
      return new EpubRenderer()
    }
    case 'pdf': {
      const { PdfRenderer } = await import('./PdfRenderer')
      return new PdfRenderer()
    }
    default: {
      const exhaustive: never = format
      throw new Error(`No renderer for format: ${String(exhaustive)}`)
    }
  }
}
