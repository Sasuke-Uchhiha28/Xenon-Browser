import { cn } from '../lib/cn'
import { ICON_GLYPHS, type IconName } from './icon-glyphs'

/**
 * Render one glyph of the subset Material Symbols font by codepoint.
 *
 * @param name - Icon name as listed in desktop/tools/fetch-fonts.py.
 * @param className - Size and color utilities (the glyph inherits color).
 * @returns A decorative span; screen readers skip it, callers must give
 *   their buttons accessible names.
 */
export function Icon({
  name,
  className,
}: {
  name: IconName
  className?: string
}) {
  return (
    <span aria-hidden="true" className={cn('icon-font', className)}>
      {ICON_GLYPHS[name]}
    </span>
  )
}
