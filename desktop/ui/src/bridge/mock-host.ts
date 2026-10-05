/**
 * MockHost — the development Bridge (Rules.md 1.9). It keeps tabs and a
 * per-tab history in memory, records navigations, and fetches nothing.
 * It is loaded only when no real host has injected `window.xenonBridge`,
 * so it never ships with the browser. Every screen it renders must say
 * plainly that no engine is attached.
 */
import type {
  Bridge,
  BridgeEvent,
  DidNavigateEvent,
  TabInfo,
  Unsubscribe,
} from './types'
import { BRIDGE_VERSION } from './types'

interface MockTab {
  info: TabInfo
  history: string[]
  historyIndex: number
}

export class MockHost implements Bridge {
  readonly apiVersion = BRIDGE_VERSION

  private tabMap = new Map<string, MockTab>()
  private order: string[] = []
  private activeId: string
  private nextId = 1
  private listeners = new Set<(event: BridgeEvent) => void>()

  constructor() {
    const first = this.createTab()
    this.order.push(first.info.id)
    this.activeId = first.info.id
    this.setActive(first.info.id)
  }

  readonly window = {
    info: async () => ({ id: 'mock-window', kind: 'standard' as const }),
  }

  readonly tabs = {
    list: async (): Promise<TabInfo[]> =>
      this.order.map((id) => ({ ...this.requireTab(id).info })),
    open: async (url?: string): Promise<TabInfo> => {
      const tab = this.createTab()
      this.order.push(tab.info.id)
      if (url !== undefined) this.loadUrl(tab, url)
      this.setActive(tab.info.id)
      this.emitTabsChanged()
      return { ...tab.info }
    },
    close: async (tabId: string): Promise<void> => {
      if (!this.tabMap.has(tabId)) {
        throw new Error(`MockHost: no tab ${tabId}`)
      }
      this.tabMap.delete(tabId)
      this.order = this.order.filter((id) => id !== tabId)
      // The window never has zero tabs (Design.md 4.2): reopen a New Tab.
      if (this.order.length === 0) {
        const fresh = this.createTab()
        this.order.push(fresh.info.id)
        this.setActive(fresh.info.id)
      } else if (this.activeId === tabId) {
        this.setActive(this.order[this.order.length - 1]!)
      }
      this.emitTabsChanged()
    },
    activate: async (tabId: string): Promise<void> => {
      if (!this.tabMap.has(tabId)) {
        throw new Error(`MockHost: no tab ${tabId}`)
      }
      this.setActive(tabId)
      this.emitTabsChanged()
    },
  }

  readonly nav = {
    navigate: async (tabId: string, url: string): Promise<void> => {
      const tab = this.requireTab(tabId)
      this.loadUrl(tab, url)
    },
    back: async (tabId: string): Promise<void> => {
      const tab = this.requireTab(tabId)
      if (tab.historyIndex > 0) {
        tab.historyIndex -= 1
        this.applyHistory(tab)
      }
    },
    forward: async (tabId: string): Promise<void> => {
      const tab = this.requireTab(tabId)
      if (tab.historyIndex < tab.history.length - 1) {
        tab.historyIndex += 1
        this.applyHistory(tab)
      }
    },
    reload: async (tabId: string): Promise<void> => {
      const tab = this.requireTab(tabId)
      // Nothing is fetched; re-announce the current URL so the UI reacts.
      this.emitNavigate(tab, tab.info.url)
    },
    stop: async (tabId: string): Promise<void> => {
      const tab = this.requireTab(tabId)
      tab.info = { ...tab.info, loading: false }
      this.emitTabsChanged()
    },
  }

  onEvent(listener: (event: BridgeEvent) => void): Unsubscribe {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  private createTab(): MockTab {
    const id = `mock-tab-${this.nextId}`
    this.nextId += 1
    const tab: MockTab = {
      info: { id, title: null, url: null, active: false, loading: false },
      history: [],
      historyIndex: -1,
    }
    this.tabMap.set(id, tab)
    return tab
  }

  private setActive(tabId: string): void {
    this.activeId = tabId
    for (const id of this.order) {
      const tab = this.tabMap.get(id)
      if (tab) tab.info = { ...tab.info, active: id === tabId }
    }
  }

  /** Record a navigation: append to history (truncating forward entries). */
  private loadUrl(tab: MockTab, url: string): void {
    const parsed = new URL(url)
    if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
      throw new Error(`MockHost refuses to load ${parsed.protocol} URLs`)
    }
    tab.history = [...tab.history.slice(0, tab.historyIndex + 1), url]
    tab.historyIndex = tab.history.length - 1
    this.applyHistory(tab)
  }

  private applyHistory(tab: MockTab): void {
    const url = tab.history[tab.historyIndex] ?? null
    tab.info = { ...tab.info, url, loading: false }
    this.emitNavigate(tab, url)
    this.emitTabsChanged()
  }

  private requireTab(tabId: string): MockTab {
    const tab = this.tabMap.get(tabId)
    if (!tab) throw new Error(`MockHost: no tab ${tabId}`)
    return tab
  }

  private emitTabsChanged(): void {
    for (const listener of this.listeners) listener({ type: 'tabs-changed' })
  }

  private emitNavigate(tab: MockTab, url: string | null): void {
    const event: DidNavigateEvent = {
      type: 'did-navigate',
      tabId: tab.info.id,
      url,
      title: null,
    }
    for (const listener of this.listeners) listener(event)
  }
}
