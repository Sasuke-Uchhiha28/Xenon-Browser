# Phases.md — Xenon Browser, Desktop

**Scope:** Desktop only (Windows x64, Debian/Ubuntu x86_64). Android is planned separately.
**Five phases, 33 milestones.** Each milestone is sized for **one working session**. Do one milestone, run its checks, update `memory.md`, stop, report.
**Size:** S = small, M = medium, L = large.
**Before each session:** read `desktop/memory.md`, `desktop/Rules.md`, and only the current milestone below plus the sections it names in `PRD.md`, `Architecture.md` and `Design.md`.
**Expectation check:** before M1.4 there is no browser yet, so nothing built earlier may be called one. After M1.6 the release zip contains the real Gecko and Blink engines and is hundreds of MB, not a few MB.

```
Phase 1  Foundation: both engines show the same UI and browse        (M1.1 to M1.6)
Phase 2  Engine switching, sessions and browsing essentials           (M2.1 to M2.7)
Phase 3  Privacy core                                                 (M3.1 to M3.7)
Phase 4  Content protection (adult shield)                            (M4.1 to M4.5)
Phase 5  Everyday features, polish and release                        (M5.1 to M5.6)
```

Spikes (S1 to S10) are defined in `Architecture.md` section 13. A failed spike stops work and goes to the owner.

---

## Phase 1: Foundation and walking skeleton

**Goal:** a runnable browser on both engines with the same UI, built by CI on Windows and Linux.
**Phase exit:** both engines browse HTTPS sites with the shared UI; the S8 extension approach is decided; CI produces runnable artifacts; the UI makes zero external requests; the core runs as a child process and stores settings in the encrypted database.

### M1.1 Repo and tooling (S)
- Inside `desktop/`, create `ui/ core/ launcher/ gecko/ blink/ tools/ packaging/ resources/` (the documents and the `Design/` folder are already there). At the repo root create `.github/workflows/`, `.gitignore`, MPL-2.0 `LICENSE`, `README.md`. Do not touch `android/`.
- Create `desktop/memory.md` per Rules.md section 12.
- Toolchain check: confirm Git, Node.js, Rust (stable), Python 3.11+ and C++ build tools are installed (Windows: Visual Studio Build Tools; Linux: build-essential, cmake, ninja). Record the versions in `memory.md`. If something is missing, give the owner exact install steps and wait. Full Firefox or Chromium builds are not expected on the owner's machine.
- Save the owner's logo (attached to the start prompt) as `desktop/resources/branding/logo-original.<ext>` and note it in `memory.md`. Never redraw or alter it.
- Pre-commit checks: secrets scan, "no URL logging" lint rule, file-length check.
- **Accept:** repo structure exists, CI runs a lint-only workflow green, `memory.md` exists and is correct.

### M1.2 Shared UI foundation (M)
- Read `Design.md` sections 3, 4.1, 4.2, 4.7, 4.8, 8, 9.
- Vite + TypeScript + Tailwind (build-time). Generate `tokens.css` from Design.md. Self-host Space Grotesk, Geist, JetBrains Mono and the used Material Symbols subset.
- Component kit: Button, Switch, Card, Tab, Omnibox, SidebarItem, Popover, MetricCard, with all states and Reduced-effects mode.
- Define `bridge/types.ts` (v0: `window`, `tabs`, `nav`, `events`) and a **MockHost** so the UI runs in a normal dev server.
- Shell layout: two-row chrome (72px max), collapsed sidebar rail, New Tab page skeleton (no fake data). Use the owner's logo in place of the mockup's placeholder "X".
- Strict CSP and the no-external-requests check in CI.
- **Accept:** UI renders in the dev harness; Design.md checklist items 4.1, 4.2, 4.7 pass; zero external requests (automated); component tests pass.

### M1.3 Core skeleton (M)
- Rust workspace with `core` and a `core-capi` crate (stub).
- JSON-RPC over stdio server with versioning, size limits and chunking helper.
- SQLCipher database, migration framework, first tables (`settings`, `site_prefs`), `keystore` module (Windows and Linux), optional master-password wrap.
- Logging module that rejects URL-like fields (PRIV-05).
- **Accept:** `cargo test` and `clippy` green; round-trip RPC test; DB file is unreadable without the key (test); key never appears in logs (test).

