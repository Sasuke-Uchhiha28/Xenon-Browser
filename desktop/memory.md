# Project state

- **Phase 1, M1.1 (repo and tooling): DONE 2026-10-05** — structure, checks,
  lint CI, logo saved, toolchain complete, pushed, CI green.
- **Phase 1, M1.2 (shared UI foundation): DONE 2026-10-05** — Vite + React +
  Tailwind in `desktop/ui/`, tokens.css generated from Design.md §3, fonts
  self-hosted (subset Material Symbols), Bridge v0 types + MockHost, 8-piece
  component kit, shell (72px two-row chrome, 56px rail), New Tab skeleton
  with owner logo, strict CSP, 53 tests green, build self-contained,
  verified visually in a real browser.
- **Next: M1.3 (core skeleton)** — Rust workspace, JSON-RPC over stdio,
  SQLCipher, keystore, URL-rejecting logger.
- **No browser exists yet.** First runnable skeleton: M1.4 (Blink) / M1.5 (Gecko).

# Decisions

- 2026-10-05: Repo root is the `Xenon V3` folder; all desktop code under
  `desktop/` per Phases.md M1.1. `android/` is empty and untouched.
- 2026-10-05: Owner's logo saved byte-identical (md5 23d63d19d5f6eac46338acbac4d19002)
  as `desktop/resources/branding/logo-original.png` (1024x1024 PNG, RGBA).
  Other sizes generated only by script (M5.6), never redrawn.
- 2026-10-05: Pre-commit checks are stdlib-only Python in `desktop/tools/`
  (hook: `git config core.hooksPath .githooks`, already set). ruff is CI +
  optional local. The checks scan `git ls-files` when `.git` exists.
- 2026-10-05: GitHub Actions pinned by commit SHA (checkout v4 =
  11d5960a326750d5838078e36cf38b85af677262, setup-python v5 =
  a26af69be951a213d495a4c3e4e4022e16d87065, setup-node v4 =
  49933ea5288caeca8642d1e84afbd3f7d6820020).
- 2026-10-05: `.gitattributes` forces LF for text files; .bat/.ps1 keep CRLF.
- 2026-10-05: **UI framework: React 19 + Zustand** (both MIT) — Design.md §9
  and Architecture.md §2 leave it open; chosen for ecosystem maturity and
  testing-library support. Kept for the project's lifetime (Rules 4.2).
- 2026-10-05: **Fonts self-hosted from pinned sources** (Space Grotesk and
  Geist variable woff2, JetBrains Mono 400/500/600, Material Symbols subset
  to 18 icons = 2.2 KB). Regenerate with `python desktop/tools/fetch-fonts.py`
  (pinned URLs + sha256 in `desktop/tools/fonts-manifest.json`; icon list in
  that script). **OWNER DECISION NEEDED: fonts are SIL OFL 1.1, which is not
  in the Rules.md 8.1 allowed list** — the PRD mandates these typefaces;
  recommendation: add OFL to the allowed list (standard permissive font
  license). Licenses ship in `desktop/ui/public/fonts/licenses/`.
- 2026-10-05: Omnibox Enter is handled by an explicit keydown handler, not
  implicit form submission (implicit submission differed between jsdom and
  Chromium; hosts may differ again). Form submit kept as backup.
- 2026-10-05: Sidebar expand preference is in-memory only; persistence moves
  to core settings in M1.3 (ENG-04 — no UI-side storage).
- 2026-10-05: SearXNG search URL is a placeholder (`searx.be`) — owner should
  pick the instance in M2.5. Search engines list per Design.md 4.5, DDG default.
- 2026-10-05: MockHost is dynamically imported and lands in its own chunk
  (`dist/assets/mock-host-*.js`); release packaging (M1.6) must exclude that
  chunk. Recorded as a known issue until then.

# How to build and test

From `desktop/ui/` (Node 24, npm):
```bash
npm ci                        # exact deps from lock file
npm run dev                   # dev harness at http://localhost:5173 (MockHost)
npm run build                 # tokens + tsc --noEmit + vite build -> dist/
npm test                      # vitest, 53 tests
npm run check:no-external     # scans dist/ for external request sources
```
From repo root:
```bash
python desktop/tools/check-secrets.py         # pre-commit checks:
python desktop/tools/check-no-url-logging.py  #   run by .githooks/pre-commit
python desktop/tools/check-file-length.py     #   and CI
python -m ruff check desktop/tools
python desktop/tools/fetch-fonts.py           # only when fonts/icons change
```
Nothing Rust exists yet. Engine builds run in GitHub Actions only (M1.4+).

# Environment

Owner's machine (local dev only; engine builds happen in CI):
- Windows 11 (build 26200). Git Bash for the assistant; owner often uses
  PowerShell: give commands one per line, no `&&`. `python` works, `python3` alias absent.
