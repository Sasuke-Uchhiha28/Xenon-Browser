# Project state

- **Phase 1, M1.1 (repo and tooling): DONE 2026-10-05.**
- **Phase 1, M1.2 (shared UI foundation): DONE 2026-10-05** — React + Zustand UI,
  tokens, self-hosted fonts, Bridge v0 + MockHost, shell, 53 tests, CI green.
- **Phase 1, M1.3 (core skeleton): DONE 2026-10-05** — Rust workspace
  `desktop/core/` (crates `xenon-core` + `xenon-core-capi` stub): JSON-RPC 2.0
  over stdio (1 MB line limit, chunking helper, `hello` handshake), SQLCipher
  DB + migrations (`settings`, `site_prefs`), keystore (Windows Credential
  Manager / Linux Secret Service) with Argon2id master-password wrap,
  PRIV-05 rejecting logger. 46 tests green, clippy `-D warnings` clean,
  fmt clean, audited. CI core jobs green on Linux AND Windows (note:
  rustup auto-installs a minimal toolchain in CI, so `rustup component add
  rustfmt clippy` runs explicitly first). Dev-only key override env var
  documented in memory.
- **Phase 1, M1.4 (Blink host skeleton): sub-step 1 of 4 COMPLETE.** Pinned
  CEF 154.0.34+g14c5a08+chromium-154.0.8037.98 (stable, same version both
  platforms, standard distribution, SHA-1 verified from the official
  cef-builds CDN — SHA-1 is the only digest CEF publishes).
  `desktop/tools/fetch-cef.py` downloads, verifies and extracts into the
  git-ignored `desktop/blink/cef/<platform>/` (top-level dir stripped).
  `.github/workflows/blink.yml` builds the UNMODIFIED cefsimple sample on
  Windows and Linux and uploads artifacts. The sample BUILDS on both
  platforms (CI runs c8da63c/c90432e proved it); artifact paths use CEF's
  real output layout `build/tests/<target>/Release/` (learned via
  diagnostic annotations). **Sub-step 1 ACCEPTED** (run 37445970411,
  cefsimple artifacts both OSes).
