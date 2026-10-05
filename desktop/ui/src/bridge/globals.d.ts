import type { Bridge } from './types'

declare global {
  interface Window {
    /**
     * Injected by the real engine hosts (Blink M1.4, Gecko M1.5) before the
     * UI loads. Absent in the dev server, where the MockHost is used.
     */
    xenonBridge?: Bridge
  }
}

export {}
