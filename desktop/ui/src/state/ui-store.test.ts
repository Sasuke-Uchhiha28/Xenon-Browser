import { beforeEach, describe, expect, it } from 'vitest'
import { MockHost } from '../bridge/mock-host'
import { useUiStore } from './ui-store'

/** Re-initialize the singleton store against a fresh MockHost per test. */
async function freshStore(): Promise<void> {
  const store = useUiStore.getState()
  await store.init(new MockHost())
}

describe('ui store', () => {
  beforeEach(async () => {
    await freshStore()
  })

  it('init loads tabs, the active tab and the window kind', () => {
    const state = useUiStore.getState()
    expect(state.bridgeReady).toBe(true)
    expect(state.usingMockHost).toBe(true)
    expect(state.windowKind).toBe('standard')
    expect(state.tabs).toHaveLength(1)
    expect(state.activeTabId).toBe(state.tabs[0]!.id)
  })

  it('openTab adds and activates a tab; activateTab moves focus back', async () => {
    await useUiStore.getState().openTab()
    let state = useUiStore.getState()
    expect(state.tabs).toHaveLength(2)
    expect(state.tabs.find((tab) => tab.active)?.id).toBe(state.activeTabId)

    const firstId = state.tabs[0]!.id
    await useUiStore.getState().activateTab(firstId)
    state = useUiStore.getState()
    expect(state.activeTabId).toBe(firstId)
  })

  it('closeTab on the last tab leaves exactly one New Tab', async () => {
    const id = useUiStore.getState().tabs[0]!.id
    await useUiStore.getState().closeTab(id)
    const state = useUiStore.getState()
    expect(state.tabs).toHaveLength(1)
    expect(state.tabs[0]!.url).toBeNull()
  })

  it('navigate updates the active tab URL through the Bridge', async () => {
    await useUiStore.getState().navigate('https://example.com/')
    const state = useUiStore.getState()
    expect(state.tabs.find((tab) => tab.id === state.activeTabId)?.url).toBe(
      'https://example.com/',
    )
  })

  it('back and forward move the active tab through its history', async () => {
    await useUiStore.getState().navigate('https://a.example/')
    await useUiStore.getState().navigate('https://b.example/')
    await useUiStore.getState().back()
    const afterBack = useUiStore.getState()
    expect(afterBack.tabs.find((tab) => tab.active)?.url).toBe('https://a.example/')
    await useUiStore.getState().forward()
    expect(useUiStore.getState().tabs.find((tab) => tab.active)?.url).toBe(
      'https://b.example/',
    )
  })

  it('toggleSidebar flips the remembered width state', () => {
    expect(useUiStore.getState().sidebarExpanded).toBe(false)
    useUiStore.getState().toggleSidebar()
    expect(useUiStore.getState().sidebarExpanded).toBe(true)
  })
})
