import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { App } from './App'

describe('App (dev harness)', () => {
  it('connects to the MockHost and renders the shell', async () => {
    render(<App />)
    // The harness note only appears when the MockHost backs the UI, which
    // proves the shell rendered after the Bridge handshake.
    expect(
      await screen.findByText(/Development harness — no engine attached/),
    ).toBeInTheDocument()
    // The New Tab skeleton shows the owner's logo instead of the mockup "X".
    expect(screen.getByAltText('Xenon')).toBeInTheDocument()
    // Reduced-effects wiring sets the data attribute the CSS keys off.
    expect(document.documentElement.dataset.effects).toBe('auto')
  })

  it('shows one tab that is already active', async () => {
    render(<App />)
    expect(await screen.findByRole('tab', { name: 'New Tab' })).toHaveAttribute(
      'aria-selected',
      'true',
    )
  })
})