- **M1.4 sub-step 2 of 4 COMPLETE (commit 4cfe683, CI green, artifacts
  xenon-blink-windows64 190.6 MB / xenon-blink-linux64 367.2 MB).** The
  Xenon host lives in `desktop/blink/app/` (main.cc, xenon_app,
  xenon_handler, ui_scheme_handler; ~900 lines) staged into the CEF tree
  by `desktop/tools/stage-blink.py` as tests/xenon and built by
  blink.yml, which also builds desktop/ui and bundles `ui/` into the
  output. Design: ONE Alloy-style BrowserView for the shared UI
  (xenon://ui/index.html served by a validated local scheme handler —
  standard+CORS+FETCH options, path traversal rejected, strict CSP)
  filling the window; the active tab is a second BrowserView added via
  `CefWindow::AddOverlayView(..., CEF_DOCKING_MODE_CUSTOM, true)` and
  positioned over the content area (constants: chrome 72px, rail 56px —
  Bridge v0 window.layout negotiation replaces them in sub-step 3);
  overlay repositioned in OnWindowBoundsChanged (154 signature takes
  only new_bounds). Sandbox ON: Windows = bootstrap.exe + xenon-blink.dll
  (USE_SANDBOX default On, CEF_USE_BOOTSTRAP export RunWinMain), Linux =
  SUID chrome-sandbox. Popups open in own Alloy windows.

  **SUB-STEP 2 DEBUGGING SESSION (2026-10-06 evening) — big findings:**
  1. FIXED (root cause of blank window): `CefExecuteProcess` was called
     with a NULL app, so child processes never received
     OnRegisterCustomSchemes; the renderer refused to commit xenon://ui
     (ERR_ABORTED after a 200 from the handler). Fix: create XenonApp
     BEFORE CefExecuteProcess and pass it (both OS entry paths).
  2. FIXED: my path-traversal guard rejected EVERY URL (paths always
     start with "/"); now strips the leading slash instead.
  3. FIXED: resource handler contract — serve from memory with a KNOWN
     response_length (content_.size()), ReadResponse returns
     bytes_read>0 and false at exhaustion (reference:
     tests/cefclient/browser/scheme_test.cc). With length -1 the loader
     never commits. Temporary LOG()/file diagnostics were removed.
  4. VERIFIED via CDP (--remote-debugging-port=9222 + websocket-client,
     dev-only): the UI view commits xenon://ui/index.html, React mounts,
     tablist + chrome render, title "Xenon".
  5. **TAB OVERLAY BLOCKED (sub-step 2's remaining piece).** The owner
     confirmed: UI renders perfectly, example.com does NOT appear. Root
     fact (from the 154 header docs): "The underlying CefBrowser will
     not be created until this view is added to the views hierarchy" —
     and AddOverlayView does NOT trigger that creation. Verified via a
     temporary OnAfterCreated file-note: only the UI browser (ui=1) is
     ever created. FOUR overlay-attach sequences all failed to create
     the tab browser: (a) in OnWindowCreated before Show, (b) after
     Show, (c) inside OnWindowChanged (cefclient's ViewsWindow pattern —
     cefclient does this for its overlay browser), (d) posted as a
     follow-up UI task. cefclient's ViewsOverlayBrowser uses the SAME
     calls (CreateBrowserView + AddOverlayView CUSTOM) — the missing
     ingredient is still unknown; next session should re-read
     tests/cefclient/browser/views_window.cc + views_overlay_browser.cc
     and diff the ENTIRE flow (e.g. their window is created with the
     overlay browser view as the INITIAL child? or check
     ViewsOverlayControls' Initialize which adds NON-browser overlays
     that DO work), or fall back to two-region BoxLayout (needs the
     owner's OK: the UI document renders twice as chrome + sidebar
     regions). REMOVE the two temporary diagnostic notes first
     (xenon_handler.cc OnAfterCreated file note — removed 2026-10-06
     22:43; xenon_app.cc notes — removed). Commit 8c8ef0a + follow-ups:
     the UI-only state is committed and WORKS (owner screenshot: full
     chrome + start page in the real host).
  6. BUILD DISCIPLINE (cost an hour): ALWAYS kill xenon-blink.exe before
     rebuilding (LNK1104 lock failure is silent in filtered output) and
     NEVER filter build output through grep — a CefString assignment
     error hid behind `tail -1`/`grep error` filters and the stale DLL
     kept failing diagnostics. Verify dll mtime after every build. 3 (core child process + Bridge v0
  + conformance tests) — stop-and-report after each; sub-step 4 = spikes
  S2 (popover above web views) and S8 (Chrome extensions in a multi-tab
  Alloy design — decides the tab view style BEFORE building out;
  candidates in Architecture.md 13).
- **Phase 1, M1.1–M1.3: DONE** (repo/tooling; shared UI + Bridge v0 +
  MockHost; core skeleton + keystore + master-password plumbing + Linux
  keyring CI verification). 53 Rust tests, 53 UI tests, all CI green.
- **No browser exists yet.**

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
- 2026-10-05: **OFL font license ratified by owner delegation.** The owner
  was told the PRD-mandated fonts are SIL OFL 1.1 (not in Rules.md 8.1's
  list) and answered "if you think these are really important then work on
  them"; the recommendation was to allow OFL, so it is treated as allowed.
  Rules.md 8.1 itself was NOT edited — say the word and it will be.
- 2026-10-05: **Master-password plumbing (post-M1.3 hardening):** the core
  binary resolves the DB key through `keystore::resolve` (raw hex default,
  Argon2id-wrapped when a master password is set). Dev/test env plumbing:
  `XENON_CORE_SETUP_MASTER_PASSWORD` (migrate to wrapped), `XENON_CORE_MASTER_PASSWORD`
  (unlock), `XENON_CORE_KEYSTORE_ACCOUNT` (tests must set this so the real
  `core/db-key` entry is never touched). UI password prompting comes later.
- 2026-10-05: **Linux keystore runtime-verified in CI — RESOLVED.** The core
  job runs a headless gnome-keyring inside `dbus-run-session` (canonical
  recipe: ONE daemon, `echo "" | gnome-keyring-daemon --unlock` with a real
  newline) and sets `XENON_FORCE_KEYSTORE_TEST=1`; all four CI jobs green.
  Without that variable the keystore test skips on Linux (no daemon
  locally). Debugging notes: empty `echo -n ""` may exit the daemon; two
  daemons (--start AND --unlock) is wrong; CI failures are emitted as
  `::error::` annotations (readable anonymously; job logs and artifacts
  need repository auth).
- 2026-10-05: MockHost is dynamically imported and lands in its own chunk
  (`dist/assets/mock-host-*.js`); release packaging (M1.6) must exclude that
  chunk. Recorded as a known issue until then.

# How to build and test

UI — from `desktop/ui/` (Node 24, npm):
```bash
npm ci && npm run build && npm test && npm run check:no-external
npm run dev        # dev harness at http://localhost:5173 (MockHost)
```
Core — from `desktop/core/` (Rust; first build compiles SQLCipher+OpenSSL):
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace        # 46 tests; OPENSSL_SRC_PERL is in the user env
cargo audit                   # dev-only tool, installed via cargo install
```
The `xenon-core` binary reads JSON-RPC lines from stdin. Dev/test override:
`XENON_CORE_DB_KEY=<64 hex chars>` (never used by the shipped browser) and
`XENON_CORE_DB_PATH=<file>`; without them it uses the OS keystore and
creates `xenon.db` in the current directory.
Repo checks from root: `python desktop/tools/check-secrets.py`,
`check-no-url-logging.py`, `check-file-length.py`, `ruff check desktop/tools`.
Nothing engine-related builds locally; engine builds run in CI (M1.4+).

# Environment

Owner's machine (local dev only; engine builds happen in CI):
- Windows 11 (build 26200). Git Bash for the assistant; owner often uses
  PowerShell: give commands one per line, no `&&`. `python` works, `python3` alias absent.
- git 2.54.0.windows.1; Node v24.18.0 / npm 11.16.0; Python 3.12.10 / pip 25.0.1.
- Rust 1.99.0 stable (pinned in `desktop/core/rust-toolchain.toml`), MSVC links fine.
- VS Build Tools 2026 (18.10.12224.181): MSVC 14.51.36231, Windows SDK.
- **Perl:** Git's bundled perl LACKS Locale::Maketext::Simple, so vendored
  OpenSSL builds fail. Portable Strawberry Perl 5.42.3 extracted to
  `C:\Users\ACER\tools\strawberry-perl\` (user env `OPENSSL_SRC_PERL` set via
  setx, no admin needed). First core build needs it: `cargo build` in a NEW
  terminal, or prefix `OPENSSL_SRC_PERL="C:\Users\ACER\tools\strawberry-perl\perl\bin\perl.exe"`.
- Repo: https://github.com/Sasuke-Uchhiha28/Xenon-Browser (public), branch
  `main`; milestone branches `phase-1/mX.Y-name` merged to main when done.

# Dependencies added

UI (`desktop/ui/package.json`, all pinned exact, lock file committed):
- react 19.3.0, react-dom 19.3.0, zustand 5.0.15 — MIT
- vite 8.3.2, @vitejs/plugin-react 6.1.2, tailwindcss 4.3.3,
  @tailwindcss/vite 4.3.3, typescript 7.0.2 (Apache-2.0), vitest 5.0.3,
  jsdom 30.1.2, @testing-library/react 16.3.3, jest-dom 7.0.1,
  user-event 14.6.7 — MIT except typescript
Core (`desktop/core/xenon-core/Cargo.toml`, Cargo.lock committed):
- serde 1.0.229, serde_json 1.0.151, thiserror 2.0.21, zeroize 1.9.0,
  getrandom 0.4.3, base64 0.23.1 — MIT/Apache-2.0
- rusqlite 0.40.2 (MIT; feature `bundled-sqlcipher-vendored-openssl` —
  SQLCipher + OpenSSL built from source, needs Perl)
- keyring 4.2.0 (MIT/Apache; features `windows-native-keyring-store`,
  `zbus-secret-service-keyring-store` — v4 feature names differ from v3)
- argon2 0.6.0, aes-gcm 0.11.1 (MIT/Apache; AES-256-GCM + Argon2id wrap)
Python dev tools: ruff 0.16.10 (MIT); fonttools 4.66.1 + brotli (MIT);
pyyaml (local only). Fonts: SIL OFL 1.1 (x3) + Apache-2.0 (Material Symbols)
— OFL ratification still pending with owner.

# Known issues and blockers

- **Dev server 504 "Outdated Optimize Dep"**: if a Vite dev server survives
  its parent shell and restarts mid-session (e.g. git touching files), the
  browser's cached module graph can point at dead dep hashes and the page
  mounts nothing. Fix: kill the stray node process, delete
  `desktop/ui/node_modules/.vite`, `npm run dev` again. Dev-only; production
  builds are unaffected. Diagnosed and fixed 2026-10-05.
- MockHost chunk must be excluded from release packaging in M1.6.
- SearXNG instance URL placeholder; final list + settings UI in M2.5.
- Design.md §11 has unconfirmed assumptions (dark-only theme, hidden 4.4
  panels, plain-language labels) — treated as decided; owner may override.

# Next steps

1. Start M1.4 (Blink host skeleton) in a fresh session — it has sub-steps
   that each stop and report: (1) unmodified CEF sample app building in CI
   on Windows + Linux, (2) Xenon window with UI view + one tab, (3) core
   child process + Bridge v0 + conformance tests, (4) spikes S2 (popover)
   and S8 (extensions). Read Architecture.md 2, 4, 5, 7 (Blink column).
2. Owner: ratify OFL font license (still pending).
3. M1.4 will need Bridge conformance tests shared across hosts (build on
   `desktop/ui/src/bridge/types.ts`).

# File map

- `desktop/Rules.md`, `PRD.md`, `Architecture.md`, `Design.md`, `Phases.md` — governing docs (read per milestone)
- `desktop/memory.md` — this file; the only cross-session state
- `desktop/core/Cargo.toml` + `rust-toolchain.toml` — Rust workspace (crates: xenon-core, xenon-core-capi)
- `desktop/core/xenon-core/src/lib.rs` — error types + hex helpers
- `desktop/core/xenon-core/src/rpc/` — JSON-RPC service (`mod.rs`) + line framing with 1 MB limit (`framing.rs`)
- `desktop/core/xenon-core/src/chunked.rs` — >1 MB message chunking helper (base64 parts)
- `desktop/core/xenon-core/src/db/` — SQLCipher open + DbKey (`mod.rs`), migrations (`migrations.rs`)
- `desktop/core/xenon-core/src/settings.rs` — settings + site_prefs (prepared statements only)
- `desktop/core/xenon-core/src/keystore/` — OS keystore (`mod.rs`), Argon2id wrap (`wrap.rs`)
- `desktop/core/xenon-core/src/logging.rs` — PRIV-05 logger (rejects URLs, banned keys, key-shaped values)
- `desktop/core/xenon-core/src/bin/xenon-core.rs` — stdio RPC process; `tests/rpc_stdio.rs` — real-binary integration tests
- `desktop/core/xenon-core-capi/src/lib.rs` — C ABI stub (abi/protocol version)
- `desktop/ui/tokens.json` → `scripts/build-tokens.mjs` → `src/styles/tokens.css` — token pipeline
- `desktop/ui/src/bridge/` — types.ts (Bridge v0 contract), mock-host.ts, index.ts
- `desktop/ui/src/components/`, `src/shell/`, `src/lib/`, `src/state/ui-store.ts` — component kit, shell, helpers, store
- `desktop/ui/public/fonts/` — committed woff2 + licenses (from `desktop/tools/fetch-fonts.py`)
- `desktop/tools/` — check-secrets.py, check-no-url-logging.py, check-file-length.py, fetch-fonts.py, fonts-manifest.json
- `.github/workflows/lint.yml` — lint, ui, core (Linux) and core-windows jobs
- `.githooks/pre-commit`, `.gitignore`, `.gitattributes`, `LICENSE` (MPL-2.0), `README.md`, `pyproject.toml`
- `desktop/resources/branding/logo-original.png` — THE logo, never modify
