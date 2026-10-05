import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Tab } from './Tab'
import type { TabInfo } from '../bridge/types'

function makeTab(overrides: Partial<TabInfo> = {}): TabInfo {
  return { id: 'tab-1', title: null, url: null, active: false, loading: false, ...overrides }
}

describe('Tab', () => {
  it('shows "New Tab" when the engine has no title yet (no fake data)', () => {
    render(<Tab tab={makeTab()} onSelect={vi.fn()} onClose={vi.fn()} />)
    expect(screen.getByRole('tab', { name: 'New Tab' })).toBeInTheDocument()
  })

  it('shows the page title when the engine provides one', () => {
    render(
      <Tab tab={makeTab({ title: 'Example Domain' })} onSelect={vi.fn()} onClose={vi.fn()} />,
    )
    expect(screen.getByRole('tab', { name: 'Example Domain' })).toBeInTheDocument()
  })

  it('selects on click and exposes aria-selected', async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()
    render(
      <Tab tab={makeTab({ active: true })} onSelect={onSelect} onClose={vi.fn()} />,
    )
    expect(screen.getByRole('tab')).toHaveAttribute('aria-selected', 'true')
    await user.click(screen.getByRole('tab'))
    expect(onSelect).toHaveBeenCalledWith('tab-1')
  })

  it('closes via the close button without selecting', async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()
    const onClose = vi.fn()
    render(<Tab tab={makeTab()} onSelect={onSelect} onClose={onClose} />)
    await user.click(screen.getByRole('button', { name: 'Close New Tab' }))
    expect(onClose).toHaveBeenCalledWith('tab-1')
    expect(onSelect).not.toHaveBeenCalled()
  })
})
