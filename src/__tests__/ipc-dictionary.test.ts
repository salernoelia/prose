import { describe, it, expect, vi, beforeEach } from 'vitest'
import { dictionaryLookup } from '../ipc/dictionary'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockedInvoke = vi.mocked(invoke)

describe('dictionary IPC wrapper', () => {
  beforeEach(() => {
    mockedInvoke.mockReset()
  })

  it('dictionaryLookup passes the word and returns the senses', async () => {
    const senses = [
      { partOfSpeech: 'noun', gloss: 'a written work', synonyms: ['volume'], examples: [] },
    ]
    mockedInvoke.mockResolvedValue(senses)

    const result = await dictionaryLookup('book')

    expect(mockedInvoke).toHaveBeenCalledWith('dictionary_lookup', { word: 'book' })
    expect(result).toEqual(senses)
  })

  it('dictionaryLookup returns an empty array for an unknown word', async () => {
    mockedInvoke.mockResolvedValue([])
    const result = await dictionaryLookup('zzzz')
    expect(result).toEqual([])
  })
})
