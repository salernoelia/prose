/**
 * Dictionary IPC wrapper: the only place `invoke` appears for word lookup
 * (FR-NOTE-03).
 *
 * The Rust core owns the bundled data set and its index; this just marshals the
 * query. An unknown word resolves to an empty array, never an error.
 */
import { invoke } from '@tauri-apps/api/core'
import type { DefinitionDto } from './types'

/** Look up the senses of a word in the offline dictionary. */
export function dictionaryLookup(word: string): Promise<DefinitionDto[]> {
  return invoke('dictionary_lookup', { word })
}
