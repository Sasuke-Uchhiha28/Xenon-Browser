/**
 * Bridge API v0 — the only way the UI talks to the engine host (Rules.md
 * ENG-02). One file defines the contract for the real hosts (Gecko M1.5,
 * Blink M1.4), the MockHost dev harness and the conformance tests; a
 * Bridge change updates all of them together.
 */

/** Bridge API major version. Hosts must report the same value. */
export const BRIDGE_VERSION = 0

/** Kind of browser window; private and Tor windows are separate windows. */
export type WindowKind = 'standard' | 'private' | 'tor'

/** Facts about the browser window the UI runs in. */
export interface WindowInfo {
  id: string
  kind: WindowKind
}

/** One browser tab as the engine reports it. */
export interface TabInfo {
  id: string
  /** Page title from the engine; null while unknown (for example New Tab). */
  title: string | null
  /** Current top-level URL; null on the New Tab page. */
  url: string | null
  /** True when this tab is the selected one in the window. */
  active: boolean
  /** True while the engine is still loading the page. */
  loading: boolean
}

/** Emitted when the set of tabs or their order changes. */
export interface TabsChangedEvent {
  type: 'tabs-changed'
}

/** Emitted when a tab navigates; null url/title mean back to New Tab. */
export interface DidNavigateEvent {
  type: 'did-navigate'
  tabId: string
  url: string | null
  title: string | null
}

/** Events the engine pushes to the UI. */
export type BridgeEvent = TabsChangedEvent | DidNavigateEvent

/** Unsubscribe function returned by event subscriptions. */
export type Unsubscribe = () => void

/** Tab management. All methods reject when the host cannot fulfill them. */
export interface TabsApi {
  list(): Promise<TabInfo[]>
  /** Open a new tab, optionally loading `url`; returns the created tab. */
  open(url?: string): Promise<TabInfo>
  close(tabId: string): Promise<void>
  activate(tabId: string): Promise<void>
}

/** Navigation inside one tab. */
export interface NavApi {
  /** Load `url` in the tab. Rejects for URLs the host refuses to load. */
  navigate(tabId: string, url: string): Promise<void>
  back(tabId: string): Promise<void>
  forward(tabId: string): Promise<void>
  reload(tabId: string): Promise<void>
  stop(tabId: string): Promise<void>
}

/** Window-level facts (the hosts own the real window controls). */
export interface WindowApi {
  info(): Promise<WindowInfo>
}

/** The complete Bridge surface the UI may use. Nothing else is allowed. */
export interface Bridge {
  readonly apiVersion: number
  readonly window: WindowApi
  readonly tabs: TabsApi
  readonly nav: NavApi
  /** Subscribe to engine-pushed events; returns an unsubscribe function. */
  onEvent(listener: (event: BridgeEvent) => void): Unsubscribe
}
