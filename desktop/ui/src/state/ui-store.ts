/**
 * UI state store (Zustand) — the single small store library for the UI
 * (Rules.md 4.2). Mirrors the Bridge state (tabs, active tab, window kind)
 * and forwards user intents to the Bridge; the Bridge is the source of
 * truth, the store is its UI-side projection.
 */
import { create } from 'zustand'
import type { Bridge, TabInfo, WindowKind } from '../bridge/types'

interface UiState {
  /** True once the Bridge handshake finished; the shell renders after it. */
  bridgeReady: boolean
  /** True when the dev-harness MockHost (not a real engine) backs the UI. */
  usingMockHost: boolean
  windowKind: WindowKind
  tabs: TabInfo[]
  activeTabId: string | null
  sidebarExpanded: boolean
  /** Connect to a Bridge, load initial state and subscribe to its events. */
  init(bridge: Bridge): Promise<void>
  openTab(url?: string): Promise<void>
  closeTab(tabId: string): Promise<void>
  activateTab(tabId: string): Promise<void>
  /** Navigate the active tab; input is already a normalized URL. */
  navigate(url: string): Promise<void>
  back(): Promise<void>
  forward(): Promise<void>
  reload(): Promise<void>
  toggleSidebar(): void
}

/** In-memory for now: layout preference persistence moves to core settings
 * in M1.3 (ENG-04 — hosts and UI never store data themselves). */
let bridgeRef: Bridge | null = null
let unsubscribe: (() => void) | null = null

export const useUiStore = create<UiState>((set, get) => ({
  bridgeReady: false,
  usingMockHost: false,
  windowKind: 'standard',
  tabs: [],
  activeTabId: null,
  sidebarExpanded: false,

  init: async (bridge) => {
    unsubscribe?.()
    unsubscribe = bridge.onEvent((event) => {
      if (event.type === 'tabs-changed') void refreshTabs(bridge, set)
      if (event.type === 'did-navigate') void refreshTabs(bridge, set)
    })
    bridgeRef = bridge
    const [windowInfo, tabs] = await Promise.all([bridge.window.info(), bridge.tabs.list()])
    set({
      bridgeReady: true,
      usingMockHost: bridge.constructor.name === 'MockHost',
      windowKind: windowInfo.kind,
      tabs,
      activeTabId: tabs.find((tab) => tab.active)?.id ?? null,
    })
  },

  openTab: async (url) => {
    if (!bridgeRef) throw new Error('store not initialized')
    await bridgeRef.tabs.open(url)
    await refreshTabs(bridgeRef, set)
  },

  closeTab: async (tabId) => {
    if (!bridgeRef) throw new Error('store not initialized')
    await bridgeRef.tabs.close(tabId)
    await refreshTabs(bridgeRef, set)
  },

  activateTab: async (tabId) => {
    if (!bridgeRef) throw new Error('store not initialized')
    await bridgeRef.tabs.activate(tabId)
    await refreshTabs(bridgeRef, set)
  },

  navigate: async (url) => {
    const { activeTabId } = get()
    if (!bridgeRef) throw new Error('store not initialized')
    if (activeTabId === null) throw new Error('no active tab')
    await bridgeRef.nav.navigate(activeTabId, url)
    await refreshTabs(bridgeRef, set)
  },

  back: async () => {
    const { activeTabId } = get()
    if (!bridgeRef || activeTabId === null) return
    await bridgeRef.nav.back(activeTabId)
    await refreshTabs(bridgeRef, set)
  },

  forward: async () => {
    const { activeTabId } = get()
    if (!bridgeRef || activeTabId === null) return
    await bridgeRef.nav.forward(activeTabId)
    await refreshTabs(bridgeRef, set)
  },

  reload: async () => {
    const { activeTabId } = get()
    if (!bridgeRef || activeTabId === null) return
    await bridgeRef.nav.reload(activeTabId)
    await refreshTabs(bridgeRef, set)
  },

  toggleSidebar: () => set((state) => ({ sidebarExpanded: !state.sidebarExpanded })),
}))

/** Re-read the tab list from the Bridge into the store. */
async function refreshTabs(
  bridge: Bridge,
  set: (partial: Partial<UiState>) => void,
): Promise<void> {
  const tabs = await bridge.tabs.list()
  set({ tabs, activeTabId: tabs.find((tab) => tab.active)?.id ?? null })
}