- git 2.54.0.windows.1; Node v24.18.0 / npm 11.16.0; Python 3.12.10 / pip 25.0.1.
- Rust 1.99.0 stable-x86_64-pc-windows-msvc — links correctly (verified).
- VS Build Tools 2026 (18.10.12224.181): MSVC 14.51.36231, Windows SDK, cmake
  and ninja bundled under `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\` (not on PATH).
- ruff 0.16.10, fonttools 4.66.1 + brotli (pip). No `gh` CLI.
- Repo: https://github.com/Sasuke-Uchhiha28/Xenon-Browser (public), branch
  `main`; milestone branches `phase-1/mX.Y-name` merged to main when done.

# Dependencies added

UI (`desktop/ui/package.json`, all pinned exact, lock file committed):
- react 19.3.0, react-dom 19.3.0, zustand 5.0.15 — MIT
- vite 8.3.2, @vitejs/plugin-react 6.1.2, tailwindcss 4.3.3,
  @tailwindcss/vite 4.3.3, typescript 7.0.2 (Apache-2.0), vitest 5.0.3,
  jsdom 30.1.2, @testing-library/react 16.3.3, jest-dom 7.0.1,
  user-event 14.6.7 — MIT except typescript
Python dev tools: ruff 0.16.10 (MIT, in requirements-dev.txt); fonttools
4.66.1 + brotli (MIT, local only, for fetch-fonts.py); pyyaml (local only).
Fonts: SIL OFL 1.1 (x3) + Apache-2.0 (Material Symbols) — see decision above.

# Known issues and blockers

- **Dev server 504 "Outdated Optimize Dep"**: if a Vite dev server survives
  its parent shell and restarts mid-session (e.g. git touching files), the
  browser's cached module graph can point at dead dep hashes and the page
  mounts nothing. Fix: kill the stray node process, delete
  `desktop/ui/node_modules/.vite`, `npm run dev` again. Dev-only; production
  builds are unaffected. Diagnosed and fixed 2026-10-05.
- **OFL font license needs owner ratification** (decision above).
- MockHost chunk must be excluded from release packaging in M1.6.
- SearXNG instance URL placeholder; final list + settings UI in M2.5.
- Design.md §11 has unconfirmed assumptions (dark-only theme, hidden 4.4
  panels, plain-language labels) — treated as decided; owner may override.

# Next steps

1. Start M1.3 (core skeleton) in a fresh session: Rust workspace `core` +
   `core-capi`, JSON-RPC over stdio (versioning, size limits, chunking),
   SQLCipher + migrations + `settings`/`site_prefs`, keystore (Windows +
   Linux), optional master-password wrap, URL-rejecting logger.
   Branch: `phase-1/m1.3-core-skeleton`.
2. Before M1.3: owner ratifies OFL fonts (or picks alternatives).
3. M1.4 will need the Bridge conformance test suite (extend from M1.2 types).

# File map

- `desktop/Rules.md`, `PRD.md`, `Architecture.md`, `Design.md`, `Phases.md` — governing docs (read per milestone)
- `desktop/memory.md` — this file; the only cross-session state
- `desktop/ui/tokens.json` → `scripts/build-tokens.mjs` → `src/styles/tokens.css` — token pipeline (generated file, do not hand-edit)
- `desktop/ui/src/styles/base.css` — Tailwind @theme mapping, focus ring, Reduced-effects overrides
- `desktop/ui/src/styles/fonts.css` — @font-face for the 6 self-hosted fonts
- `desktop/ui/src/bridge/types.ts` — Bridge v0 contract (window, tabs, nav, events); change together with mock-host + hosts + conformance tests (ENG-02)
- `desktop/ui/src/bridge/mock-host.ts` — dev Bridge (in-memory tabs + history, fetches nothing)
- `desktop/ui/src/bridge/index.ts` — host-injected bridge or dynamic MockHost import
- `desktop/ui/src/state/ui-store.ts` — Zustand store mirroring the Bridge
- `desktop/ui/src/components/` — Button, Switch, Card, Tab, Omnibox, SidebarItem, Popover, MetricCard, Icon (+ `icon-glyphs.ts` generated)
- `desktop/ui/src/shell/` — TabStrip, Toolbar, Sidebar, NewTabPage, Shell
- `desktop/ui/src/lib/` — normalize-url, search-engines, cn
- `desktop/ui/public/fonts/` — committed woff2 + licenses (from fetch-fonts.py)
- `desktop/tools/` — check-secrets.py, check-no-url-logging.py, check-file-length.py, fetch-fonts.py, fonts-manifest.json, requirements-dev.txt
- `.github/workflows/lint.yml` — lint job + ui job (typecheck, build, tests, no-external)
- `.githooks/pre-commit` — runs the three checks; `.gitignore`, `.gitattributes`, `LICENSE` (MPL-2.0), `README.md`, `pyproject.toml`
- `desktop/resources/branding/logo-original.png` — THE logo, never modify
- `desktop/gecko/ blink/ core/ launcher/ packaging/` — empty (.gitkeep) until M1.3+
