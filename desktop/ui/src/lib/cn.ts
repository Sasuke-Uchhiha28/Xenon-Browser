/** Join class names, skipping falsy entries. Tiny local stand-in for the
 * classnames package so the UI carries one dependency fewer. */
export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(' ')
}
