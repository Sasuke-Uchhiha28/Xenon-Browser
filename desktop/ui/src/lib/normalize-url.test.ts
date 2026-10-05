import { describe, expect, it } from 'vitest'
import { normalizeUrlInput } from './normalize-url'
import { SEARCH_ENGINES } from './search-engines'

const ddg = SEARCH_ENGINES[0]!

describe('normalizeUrlInput', () => {
  it('passes full URLs through unchanged', () => {
    expect(normalizeUrlInput('https://example.com/a?b=1', ddg)).toBe(
      'https://example.com/a?b=1',
    )
    expect(normalizeUrlInput('http://localhost:8080/x', ddg)).toBe(
      'http://localhost:8080/x',
    )
  })

  it('adds https:// to bare domains', () => {
    expect(normalizeUrlInput('example.com', ddg)).toBe('https://example.com')
    expect(normalizeUrlInput('example.com/path?q=1', ddg)).toBe(
      'https://example.com/path?q=1',
    )
    expect(normalizeUrlInput('localhost:3000', ddg)).toBe('https://localhost:3000')
  })

  it('keeps IPv6 literals', () => {
    expect(normalizeUrlInput('[::1]:8080', ddg)).toBe('https://[::1]:8080')
  })

  it('searches for plain words and multi-word input', () => {
    expect(normalizeUrlInput('privacy browser', ddg)).toBe(
      'https://duckduckgo.com/?q=privacy%20browser',
    )
    expect(normalizeUrlInput('  xenon  ', ddg)).toBe(
      'https://duckduckgo.com/?q=xenon',
    )
  })

  it('uses the selected engine for searches', () => {
    const mojeek = SEARCH_ENGINES.find((engine) => engine.id === 'mojeek')!
    expect(normalizeUrlInput('test', mojeek)).toBe('https://www.mojeek.com/search?q=test')
  })

  it('treats dangerous schemes as search text, never as navigation', () => {
    expect(normalizeUrlInput('javascript:alert(1)', ddg)).toBe(
      `https://duckduckgo.com/?q=${encodeURIComponent('javascript:alert(1)')}`,
    )
  })

  it('empty input searches for the empty query instead of throwing', () => {
    expect(normalizeUrlInput('', ddg)).toBe('https://duckduckgo.com/?q=')
    expect(normalizeUrlInput('   ', ddg)).toBe('https://duckduckgo.com/?q=')
  })
})
