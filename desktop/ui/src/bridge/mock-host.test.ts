import { describe, expect, it, vi } from 'vitest'
import { MockHost } from './mock-host'

describe('MockHost', () => {
  it('starts with one New Tab that is active', async () => {
    const host = new MockHost()
    const tabs = await host.tabs.list()
    expect(tabs).toHaveLength(1)
    expect(tabs[0]!.active).toBe(true)
    expect(tabs[0]!.url).toBeNull()
    expect(tabs[0]!.title).toBeNull()
  })

  it('opens tabs with unique ids and activates the new one', async () => {
    const host = new MockHost()
    const second = await host.tabs.open()
    const tabs = await host.tabs.list()
    expect(tabs).toHaveLength(2)
    expect(tabs.find((tab) => tab.id === second.id)?.active).toBe(true)
  })

  it('never has zero tabs: closing the last tab opens a New Tab', async () => {
    const host = new MockHost()
    const only = (await host.tabs.list())[0]!
    await host.tabs.close(only.id)
    const tabs = await host.tabs.list()
    expect(tabs).toHaveLength(1)
    expect(tabs[0]!.url).toBeNull()
  })

  it('records https navigations and emits did-navigate', async () => {
    const host = new MockHost()
    const events: string[] = []
    host.onEvent((event) => {
      if (event.type === 'did-navigate') events.push(event.url ?? 'null')
    })
    const tab = (await host.tabs.list())[0]!
    await host.nav.navigate(tab.id, 'https://example.org/page')
    const updated = (await host.tabs.list())[0]!
    expect(updated.url).toBe('https://example.org/page')
    expect(events).toEqual(['https://example.org/page'])
  })

  it('refuses non-http schemes', async () => {
    const host = new MockHost()
    const tab = (await host.tabs.list())[0]!
    await expect(host.nav.navigate(tab.id, 'file:///etc/passwd')).rejects.toThrow(
      /refuses to load/,
    )
    await expect(host.nav.navigate(tab.id, 'javascript:alert(1)')).rejects.toThrow()
  })

  it('walks history back and forward', async () => {
    const host = new MockHost()
    const tab = (await host.tabs.list())[0]!
    await host.nav.navigate(tab.id, 'https://a.example/')
    await host.nav.navigate(tab.id, 'https://b.example/')
    await host.nav.back(tab.id)
    expect((await host.tabs.list())[0]!.url).toBe('https://a.example/')
    await host.nav.forward(tab.id)
    expect((await host.tabs.list())[0]!.url).toBe('https://b.example/')
  })

  it('reload re-announces the current URL; activate switches tabs', async () => {
    const host = new MockHost()
    const listener = vi.fn()
    host.onEvent(listener)
    const first = (await host.tabs.list())[0]!
    await host.nav.navigate(first.id, 'https://example.com/')
    listener.mockClear()
    await host.nav.reload(first.id)
    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({ type: 'did-navigate', url: 'https://example.com/' }),
    )
    const other = await host.tabs.open()
    await host.tabs.activate(first.id)
    expect((await host.tabs.list()).find((tab) => tab.id === first.id)?.active).toBe(
      true,
    )
    expect((await host.tabs.list()).find((tab) => tab.id === other.id)?.active).toBe(
      false,
    )
  })

  it('close on a missing tab rejects', async () => {
    const host = new MockHost()
    await expect(host.tabs.close('nope')).rejects.toThrow(/no tab/)
  })

  it('unsubscribe stops event delivery', async () => {
    const host = new MockHost()
    const listener = vi.fn()
    const off = host.onEvent(listener)
    off()
    await host.tabs.open()
    expect(listener).not.toHaveBeenCalled()
  })
})
