import type { TabInfo } from '../bridge/types'
import { cn } from '../lib/cn'
import { Icon } from './Icon'

interface TabProps {
  tab: TabInfo
  onSelect: (tabId: string) => void
  onClose: (tabId: string) => void
}

/**
 * One tab in the strip (Design.md 4.2 row one, 32px tall). The active tab
 * uses the level-2 floating shadow, hover raises the surface, close is an
 * icon button with a tooltip. No favicon yet: nothing fake is shown when
 * the engine has not provided one (Design.md 4.3).
 *
 * @returns A `<div role="tab">` wrapping a select button and a close button.
 */
export function Tab({ tab, onSelect, onClose }: TabProps) {
  return (
    <div
      className={cn(
        'group flex h-6 max-w-48 min-w-24 items-center rounded-control',
        'transition-colors duration-(--xenon-motion-hover)',
        tab.active
          ? 'bg-surface-high shadow-floating'
          : 'bg-surface-container shadow-extruded hover:bg-surface-container-high',
      )}
    >
      <button
        type="button"
        role="tab"
        aria-selected={tab.active}
        title={tab.title ?? 'New Tab'}
        onClick={() => onSelect(tab.id)}
        className="flex-1 truncate px-2 text-left font-mono text-xs text-on-surface"
      >
        {tab.title ?? 'New Tab'}
      </button>
      <button
        type="button"
        title="Close tab"
        aria-label={`Close ${tab.title ?? 'New Tab'}`}
        onClick={() => onClose(tab.id)}
        className="mr-1 flex h-4 w-4 items-center justify-center rounded-full text-on-surface-variant opacity-0 group-hover:opacity-100 focus-visible:opacity-100 hover:text-on-surface"
      >
        <Icon name="close" className="text-[14px]" />
      </button>
    </div>
  )
}
