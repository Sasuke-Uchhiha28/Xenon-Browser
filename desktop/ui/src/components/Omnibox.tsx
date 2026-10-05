import { useEffect, useRef, useState, type FormEvent } from 'react'
import { cn } from '../lib/cn'
import { normalizeUrlInput } from '../lib/normalize-url'
import { DEFAULT_ENGINE } from '../lib/search-engines'
import { Icon } from './Icon'

interface OmniboxProps {
  /** Called with a normalized URL when the user submits. */
  onNavigate: (url: string) => void
  /** URL of the active tab, shown when the user is not editing. */
  currentUrl: string | null
  className?: string
}

/**
 * The URL and search bar (Design.md 4.2 row two). JetBrains Mono, recessed
 * trough, placeholder names the active search engine (Design.md 4.5).
 * Shows the active tab's URL unless the user is editing; the HTTPS padlock
 * stays hidden until the engine reports real status (Design.md 4.3).
 *
 * @returns A `<form role="search">`; submitting empty input does nothing.
 */
export function Omnibox({ onNavigate, currentUrl, className }: OmniboxProps) {
  const [value, setValue] = useState(currentUrl ?? '')
  // Ref, not state: the sync effect must read it without re-firing.
  const editingRef = useRef(false)

  // Follow the active tab's URL unless the user is mid-edit. On submit the
  // normalized URL is shown optimistically, before the engine confirms.
  useEffect(() => {
    if (!editingRef.current) setValue(currentUrl ?? '')
  }, [currentUrl])

  const commit = () => {
    if (value.trim() === '') return
    const url = normalizeUrlInput(value, DEFAULT_ENGINE)
    editingRef.current = false
    setValue(url)
    onNavigate(url)
  }

  const submit = (event: FormEvent) => {
    event.preventDefault()
    commit()
  }

  // Explicit Enter handling: implicit form submission differs between
  // embedding hosts (CEF, Gecko), so the omnibox never relies on it.
  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter') {
      event.preventDefault()
      commit()
    }
  }

  return (
    <form
      role="search"
      onSubmit={submit}
      className={cn(
        'flex h-7 min-w-0 flex-1 items-center gap-2 rounded-control bg-surface-lowest px-2 shadow-recessed',
        className,
      )}
    >
      <Icon name="search" className="shrink-0 text-[16px] text-outline" />
      <input
        type="search"
        value={value}
        onChange={(event) => setValue(event.target.value)}
        onKeyDown={onKeyDown}
        onFocus={() => {
          editingRef.current = true
        }}
        onBlur={() => {
          editingRef.current = false
          setValue(currentUrl ?? '')
        }}
        placeholder={`Search ${DEFAULT_ENGINE.name} or type a link`}
        aria-label="Search or type a link"
        spellCheck={false}
        className="min-w-0 flex-1 bg-transparent font-mono text-sm text-on-surface outline-none placeholder:text-outline [&::-webkit-search-cancel-button]:hidden"
      />
    </form>
  )
}
