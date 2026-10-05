import { TabStrip } from './TabStrip'
import { Toolbar } from './Toolbar'
import { Sidebar } from './Sidebar'
import { NewTabPage } from './NewTabPage'
import { useUiStore } from '../state/ui-store'

/**
 * The application shell (Design.md 4.2 / 5.1): two chrome rows of 72px
 * total, the sidebar rail, and the content area. Popovers render through
 * a portal into the overlay layer, never into this layout.
 *
 * @returns The full shell; only called after the Bridge handshake.
 */
export function Shell() {
  const activeTabId = useUiStore((state) => state.activeTabId)

  return (
    <div className="flex h-full flex-col">
      <header className="flex flex-col">
        <TabStrip />
        <Toolbar />
      </header>
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="min-w-0 flex-1">
          {activeTabId !== null && <NewTabPage />}
        </main>
      </div>
    </div>
  )
}
