import { Tab } from '../components/Tab'
import { Icon } from '../components/Icon'
import { useUiStore } from '../state/ui-store'

/**
 * Row one of the chrome (32px, Design.md 4.2): the tab strip with the
 * `+` button directly after the last tab and a flexible drag area. The
 * strip never shows zero tabs — the Bridge reopens a New Tab on close.
 *
 * @returns The tab strip row; the hosts draw the real window controls.
 */
export function TabStrip() {
  const tabs = useUiStore((state) => state.tabs)
  const activateTab = useUiStore((state) => state.activateTab)
  const closeTab = useUiStore((state) => state.closeTab)
  const openTab = useUiStore((state) => state.openTab)

  return (
    <div className="flex h-8 items-end gap-1 bg-surface-container-low px-2 pt-1">
      <div role="tablist" aria-label="Open tabs" className="flex items-end gap-1 overflow-hidden">
        {tabs.map((tab) => (
          <Tab
            key={tab.id}
            tab={tab}
            onSelect={(id) => void activateTab(id)}
            onClose={(id) => void closeTab(id)}
          />
        ))}
      </div>
      <button
        type="button"
        title="New tab"
        aria-label="New tab"
        onClick={() => void openTab()}
        className="flex h-6 w-6 items-center justify-center rounded-control text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface"
      >
        <Icon name="add" className="text-[16px]" />
      </button>
      <div className="flex-1 self-stretch" />
    </div>
  )
}
