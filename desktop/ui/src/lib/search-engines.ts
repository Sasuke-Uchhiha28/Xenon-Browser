/** Search engines offered in the omnibox (Design.md 4.5). DuckDuckGo is the
 * default; the list and the placeholder react to the selected engine.
 * NOTE: the SearXNG instance URL is a placeholder decision pending owner
 * input in M2.5 (recorded in memory.md). */
export interface SearchEngine {
  id: 'duckduckgo' | 'searxng' | 'startpage' | 'mojeek' | 'google'
  name: string
  /** Search URL template; `%s` is replaced by the URL-encoded query. */
  searchUrl: string
}

export const SEARCH_ENGINES: SearchEngine[] = [
  {
    id: 'duckduckgo',
    name: 'DuckDuckGo',
    searchUrl: 'https://duckduckgo.com/?q=%s',
  },
  { id: 'searxng', name: 'SearXNG', searchUrl: 'https://searx.be/?q=%s' },
  {
    id: 'startpage',
    name: 'Startpage',
    searchUrl: 'https://www.startpage.com/sp/search?query=%s',
  },
  { id: 'mojeek', name: 'Mojeek', searchUrl: 'https://www.mojeek.com/search?q=%s' },
  { id: 'google', name: 'Google', searchUrl: 'https://www.google.com/search?q=%s' },
]

/** The engine used for omnibox searches; the setting UI arrives in M2.5. */
export const DEFAULT_ENGINE: SearchEngine = SEARCH_ENGINES[0]!
