import { useEffect } from 'react'
import { getBridge } from './bridge'
import { Shell } from './shell/Shell'
import { useUiStore } from './state/ui-store'

/**
 * Apply the Reduced-effects mode (Design.md 4.7): on when the system asks
 * for reduced motion. A manual override setting arrives with Settings
 * (M5.5); the data-effects attribute is what the CSS keys off.
 */
function useReducedEffects(): void {
  useEffect(() => {
    const query = window.matchMedia('(prefers-reduced-motion: reduce)')
    const apply = () => {
      document.documentElement.dataset.effects = query.matches ? 'reduced' : 'auto'
    }
    apply()
    query.addEventListener('change', apply)
    return () => query.removeEventListener('change', apply)
  }, [])
}

/**
 * Root component: connect the Bridge (host-injected or MockHost in the
 * dev harness), then render the shell. Renders nothing while connecting.
 */
export function App() {
  const bridgeReady = useUiStore((state) => state.bridgeReady)
  const init = useUiStore((state) => state.init)
  useReducedEffects()

  useEffect(() => {
    getBridge()
      .then((bridge) => void init(bridge))
      .catch((error: unknown) => {
        // Surface the failure instead of a silent blank window (Rules.md 4.1).
        document.body.textContent =
          'Xenon UI failed to connect to the engine bridge. See the console.'
        console.error('[xenon] bridge init failed', error)
      })
  }, [init])

  return bridgeReady ? <Shell /> : null
}
