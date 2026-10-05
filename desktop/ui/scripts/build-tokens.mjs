// Generates src/styles/tokens.css from tokens.json (Design.md section 3).
// Run via `npm run tokens`; it runs automatically before dev, build and test.
// Never edit src/styles/tokens.css by hand — edit tokens.json instead.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const tokens = JSON.parse(readFileSync(join(here, '..', 'tokens.json'), 'utf8'))

/** Convert a camelCase or kebab token key to the kebab-case CSS suffix. */
const kebab = (key) => key.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()

const lines = [
  '/* GENERATED from tokens.json (Design.md section 3). Do not edit by hand:',
  '   run `npm run tokens` from desktop/ui. */',
  ':root {',
]
for (const [group, values] of Object.entries(tokens)) {
  if (group === 'meta') continue
  for (const [key, value] of Object.entries(values)) {
    lines.push(`  --xenon-${kebab(group)}-${kebab(key)}: ${value};`)
  }
}
lines.push('}')
lines.push('')

const outPath = join(here, '..', 'src', 'styles', 'tokens.css')
mkdirSync(dirname(outPath), { recursive: true })
writeFileSync(outPath, lines.join('\n'), 'utf8')
console.log(`tokens.css written (${lines.length} lines)`)
