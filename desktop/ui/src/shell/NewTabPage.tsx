import { useUiStore } from '../state/ui-store'
import logoUrl from '../../../resources/branding/logo-original.png'

/**
 * The New Tab / Start page skeleton (Design.md 6.1 area, M1.2 scope):
 * the owner's logo and nothing invented — no speed dial, no metrics,
 * no wallpaper. When the dev harness MockHost backs the UI, it says so
 * in plain language instead of pretending an engine is attached.
 *
 * @returns The start page filling the content area.
 */
export function NewTabPage() {
  const usingMockHost = useUiStore((state) => state.usingMockHost)

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 bg-surface">
      <img
        src={logoUrl}
        alt="Xenon"
        className="h-24 w-24 rounded-panel shadow-extruded"
      />
      <h1 className="font-headline text-xl text-on-surface">Xenon</h1>
      {usingMockHost && (
        <p className="font-mono text-xs text-outline" role="note">
          Development harness — no engine attached. Pages do not load here.
        </p>
      )}
    </div>
  )
}
