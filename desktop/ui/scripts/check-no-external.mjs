// Scans the built output (dist/) for anything that would make an outbound
// network request — the automated half of the "zero external requests" rule
// (Design.md 4.1; the other half is the strict CSP in index.html).
//
// What counts as a violation:
//   HTML: src=, href= or srcset= pointing at http(s)
//   CSS:  url(http...), @import url(http...) / @import "http..."
//   JS:   network primitives called with an http(s) literal, and http(s)
//         string literals that reference loadable asset files
// Non-load-bearing https strings (for example React's error-message links)
// do not make requests and are allowed; the CSP is the runtime backstop.
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, extname } from 'node:path'

const distDir = join(import.meta.dirname, '..', 'dist')

const HTML_RULES = [/\b(?:src|href|srcset)\s*=\s*["']https?:\/\//gi]
const CSS_RULES = [/url\(\s*['"]?https?:/gi, /@import\s+["']?https?:/gi]
const JS_RULES = [
  /\b(?:fetch|XMLHttpRequest|WebSocket|EventSource)\s*\(\s*["'`]https?:/gi,
  /\bimport\s*\(\s*["'`]https?:/gi,
  /["'`]https?:\/\/[^"'`]+\.(?:js|mjs|css|woff2?|ttf|otf|png|jpe?g|gif|svg|ico|wasm)(?:\?[^"'`]*)?["'`]/gi,
]

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    if (statSync(full).isDirectory()) yield* walk(full)
    else yield full
  }
}

const rulesByExt = {
  '.html': HTML_RULES,
  '.css': CSS_RULES,
  '.js': JS_RULES,
  '.mjs': JS_RULES,
}

const findings = []
for (const filePath of walk(distDir)) {
  const rules = rulesByExt[extname(filePath).toLowerCase()]
  if (!rules) continue
  const text = readFileSync(filePath, 'utf8')
  for (const rule of rules) {
    rule.lastIndex = 0
    for (let match = rule.exec(text); match !== null; match = rule.exec(text)) {
      findings.push(`${filePath}: ${match[0].slice(0, 90)}`)
    }
  }
}

if (findings.length > 0) {
  console.error('External request sources found in dist/ (Design.md 4.1):')
  for (const finding of findings) console.error(`  ${finding}`)
  process.exit(1)
}
console.log('no-external-requests: dist/ is fully self-contained')