### M1.4 Blink host skeleton (L)
- Read `Architecture.md` sections 2, 4, 5, 7 (Blink column).
- CEF host (pinned, checksum-verified download script). Views window, Alloy-style UI view, Alloy-style tab views, core child process.
- **Sub-steps (stop and report after each):** (1) build the unmodified CEF sample app in CI on Windows and Linux to prove the toolchain; (2) replace it with the Xenon window (UI view plus one tab view loading a URL); (3) core child process and Bridge v0 with conformance tests; (4) spikes S2 and S8.
- Implement Bridge v0 (`tabs`, `nav`, `window`, `events`) and the conformance tests.
- **Spike S2:** popover above web content on Windows and Linux.
- **Spike S8 (moved here from Phase 5):** can Chrome extensions run in this multi-tab design? Try the candidate approaches in `Architecture.md` section 13 and report to the owner before building further. The result decides the tab view style.
- **Accept:** Windows and Linux builds in CI; shared UI runs inside the host; open, switch, close tabs and navigate to HTTPS sites; popover renders above the page; S2 and S8 results recorded and discussed with the owner.

### M1.5 Gecko host skeleton (L)
- Read `Architecture.md` sections 2, 4, 5, 7 (Gecko column), 11.
- Surfer (or equivalent) pinned to Firefox 153 ESR, patch set structure, branding (Xenon name and the owner's logo, no Firefox marks), baseline privacy prefs and policies (telemetry and services off), core child process.
- Hide native chrome, host the shared UI, implement `XenonBridge` v0, run the same conformance tests.
- **Sub-steps (stop and report after each):** (1) get an unmodified Firefox 153 ESR artifact build running in CI to prove the toolchain (spike S3); (2) branding plus privacy prefs and policies patch; (3) hide native chrome and load the shared UI (S1); (4) XenonBridge v0, core child process and conformance tests.
- **Spikes S1 and S3:** UI hosting, and Windows and Linux builds within CI limits (artifact build for UI-only changes, full build documented).
- **Accept:** Linux and Windows artifacts produced by CI (or the S3 fallback agreed with the owner); shared UI runs; tabs and navigation work; S1 and S3 results recorded.

### M1.6 Launcher, engine picker, first CI artifacts (M)
- Launcher with the switch protocol skeleton (exit code 75, state file, rollback flag).
- First-run engine picker UI (Design.md section 7), engine badge, default-engine setting stored in core.
- `release.yml` produces zip artifacts for both OSes containing both engines.
- **Accept:** a fresh run shows the picker, the chosen engine starts, the other engine also starts when chosen in Settings (after manual restart); Phase 1 exit criteria met; `memory.md` has working build and test commands. Size check in CI: the release zip contains both engine folders and exceeds the minimum size recorded in `memory.md` (expected hundreds of MB). A few-MB executable fails the check.

---

## Phase 2: Engine switching, sessions and browsing essentials

**Goal:** the signature feature works: switch engines and keep tabs and, as far as possible, logins. Everyday basics exist.
**Phase exit:** the switch works both ways with rollback; the 20-site matrix has a measured login-carry-over rate; history, bookmarks, downloads, find, zoom and print work on both engines.

### M2.1 Session model, restore and multi-window (M)
- `session` module, window and tab snapshot format, crash recovery, startup options (new tab, restore, custom page), multiple windows.
- **Accept:** close and reopen restores windows and tabs on both engines; simulated crash offers restore; private windows are excluded (test).

### M2.2 Cookie migration (M)
- Engine-neutral cookie format, runtime-API export and import on both engines, `migration` staging with encryption and TTL.
- **Spike S5 (part 1):** measure on the site matrix. First propose the 20 sites (a mix of email, social, shopping, video, news and bank-style logins) and get the owner's approval of the list.
- **Accept:** cookies move Gecko to Blink and back in tests; expiry, secure, httpOnly, sameSite and partition keys preserved; staging deleted after use (test).

### M2.3 Storage migration (L)
- Type-tagged serializer for localStorage, sessionStorage and IndexedDB, 1 MB chunking, 50 MB per-origin cap, document-start restore, skip-and-report for oversized origins.
- **Spike S5 (part 2):** run the 20-site matrix and record the carry-over rate in `tools/site-matrix.md`.
- **Accept:** unit tests for the serializer (Date, Blob, ArrayBuffer, Map, Set); matrix results recorded; owner reviews the measured rate and the limits wording.

### M2.4 Switch flow end to end (L)
- Warning dialog, snapshot, staging, handoff, lazy tab restore, rollback on failure, old-engine data wipe (PRD FR-B8), progress UI.
- Per-site engine memory, per-tab "Open in other engine", broken-site popup (Gecko).
- **Accept:** switch with 10 mixed tabs in both directions under 15 seconds for 20 tabs; forced failure rolls back with tabs intact; engine memory used on next visit; no private data in staging (test).

### M2.5 Tabs and omnibox essentials (M)
- Pin, mute, duplicate, reopen closed, tab search, drag reorder, `+` placement, last-tab behavior.
- Omnibox suggestions from local data, search engine list with dynamic icon and placeholder, `@keyword` shortcuts.
- **Accept:** FR-A3, FR-A4, FR-A10 pass on both engines; Back/Forward state correct after searches.

### M2.6 History, bookmarks, downloads (M)
- History page and dropdown, delete by item, domain, range. Bookmarks: star, bar, manager, folders, HTML import and export. Downloads manager (pause, resume, cancel, open folder).
- **Accept:** FR-E1, FR-E2, FR-E3 pass on both engines; deleting history removes it from the database (test).

### M2.7 Page tools and system integration (M)
- Find in page, per-domain zoom with badge, print and Save as PDF, view source, full screen, context menus, default-browser registration, external links.
- **Accept:** FR-E4, FR-E9 pass on both engines on Windows and Linux.

---

## Phase 3: Privacy core

**Goal:** the privacy promises are real and tested on both engines.
**Phase exit:** packet captures prove encrypted DNS only and only allowed connections; ad and tracker blocking works on both engines with matching decisions; private, Fast Private and Tor windows behave as specified.

### M3.1 Network privacy baseline (M)
- Encrypted DNS secure mode, resolver order Quad9, Cloudflare, Mullvad (Gecko: single health-checked endpoint, **S4**), ECH, HTTPS-only with error page, verification that all background services are off.
- Packet-capture test tooling in `tools/`.
- **Accept:** DNS capture shows zero plaintext; idle capture shows only allowed connections; S4 result recorded; ECH status exposed to the UI.

### M3.2 Tracker and ad blocking (L)
- Read `Architecture.md` sections 6.3 and 7.
- Filter lists with source and license records, compiled form, per-engine engines (Gecko add-on, Blink C ABI), cosmetic filtering, per-site shield toggle that cannot affect the adult shield, weekly list update (can be turned off).
- **Spike S6:** shared decision test list, agreement and speed.
- **Accept:** decisions agree within 2%; under 5 ms per request; FR-C5 passes; per-site toggle works.

### M3.3 Fingerprinting, cookies, WebRTC, clipboard (M)
- Per-engine fingerprint protection, third-party cookie blocking and partitioning, WebRTC leak protection, permission-based clipboard protection, safe permission defaults.
- **Accept:** FR-C6, FR-C7, FR-C8 pass; fingerprint test pages show protection active on both engines; clipboard read denied without user paste.

### M3.4 Private windows and data clearing (M)
- Private windows on both engines, clear browsing data with time ranges, clear on exit, cookie manager.
- **Accept:** file-system diff test shows no data written by private windows; FR-C9, FR-C14 pass.

### M3.5 Fast Private (M)
- Proxy settings UI, per-engine proxy application, DNS through the proxy, kill switch, status indicator, privacy-level indicator, Connection card on the New Tab page.
- **Accept:** with a test proxy, all traffic and DNS go through it; killing the proxy blocks traffic with no direct leak (capture); FR-C10 passes.

### M3.6 Tor windows, Gecko only (L)
- Tor Expert Bundle download-and-verify script, lifecycle (on demand, stop when last Tor window closes), bridges (obfs4, Snowflake), control-port circuit info, per-window isolation, WebRTC blocked, `.onion` support, Blink explanation screen.
- **Spike S9:** leak and isolation tests.
- **Accept:** no leaks in capture; different circuits per window; `.onion` test site loads; Tor stops after the last window closes; S9 recorded.

### M3.7 Noise generator, stats, Shield and Inspector pages (L)
- Noise scheduler (Low default, can be turned off, exclusions, battery-saver pause), daily stats, real data on the New Tab page and the Privacy Shield page, in-memory domain-only log, Inspector page.
- **Accept:** noise stays under 1% CPU and 100 KB/s; Shield and New Tab pages show only real data (audit against Design.md section 4.3); log never persisted (test).

---

## Phase 4: Content protection

**Goal:** the adult shield is complete, permanent and fast.
**Phase exit:** all layers that meet budgets are active; the shield cannot be disabled by any route; the average latency budget is met.

### M4.1 Blocklist pipeline, layers 1 and 2, block page (L)
- Propose 2 or 3 candidate blocklist sources with their licenses and wait for the owner's approval before downloading any. `tools/` pipeline builds `adult-domains.fst` from the approved, license-checked sources (record in `SOURCES.md`). Layers 1 and 2 in core, navigation and sub-frame hooks on both engines, `xenon://blocked` page (no bypass), blocked pages excluded from history.
- **Accept:** known test domains blocked on both engines under 1 ms lookup; parent-domain matching works; safe-site test set has zero false positives; block page has only "Go back".

### M4.2 Layer 3 and SafeSearch (M)
- Meta-rating inspection on both engines, SafeSearch rewriting for Google, Bing, DuckDuckGo and YouTube.
- **Accept:** test pages with rating tags blocked; SafeSearch parameters present on all four services on both engines.

### M4.3 Immutability hardening (M)
- Gecko built-in non-removable add-on, locked prefs and policies, tamper tests, `tools/audit-shield.py` in CI, coverage tests for normal, private and Tor windows on both engines.
- **Accept:** audit passes; attempts to disable via settings, `about:config`-style routes, flags and environment variables fail (tests); Tor and private windows are filtered.

### M4.4 Layer 4, local text classifier (L)
- **Spike S7 (text):** choose a small local model that meets license and size rules. Run it off the main thread in core. Benchmark harness for 100 page loads.
- **Accept:** average added latency across layers 1 to 4 under 200 ms; false-positive test set reviewed by the owner. If it cannot meet the budget, stop and ask (never ship as optional).

### M4.5 Layer 5, local image classifier (L)
- **Spike S7 (image):** async image classification, blur, minimum image size, never blocks rendering.
- **Accept:** unsafe test images blurred, safe images untouched, page load not delayed; budget and quality reviewed with the owner. Same rule: if it fails, it is not shipped.

---

## Phase 5: Everyday features, polish and release

**Goal:** a complete browser, packaged and published.
**Phase exit:** every P0 item in the PRD passes; installers work; v1.0.0 tagged and the repository public.

### M5.1 Passwords, permissions, site controls (L)
- Password manager, generator, shared autofill script on both engines, master-password option, CSV import and export. Per-site permissions and site-info panel, popup blocker, autoplay control.
- **Accept:** FR-E5, FR-E6, FR-E7 pass on both engines; passwords never appear in logs or plain files (test).

### M5.2 Workspaces, tab manager, hibernation (L)
- Workspaces and the tab manager (grid, tree, timeline) per `Design.md` 6.2, HyperDrop hibernation, Quick Notes (`Ctrl+Alt+N`), full-page screenshot, QR code, wallpaper.
- **Accept:** FR-F1 to FR-F4, FR-F7 pass; at least 60% RAM reduction with 15 idle tabs; private windows excluded from the tab manager.

### M5.3 Reader, PiP, PDF, spellcheck, extensions (L)
- Reader mode (shared), engine built-ins for PiP and PDF, spellcheck.
- Extension install and manage UI for both engines (FR-F10 Gecko, FR-F11 Blink, using the approach decided in S8), one list per engine, separate installs, engine label on each extension.
- **Accept:** FR-E10, FR-E11, FR-F10, FR-F11 pass; a content-blocking extension installs and runs in each engine separately; switching engines shows each engine's own extension list.

### M5.4 Import and backup (M)
- Import wizard for Chrome, Firefox, Edge, Brave (bookmarks and history first, passwords if feasible), encrypted backup export and import.
- **Accept:** FR-F5 and FR-F6 pass; a backup restores onto a clean install; nothing is uploaded anywhere.

### M5.5 Settings, onboarding, accessibility, error pages (M)
- Complete the Settings pages, onboarding, empty states, error and certificate pages, shortcuts, Reduced-effects, accessibility pass. P2 items (link preview opt-in, page archive) only if everything else is done.
- **Accept:** `Design.md` section 10 checklist fully green; keyboard-only walkthrough passes; contrast checks pass.

### M5.6 Packaging, signing decision, release (L)
- Windows per-user NSIS installer, Linux `.deb` (app and installer icons generated from the owner's logo), licenses and SBOM page, update notification, user and developer docs, **Spike S10** (code signing), release workflow, v1.0 acceptance run from `PRD.md` section 11.
- **Accept:** clean install, upgrade and uninstall on Windows 10/11 and Ubuntu 22.04 and Debian 12; checksums published; owner approves; repository made public and `v1.0.0` tagged.

---

## After v1.0: research track (not one of the five phases)

Starts only after v1.0 ships, or earlier if the owner decides. Do not start it during Phases 1 to 5.

### R1 Xenon Route research spike (M)
- Read `PRD.md` section 14 and `Architecture.md` section 14.
- Write a short design comparison of candidates A, B and C and measure them: speed versus Strict and versus Standard on the 20-site matrix, ECH on direct connections, DNS leakage (packet capture).
- **Accept:** a written go or no-go report with measurements. No product code is merged.

### R2 Xenon Route prototype (L), only after the owner says go
- Implement the chosen candidate behind the Off, Balanced and Strict modes, default Off, with the per-site meter.
- **Accept:** FR-R1 to FR-R6 pass; zero ECH and DNS regressions; the owner decides whether it becomes a default.
