import { describe, it, expect } from 'vitest'
import {
  wikipediaUrl,
  wiktionaryUrl,
  googleSearchUrl,
  translateUrl,
  defaultTranslationLanguage,
  TRANSLATION_LANGUAGES,
} from '../lib/externalLookup'

describe('externalLookup', () => {
  it('generates correct Wikipedia URLs', () => {
    expect(wikipediaUrl('quantum mechanics')).toBe(
      'https://en.wikipedia.org/w/index.php?search=quantum%20mechanics'
    )
  })

  it('generates correct Wiktionary URLs', () => {
    expect(wiktionaryUrl('serendipity')).toBe(
      'https://en.wiktionary.org/wiki/serendipity'
    )
  })

  it('generates correct Google Search URLs', () => {
    expect(googleSearchUrl('hello world')).toBe(
      'https://www.google.com/search?q=hello%20world'
    )
  })

  it('generates correct Translate URLs', () => {
    expect(translateUrl('hello world', 'de')).toBe(
      'https://translate.google.com/?sl=auto&tl=de&text=hello%20world&op=translate'
    )
  })

  it('provides supported translation languages', () => {
    expect(TRANSLATION_LANGUAGES.length).toBeGreaterThan(10)
    expect(TRANSLATION_LANGUAGES.find((l) => l.value === 'en')).toBeDefined()
  })

  it('determines default translation language safely', () => {
    const lang = defaultTranslationLanguage()
    expect(typeof lang).toBe('string')
  })
})
