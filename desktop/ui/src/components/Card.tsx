import type { ReactNode } from 'react'
import { cn } from '../lib/cn'

interface CardProps {
  children: ReactNode
  /** `floating` uses the level-2 shadow for popovers and shield cards. */
  elevation?: 'extruded' | 'floating'
  className?: string
}

/**
 * The kit card: 16px radius on `surface-container` (Design.md 3 radii),
 * extruded or level-2 floating shadow.
 *
 * @param elevation - Defaults to `extruded`.
 * @returns A `<div>` with the card chrome; content is whatever you pass.
 */
export function Card({ children, elevation = 'extruded', className }: CardProps) {
  return (
    <div
      className={cn(
        'rounded-panel bg-surface-container',
        elevation === 'floating' ? 'shadow-floating' : 'shadow-extruded',
        className,
      )}
    >
      {children}
    </div>
  )
}
