import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Switch } from './Switch'

describe('Switch', () => {
  it('toggles and reports the new value', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<Switch checked={false} onChange={onChange} aria-label="Reduced effects" />)
    const sw = screen.getByRole('switch', { name: 'Reduced effects' })
    expect(sw).toHaveAttribute('aria-checked', 'false')
    await user.click(sw)
    expect(onChange).toHaveBeenCalledWith(true)
  })

  it('supports keyboard activation (space)', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<Switch checked onChange={onChange} aria-label="Toggle" />)
    await user.tab()
    await user.keyboard(' ')
    expect(onChange).toHaveBeenCalledWith(false)
  })

  it('does nothing when disabled', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<Switch checked onChange={onChange} disabled aria-label="Toggle" />)
    await user.click(screen.getByRole('switch', { name: 'Toggle' }))
    expect(onChange).not.toHaveBeenCalled()
  })
})
