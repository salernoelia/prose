/**
 * Dictionary lookup state for the reader (FR-NOTE-03).
 *
 * Holds the looked-up word, its senses, and the rect to anchor the definition
 * popover over. The lookup itself crosses the IPC boundary; this composable just
 * tracks the request so the view stays declarative.
 */
import { ref, shallowRef } from 'vue'
import { dictionaryLookup } from '../ipc/dictionary'
import type { DefinitionDto } from '../ipc/types'
import type { ViewportRect } from '../readers'

export function useDictionary() {
  const word = ref<string | null>(null)
  const definitions = shallowRef<DefinitionDto[]>([])
  const rect = ref<ViewportRect | null>(null)
  const loading = ref(false)

  /** Look up a word and open the definition popover anchored at `at`. */
  async function lookup(term: string, at: ViewportRect) {
    const cleaned = term.trim()
    if (!cleaned) return
    word.value = cleaned
    rect.value = at
    definitions.value = []
    loading.value = true
    // A late response for a superseded word must not overwrite a newer one.
    const requested = cleaned
    try {
      const result = await dictionaryLookup(cleaned)
      if (word.value === requested) definitions.value = result
    } finally {
      if (word.value === requested) loading.value = false
    }
  }

  function clear() {
    word.value = null
    definitions.value = []
    rect.value = null
    loading.value = false
  }

  return { word, definitions, rect, loading, lookup, clear }
}
