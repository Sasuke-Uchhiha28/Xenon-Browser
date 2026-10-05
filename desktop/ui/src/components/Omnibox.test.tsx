import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Omnibox } from './Omnibox'

describe('Omnibox', () => {
  it('names the default engine in the placeholder (Design.md 4.5)', () => {
    render(<Omnibox onNavigate={() => undefined} currentUrl={null} />)
    expect(screen.getByRole('searchbox')).toHaveAttribute(
      'placeholder',
      'Search DuckDuckGo or type a link',
    )
  })

  it('navigates with a normalized URL on Enter', async () => {
    const user = userEvent.setup()
    const onNavigate = vi.fn()
    render(<Omnibox onNavigate={onNavigate} currentUrl={null} />)
    const box = screen.getByRole('searchbox')
    await user.type(box, 'example.com{Enter}')
    expect(onNavigate).toHaveBeenCalledWith('https://example.com')
    // The omnibox shows the normalized URL immediately, like a real browser.
    expect(box).toHaveValue('https://example.com')
  })

  it('searches for plain text', async () => {
    const user = userEvent.setup()
    const onNavigate = vi.fn()
    render(<Omnibox onNavigate={onNavigate} currentUrl={null} />)
    await user.type(screen.getByRole('searchbox'), 'privacy browser{Enter}')
    expect(onNavigate).toHaveBeenCalledWith('https://duckduckgo.com/?q=privacy%20browser')
  })

  it('does nothing on empty input', async () => {
    const user = userEvent.setup()
    const onNavigate = vi.fn()
    render(<Omnibox onNavigate={onNavigate} currentUrl={null} />)
    await user.type(screen.getByRole('searchbox'), '{Enter}')
    expect(onNavigate).not.toHaveBeenCalled()
  })

  it('shows the active tab URL when not being edited', async () => {
    const onNavigate = vi.fn()
    const { rerender } = render(<Omnibox onNavigate={onNavigate} currentUrl={null} />)
    const box = screen.getByRole('searchbox')
    expect(box).toHaveValue('')
    rerender(<Omnibox onNavigate={onNavigate} currentUrl="https://example.com/" />)
    expect(box).toHaveValue('https://example.com/')
  })

  it('does not clobber what the user is typing when the URL changes', async () => {
    const user = userEvent.setup()
    const onNavigate = vi.fn()
    const { rerender } = render(<Omnibox onNavigate={onNavigate} currentUrl={null} />)
    const box = screen.getByRole('searchbox')
    await user.click(box)
    await user.type(box, 'privacy')
    rerender(<Omnibox onNavigate={onNavigate} currentUrl="https://example.com/" />)
    expect(box).toHaveValue('privacy')
  })
})
