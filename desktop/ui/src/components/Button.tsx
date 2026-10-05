import type { ButtonHTMLAttributes, ReactNode } from 'react'
import { cn } from '../lib/cn'

type ButtonVariant = 'default' | 'primary' | 'ghost'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual weight: `default` extruded, `primary` violet fill, `ghost` flat. */
  variant?: ButtonVariant
  children: ReactNode
}

const VARIANT_CLASSES: Record<ButtonVariant, string> = {
  default:
    'bg-surface-container text-on-surface shadow-extruded hover:bg-surface-high active:bg-surface-lowest active:shadow-recessed',
  primary:
    'bg-primary-container text-on-primary-container shadow-extruded hover:shadow-glow active:shadow-recessed',
  ghost:
    'bg-transparent text-on-surface hover:bg-surface-container active:bg-surface-lowest',
}

/**
 * The kit button (Design.md section 3 neumorphism, section 8 motion).
 * Press switches to the recessed shadow in 90 ms; disabled buttons drop
 * all shadows. `title` doubles as the tooltip required by Design.md 4.8.
 *
 * @param variant - Defaults to `default`.
 * @param children - Button content (text and/or Icon).
 * @returns A styled `<button>`; errors come from native button semantics.
 */
export function Button({ variant = 'default', className, children, ...rest }: ButtonProps) {
  return (
    <button
      type="button"
      {...rest}
      className={cn(
        'rounded-control px-3 py-1.5 text-sm',
        'transition-colors duration-(--xenon-motion-press)',
        'disabled:cursor-not-allowed disabled:bg-surface-container/50 disabled:text-outline-variant disabled:shadow-none',
        VARIANT_CLASSES[variant],
        className,
      )}
    >
      {children}
    </button>
  )
}
