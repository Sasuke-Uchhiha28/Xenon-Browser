import { useRef, useState, type ReactNode } from 'react'
import { Icon } from '../components/Icon'
import { Omnibox } from '../components/Omnibox'
import { Popover } from '../components/Popover'
import { useUiStore } from '../state/ui-store'

/** One disabled entry of the main menu; the tooltip says why. */
function MenuNote({ label }: { label: string }) {
  return (
    <div
      title="Not available yet"
      className="flex cursor-not-allowed items-center px-3 py-1.5 text-sm text-outline-variant"
    >
      {label}
    </div>
  )
}

/** A clickable main-menu entry. */
function MenuItem({
  label,
  onClick,
  children,
}: {
  label: string
  onClick: () => void
  children?: ReactNode
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full items-center px-3 py-1.5 text-left text-sm text-on-surface hover:bg-surface-high"
    >
      {children ?? label}
    </button>
  )
}

/**
 * Row two of the chrome (40px, Design.md 4.2): back, forward, reload,
 * the omnibox, and the three-dot menu. The shield indicator will join the
 * omnibox area in Phase 3 when real data exists (Design.md 4.3).
 *
 * @returns The toolbar row with its popover menu.
 */
export function Toolbar() {
  const back = useUiStore((state) => state.back)
  const forward = useUiStore((state) => state.forward)
  const reload = useUiStore((state) => state.reload)
  const navigate = useUiStore((state) => state.navigate)
  const openTab = useUiStore((state) => state.openTab)
  const activeUrl = useUiStore(
    (state) => state.tabs.find((tab) => tab.id === state.activeTabId)?.url ?? null,
  )

  const [menuOpen, setMenuOpen] = useState(false)
  const [anchor, setAnchor] = useState<DOMRect | null>(null)
  const menuButtonRef = useRef<HTMLButtonElement>(null)

  const openMenu = () => {
    setAnchor(menuButtonRef.current?.getBoundingClientRect() ?? null)
    setMenuOpen(true)
  }

  return (
    <div className="relative flex h-10 items-center gap-1 bg-surface-container-low px-2">
      <button
        type="button"
        title="Back"
        aria-label="Back"
        onClick={() => void back()}
        className="flex h-7 w-7 items-center justify-center rounded-control text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      >
        <Icon name="arrow_back" className="text-[18px]" />
      </button>
      <button
        type="button"
        title="Forward"
        aria-label="Forward"
        onClick={() => void forward()}
        className="flex h-7 w-7 items-center justify-center rounded-control text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      >
        <Icon name="arrow_forward" className="text-[18px]" />
      </button>
      <button
        type="button"
        title="Reload"
        aria-label="Reload"
        onClick={() => void reload()}
        className="flex h-7 w-7 items-center justify-center rounded-control text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      >
        <Icon name="refresh" className="text-[18px]" />
      </button>
      <Omnibox
        onNavigate={(url) => void navigate(url)}
        currentUrl={activeUrl}
        className="ml-1"
      />
      <button
        ref={menuButtonRef}
        type="button"
        title="Menu"
        aria-label="Menu"
        aria-haspopup="dialog"
        onClick={openMenu}
        className="flex h-7 w-7 shrink-0 items-center justify-center rounded-control text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      >
        <Icon name="more_vert" className="text-[18px]" />
      </button>
      <Popover
        open={menuOpen}
        onClose={() => setMenuOpen(false)}
        anchor={anchor}
        label="Main menu"
        className="w-56"
      >
        <MenuItem
          label="New tab"
          onClick={() => {
            setMenuOpen(false)
            void openTab()
          }}
        />
        <MenuNote label="New private window" />
        <MenuNote label="New Tor window" />
        <MenuNote label="History" />
        <MenuNote label="Downloads" />
        <MenuNote label="Bookmarks" />
        <MenuNote label="Find in page" />
        <MenuNote label="Settings" />
        <MenuNote label="About Xenon" />
      </Popover>
    </div>
  )
}
