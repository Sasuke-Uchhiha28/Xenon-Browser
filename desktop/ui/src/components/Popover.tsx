import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react'
import { createPortal } from 'react-dom'
import { cn } from '../lib/cn'

interface PopoverProps {
  open: boolean
  onClose: () => void
  /** Bounding rect of the anchor element; the popover opens under it. */
  anchor: DOMRect | null
  /** Accessible name of the dialog. */
  label: string
  children: ReactNode
  className?: string
}

/**
 * Floating panel rendered into the `#xenon-overlay` layer above all web
 * content (Design.md 5.1: never shares layout space with the web view).
 * Closes on outside click and Escape, fades and slides 4px in 120 ms
 * (Design.md 8), uses the level-2 floating shadow.
 *
 * @returns The portal content, or nothing when closed.
 */
export function Popover({ open, onClose, anchor, label, children, className }: PopoverProps) {
  const panelRef = useRef<HTMLDivElement>(null)
  const [position, setPosition] = useState<{ top: number; left: number } | null>(null)

  useLayoutEffect(() => {
    if (!open || anchor === null) {
      setPosition(null)
      return
    }
    const panel = panelRef.current
    const width = panel?.offsetWidth ?? 224
    const left = Math.min(anchor.left, window.innerWidth - width - 8)
    setPosition({ top: anchor.bottom + 4, left: Math.max(8, left) })
  }, [open, anchor])

  useEffect(() => {
    if (!open) return
    const onPointerDown = (event: PointerEvent) => {
      if (panelRef.current && event.target instanceof Node) {
        if (!panelRef.current.contains(event.target)) onClose()
      }
    }
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose()
    }
    document.addEventListener('pointerdown', onPointerDown)
    document.addEventListener('keydown', onKeyDown)
    return () => {
      document.removeEventListener('pointerdown', onPointerDown)
      document.removeEventListener('keydown', onKeyDown)
    }
  }, [open, onClose])

  // Focus the panel when it opens so Escape works without a prior click.
  useEffect(() => {
    if (open) panelRef.current?.focus()
  }, [open])

  if (!open || anchor === null) return null

  return createPortal(
    <div
      ref={panelRef}
      role="dialog"
      aria-label={label}
      tabIndex={-1}
      style={{ top: position?.top, left: position?.left }}
      className={cn(
        'bevel-border absolute z-[1000] rounded-panel bg-surface-container shadow-floating',
        'animate-popover-in py-1 outline-none',
        className,
      )}
    >
      {children}
    </div>,
    document.getElementById('xenon-overlay') ?? document.body,
  )
}
