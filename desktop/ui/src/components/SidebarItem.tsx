import type { ReactNode } from 'react'
import { cn } from '../lib/cn'
import { Icon } from './Icon'
import type { IconName } from './icon-glyphs'

interface SidebarItemProps {
  icon: IconName
  /** Visible label when the sidebar is expanded; also the tooltip text. */
  label: string
  active?: boolean
  disabled?: boolean
  /** Collapsed mode shows the icon only, with the label as tooltip. */
  collapsed?: boolean
  onClick?: () => void
}

/**
 * One sidebar entry (Design.md 5.1). Collapsed (56px rail): icon with the
 * label as tooltip; expanded (256px): icon plus label. The active item
 * carries the violet glow — glow is reserved for active/focus states.
 *
 * @returns A `<button>`; disabled items keep their tooltip but no pointer.
 */
export function SidebarItem({
  icon,
  label,
  active = false,
  disabled = false,
  collapsed = false,
  onClick,
}: SidebarItemProps) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      aria-current={active ? 'page' : undefined}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        'flex h-9 items-center rounded-control transition-colors duration-(--xenon-motion-hover)',
        collapsed ? 'w-9 justify-center' : 'w-full gap-2 px-2',
        active
          ? 'bg-surface-high shadow-glow'
          : 'hover:bg-surface-container',
        disabled
          ? 'cursor-not-allowed text-outline-variant hover:bg-transparent'
          : 'text-on-surface-variant hover:text-on-surface',
        active && 'text-on-surface',
      )}
    >
      <Icon name={icon} className="shrink-0 text-[18px]" />
      {!collapsed && <span className="truncate text-sm">{label}</span>}
    </button>
  )
}

/** Convenience wrapper typing for callers that map over item definitions. */
export type SidebarItemData = Pick<SidebarItemProps, 'icon' | 'label'> & {
  disabled?: boolean
}

/** Render a list of {@link SidebarItem} from plain data. */
export function SidebarItemList({
  items,
  collapsed,
  activeLabel,
  onItem,
}: {
  items: SidebarItemData[]
  collapsed: boolean
  activeLabel?: string
  onItem?: (label: string) => void
}): ReactNode {
  return (
    <>
      {items.map((item) => (
        <SidebarItem
          key={item.label}
          icon={item.icon}
          label={item.label}
          collapsed={collapsed}
          disabled={item.disabled}
          active={item.label === activeLabel}
          onClick={item.disabled === true ? undefined : () => onItem?.(item.label)}
        />
      ))}
    </>
  )
}
