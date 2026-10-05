import { DEFAULT_ENGINE, type SearchEngine } from './search-engines'

/**
 * Turn omnibox input into a URL, like mainstream browsers do: real URLs
 * pass through (http/https get no rewrite), anything that looks like a
 * domain gets https:// in front, everything else becomes a search on the
 * given engine. IPv6 literals and `localhost` count as addresses.
 */
export function normalizeUrlInput(
  input: string,
  engine: SearchEngine = DEFAULT_ENGINE,
): string {
  const text = input.trim()
  if (text === '') return engineSearchUrl('', engine)
  if (/^https?:\/\//i.test(text)) return text
  if (looksLikeAddress(text)) return `https://${text}`
  return engineSearchUrl(text, engine)
}

/** True when the input is an address rather than a search term. */
function looksLikeAddress(text: string): boolean {
  if (/^\[?[0-9a-fA-F:.]+\]?(:\d+)?$/.test(text) && text.includes(':')) {
    return true // IPv6 literal, with optional port
  }
  if (/^localhost(:\d+)?$/i.test(text)) return true
  const host = text.split(/[/?#]/, 1)[0] ?? ''
  const dot = host.lastIndexOf('.')
  return dot > 0 && dot < host.length - 1 && !/\s/.test(host)
}

/** Build a search URL for `query`, URL-encoded, on the given engine. */
export function engineSearchUrl(
  query: string,
  engine: SearchEngine = DEFAULT_ENGINE,
): string {
  return engine.searchUrl.replace('%s', encodeURIComponent(query))
}
