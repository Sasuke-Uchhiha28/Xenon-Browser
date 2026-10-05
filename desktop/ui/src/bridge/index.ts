/**
 * Bridge loader. Returns the host-injected Bridge when one exists
 * (production), otherwise dynamically imports the MockHost dev harness so
 * the UI runs in a plain dev server (Rules.md 1.9) without shipping the
 * mock in the main bundle.
 */
import type { Bridge } from './types'

let cached: Promise<Bridge> | null = null

/** Get the Bridge exactly once per UI session. */
export function getBridge(): Promise<Bridge> {
  if (cached === null) {
    const injected = window.xenonBridge
    if (injected !== undefined) {
      if (injected.apiVersion !== 0) {
        return Promise.reject(
          new Error(
            `Host bridge version ${injected.apiVersion} does not match UI version 0`,
          ),
        )
      }
      cached = Promise.resolve(injected)
    } else {
      cached = import('./mock-host').then((module) => new module.MockHost())
    }
  }
  return cached
}

/** Test helper: forget the cached Bridge so the next call re-resolves. */
export function resetBridgeCache(): void {
  cached = null
}
