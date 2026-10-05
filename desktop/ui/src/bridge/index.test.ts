import { beforeEach, describe, expect, it } from 'vitest'
import { getBridge, resetBridgeCache } from './index'
import type { Bridge, TabInfo } from './types'
import { BRIDGE_VERSION } from './types'

/** Minimal fake host bridge (distinct from MockHost on purpose: the loader
 * must prefer anything injected on window). */
function fakeHostBridge(): Bridge {
  const tabs: TabInfo[] = [{ id: 'host-1', title: 'Host', url: null, active: true, loading: false }]
  return {
    apiVersion: BRIDGE_VERSION,
    window: { info: async () => ({ id: 'host-window', kind: 'standard' }) },
    tabs: {
      list: async () => tabs,
      open: async () => tabs[0]!,
      close: async () => undefined,
      activate: async () => undefined,
    },
    nav: {
      navigate: async () => undefined,
      back: async () => undefined,
      forward: async () => undefined,
      reload: async () => undefined,
      stop: async () => undefined,
    },
    onEvent: () => () => undefined,
  }
}

describe('getBridge', () => {
  beforeEach(() => {
    resetBridgeCache()
    delete window.xenonBridge
  })

  it('uses the host-injected bridge when present', async () => {
    const injected = fakeHostBridge()
    window.xenonBridge = injected
    const bridge = await getBridge()
    expect(bridge).toBe(injected)
  })

  it('falls back to the MockHost when no host is injected', async () => {
    const bridge = await getBridge()
    expect(bridge.constructor.name).toBe('MockHost')
    const tabs = await bridge.tabs.list()
    expect(tabs).toHaveLength(1)
  })

  it('rejects a host with a mismatched API version', async () => {
    window.xenonBridge = { ...fakeHostBridge(), apiVersion: 99 }
    await expect(getBridge()).rejects.toThrow(/does not match UI version/)
  })
})
