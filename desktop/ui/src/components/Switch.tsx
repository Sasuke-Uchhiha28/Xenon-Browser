import { cn } from '../lib/cn'

interface SwitchProps {
  /** Current state of the switch. */
  checked: boolean
  /** Called with the new value when the user toggles. */
  onChange: (checked: boolean) => void
  disabled?: boolean
  /** Accessible name (also the visible tooltip anchor for icon-only use). */
  'aria-label': string
}

/**
 * The kit toggle (Design.md 3.2 recessed track, section 8 knob motion of
 * 150 ms, gunmetal to violet). Rendered as a real button with
 * `role="switch"` so keyboard activation works out of the box.
 *
 * @returns A `<button role="switch">`; throws nothing.
 */
export function Switch({ checked, onChange, disabled, 'aria-label': ariaLabel }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      title={ariaLabel}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={cn(
        'relative h-5 w-9 rounded-pill transition-colors duration-(--xenon-motion-switch)',
        checked ? 'bg-primary-container shadow-glow' : 'bg-surface-lowest shadow-recessed',
        'disabled:cursor-not-allowed disabled:opacity-50 disabled:shadow-none',
      )}
    >
      <span
        className={cn(
          'absolute top-0.5 left-0.5 h-4 w-4 rounded-pill transition-colors duration-(--xenon-motion-switch)',
          checked
            ? 'translate-x-4 bg-primary'
            : 'translate-x-0 bg-surface-highest',
        )}
      />
    </button>
  )
}
