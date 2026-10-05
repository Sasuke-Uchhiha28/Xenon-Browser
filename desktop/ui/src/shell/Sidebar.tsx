import { SidebarItemList } from '../components/SidebarItem'
import { Icon } from '../components/Icon'
import { useUiStore } from '../state/ui-store'

/** Top navigation entries (Design.md 5.1); only Browse works in M1.2. */
const MAIN_ITEMS = [
  { icon: 'public', label: 'Browse', disabled: false },
  { icon: 'workspaces', label: 'Workspaces', disabled: true },
  { icon: 'shield', label: 'Privacy Shield', disabled: true },
  { icon: 'terminal', label: 'Inspector', disabled: true },
  { icon: 'key', label: 'Passwords', disabled: true },
  { icon: 'settings', label: 'Settings', disabled: true },
] as const

/** Bottom utility entries; the privacy-level and engine badges stay hidden
 * until real data exists (Design.md 4.3 / UI-08). */
const BOTTOM_ITEMS = [
  { icon: 'history', label: 'History', disabled: true },
  { icon: 'download', label: 'Downloads', disabled: true },
  { icon: 'extension', label: 'Extensions', disabled: true },
] as const

/**
 * The left sidebar (Design.md 4.2 / 5.1): a 56px icon rail by default,
 * expandable to 256px with a 180ms ease-out; the choice is remembered in
 * memory for now and moves to core settings in M1.3 (ENG-04).
 *
 * @returns The sidebar column.
 */
export function Sidebar() {
  const sidebarExpanded = useUiStore((state) => state.sidebarExpanded)
  const toggleSidebar = useUiStore((state) => state.toggleSidebar)

  return (
    <nav
      aria-label="Sidebar"
      className="flex flex-col gap-1 bg-surface-container-lowest p-1 transition-[width] duration-(--xenon-motion-sidebar) ease-out"
      style={{ width: sidebarExpanded ? '256px' : '56px' }}
    >
      <button
        type="button"
        title={sidebarExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
        aria-label={sidebarExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
        aria-expanded={sidebarExpanded}
        onClick={toggleSidebar}
        className="flex h-9 items-center justify-center rounded-control text-on-surface-variant hover:bg-surface-container hover:text-on-surface"
      >
        <Icon name={sidebarExpanded ? 'chevron_left' : 'chevron_right'} className="text-[18px]" />
      </button>
      <SidebarItemList
        items={MAIN_ITEMS.map((item) => ({ ...item }))}
        collapsed={!sidebarExpanded}
        activeLabel="Browse"
      />
      <div className="mt-auto flex flex-col gap-1">
        <SidebarItemList
          items={BOTTOM_ITEMS.map((item) => ({ ...item }))}
          collapsed={!sidebarExpanded}
        />
      </div>
    </nav>
  )
}
