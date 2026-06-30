/**
 * External reference lookups for selected text (FR-NOTE-03 adjacent): open a
 * word or passage in a web dictionary, encyclopedia, search, or translator in
 * the user's default browser. The renderer only builds the URL and hands it to
 * the opener plugin; nothing here touches the domain core.
 */
import { openUrl } from '@tauri-apps/plugin-opener'

/**
 * Languages the Translate action can target. The codes are Google Translate
 * `tl` values. Google is the only provider: its `?text=` URL is a universal
 * link, so the opener hands it to the Google Translate app when installed and
 * falls back to the website otherwise. DeepL is intentionally not offered;
 * its prefill lives in the URL hash fragment, which the DeepL app ignores.
 */
export const TRANSLATION_LANGUAGES = [
  { label: 'English', value: 'en' },
  { label: 'Spanish', value: 'es' },
  { label: 'French', value: 'fr' },
  { label: 'German', value: 'de' },
  { label: 'Italian', value: 'it' },
  { label: 'Portuguese', value: 'pt' },
  { label: 'Dutch', value: 'nl' },
  { label: 'Russian', value: 'ru' },
  { label: 'Polish', value: 'pl' },
  { label: 'Turkish', value: 'tr' },
  { label: 'Arabic', value: 'ar' },
  { label: 'Hindi', value: 'hi' },
  { label: 'Chinese (Simplified)', value: 'zh-CN' },
  { label: 'Chinese (Traditional)', value: 'zh-TW' },
  { label: 'Japanese', value: 'ja' },
  { label: 'Korean', value: 'ko' },
] as const

/** A Google Translate target language code, e.g. `de` or `zh-CN`. */
export type TranslationLanguage = (typeof TRANSLATION_LANGUAGES)[number]['value']

/** The best default target language for this device, falling back to English. */
export function defaultTranslationLanguage(): TranslationLanguage {
  const locale = (typeof navigator !== 'undefined' ? navigator.language : 'en') || 'en'
  const base = locale.toLowerCase().split('-')[0]
  const match = TRANSLATION_LANGUAGES.find((l) => l.value.toLowerCase().split('-')[0] === base)
  return match ? match.value : 'en'
}

/** Wikipedia search for a term. */
export function wikipediaUrl(term: string): string {
  return `https://en.wikipedia.org/w/index.php?search=${encodeURIComponent(term)}`
}

/** Wiktionary entry for a term. */
export function wiktionaryUrl(term: string): string {
  return `https://en.wiktionary.org/wiki/${encodeURIComponent(term)}`
}

/** Google web search for a term or passage. */
export function googleSearchUrl(term: string): string {
  return `https://www.google.com/search?q=${encodeURIComponent(term)}`
}

/** Translate a term or passage into the target language via Google Translate. */
export function translateUrl(term: string, target: TranslationLanguage): string {
  const text = encodeURIComponent(term)
  return `https://translate.google.com/?sl=auto&tl=${target}&text=${text}&op=translate`
}

/** Open a built lookup URL in the default browser, swallowing opener failures. */
export async function openExternal(url: string): Promise<void> {
  try {
    await openUrl(url)
  } catch (err) {
    console.error('Failed to open external URL:', err)
  }
}
