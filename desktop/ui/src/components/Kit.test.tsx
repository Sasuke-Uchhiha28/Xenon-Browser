import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Button } from './Button'
import { Card } from './Card'
import { MetricCard } from './MetricCard'
import { SidebarItem } from './SidebarItem'

describe('Card', () => {
  it('renders children with the panel radius; floating uses the level-2 shadow', () => {
    const { rerender } = render(<Card>content</Card>)
    const card = screen.getByText('content')
    expect(card).toHaveClass('rounded-panel')
    expect(card).toHaveClass('shadow-extruded')
    rerender(<Card elevation="floating">content</Card>)
    expect(screen.getByText('content')).toHaveClass('shadow-floating')
  })
})

describe('Button', () => {
  it('handles clicks and honors disabled', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    const { rerender } = render(<Button onClick={onClick}>Save</Button>)
    await user.click(screen.getByRole('button', { name: 'Save' }))
    expect(onClick).toHaveBeenCalledTimes(1)
    rerender(
      <Button onClick={onClick} disabled title="Save">
        Save
      </Button>,
    )
    await user.click(screen.getByRole('button', { name: 'Save' }))
    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it('renders the primary variant for the violet fill', () => {
    render(<Button variant="primary">Go</Button>)
    expect(screen.getByRole('button', { name: 'Go' })).toHaveClass('bg-primary-container')
  })
})

describe('MetricCard', () => {
  it('renders nothing when there is no real value (UI-08)', () => {
    const { container } = render(<MetricCard label="Trackers blocked" value={null} />)
    expect(container).toBeEmptyDOMElement()
  })

  it('shows label and value when data exists', () => {
    render(<MetricCard label="Trackers blocked" value="17" />)
    expect(screen.getByText('Trackers blocked')).toBeInTheDocument()
    expect(screen.getByText('17')).toBeInTheDocument()
  })
})

describe('SidebarItem', () => {
  it('collapsed: icon-only with label tooltip; expanded: icon plus label', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    const { rerender } = render(
      <SidebarItem icon="public" label="Browse" collapsed onClick={onClick} />,
    )
    const item = screen.getByRole('button', { name: 'Browse' })
    expect(item).toHaveAttribute('title', 'Browse')
    expect(screen.queryByText('Browse')).not.toBeInTheDocument()
    await user.click(item)
    expect(onClick).toHaveBeenCalled()

    rerender(<SidebarItem icon="public" label="Browse" onClick={onClick} />)
    expect(screen.getByText('Browse')).toBeInTheDocument()
  })

  it('disabled items show the tooltip but cannot be activated', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    render(<SidebarItem icon="shield" label="Privacy Shield" disabled onClick={onClick} />)
    const item = screen.getByRole('button', { name: 'Privacy Shield' })
    expect(item).toBeDisabled()
    await user.click(item)
    expect(onClick).not.toHaveBeenCalled()
  })

  it('marks the active item with aria-current', () => {
    render(<SidebarItem icon="public" label="Browse" active />)
    expect(screen.getByRole('button', { name: 'Browse' })).toHaveAttribute(
      'aria-current',
      'page',
    )
  })
})
