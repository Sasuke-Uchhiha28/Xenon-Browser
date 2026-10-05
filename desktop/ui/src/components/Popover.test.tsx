import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Popover } from './Popover'

const ANCHOR = {
  left: 100,
  top: 0,
  right: 132,
  bottom: 40,
  width: 32,
  height: 40,
  x: 100,
  y: 0,
  toJSON: () => undefined,
} as DOMRect

function TestBed({ onClose }: { onClose: () => void }) {
  return (
    <div>
      <button type="button">outside</button>
      <Popover open onClose={onClose} anchor={ANCHOR} label="Test menu">
        <div>menu content</div>
      </Popover>
    </div>
  )
}

describe('Popover', () => {
  beforeEach(() => {
    if (document.getElementById('xenon-overlay') === null) {
      const overlay = document.createElement('div')
      overlay.id = 'xenon-overlay'
      document.body.appendChild(overlay)
    }
  })

  it('renders into the overlay layer, not the component tree (Design.md 5.1)', () => {
    render(<TestBed onClose={vi.fn()} />)
    const overlay = document.getElementById('xenon-overlay')!
    expect(overlay).toContainElement(screen.getByRole('dialog', { name: 'Test menu' }))
  })

  it('positions below the anchor', () => {
    render(<TestBed onClose={vi.fn()} />)
    const dialog = screen.getByRole('dialog', { name: 'Test menu' })
    expect(dialog).toHaveStyle({ top: '44px', left: '100px' })
  })

  it('closes on Escape', async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    render(<TestBed onClose={onClose} />)
    await user.keyboard('{Escape}')
    expect(onClose).toHaveBeenCalled()
  })

  it('closes on outside click but not on inside click', async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    render(<TestBed onClose={onClose} />)
    await user.click(screen.getByRole('dialog'))
    expect(onClose).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'outside' }))
    expect(onClose).toHaveBeenCalled()
  })

  it('renders nothing when closed or without an anchor', () => {
    const { container } = render(
      <Popover open={false} onClose={vi.fn()} anchor={ANCHOR} label="x">
        hi
      </Popover>,
    )
    expect(container).toBeEmptyDOMElement()
  })
})
