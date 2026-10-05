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
  fmt clean, audited. Dev-only key override env var documented in memory.
- **Next: M1.4 (Blink host skeleton)** — CEF host, sub-steps with stop-and-report.
- **No browser exists yet.** First runnable skeleton: M1.4 / M1.5.

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
- **OFL font license needs owner ratification** (decision above).
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
