# PRD.md — Xenon Browser, Desktop v1.0

**Scope:** Desktop only. Windows 10/11 x64 and Debian/Ubuntu x86_64 (`.deb`). Android has its own PRD, written later.
**License:** MPL-2.0. **Language of the UI:** English (strings externalized for later translation).
**Marker:** items tagged **[CONFIRM]** are defaults chosen by Claude that the owner has not yet approved (list in section 13).
**Priorities:** P0 = must ship in v1.0. P1 = ships in v1.0 if time allows, cut last-first. P2 = stretch, only after everything else.

---

## 1. Summary

Xenon is a privacy-first, open-source, zero-telemetry desktop browser that ships **two engines in one product**:

- **Gecko** (Firefox 153 ESR base): maximum privacy, a little slower.
- **Blink** (Chromium Embedded Framework): faster, widest site compatibility, with Brave-level protections added on top.

The user picks a default engine on first run. Switching engines restarts the browser and carries over tabs and logins as far as technically possible. The UI is identical on both engines (see `Design.md`).

Xenon also provides: encrypted DNS and ECH, tracker and ad blocking, fingerprint protection, private windows, a bring-your-own-proxy mode, Tor windows with `.onion` access (Gecko only), and a permanent local adult-content shield.

## 2. Goals and non-goals

**Goals**
1. Strong privacy by default on both engines, with no setup.
2. Two engines, one UI, one shared database, and a switch that mostly keeps users logged in.
3. An adult-content shield that end users cannot turn off, running 100% on the device.
4. A complete everyday browser (bookmarks, passwords, downloads, find, zoom, and so on).
5. A codebase one person plus an AI assistant can build and maintain.

**Non-goals for v1.0**
- Android (separate PRD), macOS, ARM builds, other UI languages.
- Accounts, cloud sync, device handoff. (Encrypted local backup replaces sync.)
- Writing our own browser engine.
- Xenon Route (exposure-aware routing) is a post-v1 research track, see section 14.
- Parental-control accounts or passwords.

## 3. Users

- Privacy-conscious individuals and families who want ISP-level and tracker-level protection without configuration.
- Students, researchers and journalists who need isolated sessions, notes and offline pages.
- Users on shared computers who want nothing left on disk (private windows, clear-on-exit).

## 4. Product principles

1. **Zero telemetry.** No analytics, crash upload or usage pings. Allowed background connections are listed in section 9 and nothing else.
2. **Secure by default, simple to use.** The only choice a new user must make is the engine.
3. **Honest claims.** The UI and marketing never state a protection the product does not provide. No invented numbers.
4. **Engine parity.** Every feature works on both engines unless section 5.2 says otherwise.
5. **Local data, encrypted.** One database, encrypted at rest, under the user's control.
6. **Speed matters.** Privacy features run within the budgets in section 10.

### 4.1 What Xenon does not claim

- Xenon does not make users anonymous, except inside a **Tor window**, and even there it does not match Tor Browser's identical-fingerprint protection. For high-risk situations, recommend the official Tor Browser.
- DoH and ECH do not hide IP addresses from an ISP. ECH works only on sites that support it.
- The traffic-noise generator is a supplementary measure, not a shield.
- Blink-mode protection is "Brave-level", weaker than Gecko mode. The first-run screen says so plainly.
- "Cannot be disabled" describes the product for end users. It is a product policy, not a technical guarantee against someone modifying the open-source code.

## 5. Engines

### 5.1 First run and switching

- First-run screen with two cards: **Blink** ("Faster, widest site compatibility, strong built-in protection") and **Gecko** ("Maximum privacy, a little slower"). Changeable later in Settings > Engines.
- Switching shows a warning (tabs reload, some sites may ask to sign in again, unsaved form text lost, private windows closed) with "Don't show again".
- Per-site memory: the browser remembers which engine each site works best in (set by the user or by the broken-site popup).
- Per-tab "Open in other engine" action. It switches the whole browser, because only one engine runs at a time.
- Broken-site popup (when the active engine is Gecko): "This site may work better in Blink. Restart now?"

### 5.2 Engine differences (the only allowed ones)

| Capability | Gecko | Blink |
|---|---|---|
| Tor windows and `.onion` | Yes | No (UI explains, offers to switch) |
| Browser extensions | Firefox add-ons (WebExtensions) | Chrome extensions (Manifest V3), separate from Gecko's. Approach decided by spike S8 [CONFIRM] |
| Fingerprint protection | Firefox built-in protection | Randomization injected by Xenon (weaker) |
| Built-in PDF viewer, PiP, reader mode | Engine built-ins plus shared reader mode | Engine built-ins where available plus shared reader mode |

Everything else (DoH, ECH, HTTPS-only, ad blocking, cookie partitioning, private windows, Fast Private, adult shield, noise, passwords, bookmarks, history) must exist on both engines.

### 5.3 Parity matrix for privacy protections

| Protection | Gecko | Blink |
|---|---|---|
| Encrypted DNS (secure mode, no plaintext fallback) | Yes | Yes |
| ECH where the site supports it | Yes | Yes |
| HTTPS-only with warning page | Yes | Yes |
| Tracker and ad blocking | Yes | Yes |
| Third-party cookie blocking and storage partitioning | Yes | Yes |
| WebRTC local-IP leak protection | Yes | Yes |
| Fingerprint protection | Yes | Yes (weaker) |
| Private window (RAM only) | Yes | Yes |
| Fast Private (own proxy) | Yes | Yes |
| Adult-content shield | Yes | Yes |

## 6. Privacy levels

| Level | Scope | Speed | ISP can see | Sites can see | Engines |
|---|---|---|---|---|---|
| **Standard** (default) | All normal windows | Fast | IP addresses, timing, volume; site names where ECH is unsupported | Your real IP | Both |
| **Fast Private** | Normal windows, when a proxy is configured | Fast | Only the proxy address | The proxy's IP | Both |
| **Private window** | Separate window, RAM only | Same as its base level | Same as its base level | Same as its base level | Both |
| **Tor window** | Separate window | Slow | Only that you use Tor (or a bridge) | A Tor exit IP | Gecko only |

Fast Private requires a kill switch: if the proxy fails, traffic is blocked, never sent directly. The UI recommends using a VPN app for whole-system protection.

## 7. Functional requirements

### A. Shell and UI
| ID | Requirement | Pri |
|---|---|---|
| FR-A1 | UI per `Design.md` on both engines: dark theme, tokens, Reduced-effects mode. | P0 |
| FR-A2 | Two-row chrome, 72px or less. Sidebar is a 56px rail by default, expandable. | P0 |
| FR-A3 | Tabs: open, close, reorder (drag), pin, mute, duplicate, reopen closed (`Ctrl+Shift+T`), tab search. `+` after the last tab. Closing the last tab opens a new tab. | P0 |
| FR-A4 | Omnibox: URL or search, selected engine's icon and name in the placeholder (updates reactively). Suggestions from local history and bookmarks only. Remote suggestions are off by default. | P0 |
| FR-A5 | New Tab page: search box, speed dial, real privacy stats, connection card, optional local wallpaper (blur and dim sliders). | P0 |
| FR-A6 | Settings: General, Appearance, Privacy, Search, Engines, Passwords, Downloads, Extensions, About. | P0 |
| FR-A7 | Menus and popovers always render above web content and never overlap or clip it. | P0 |
| FR-A8 | Keyboard shortcuts for all common actions, tooltips, visible focus, accessibility per Design.md section 4.8. | P0 |
| FR-A9 | Multiple windows. | P0 |
| FR-A10 | Search engines: DuckDuckGo (default), SearXNG, Startpage, Mojeek, Google, plus `@keyword` shortcuts. | P0 |

### B. Engines and switching
| ID | Requirement | Pri |
|---|---|---|
| FR-B1 | First-run engine picker, engine badge, default-engine setting. | P0 |
| FR-B2 | Restart-based switch with warning dialog and automatic rollback if the new engine fails to start. | P0 |
| FR-B3 | Session carry-over: windows, tabs, order, active tab, pinned state, URLs, titles, per-tab history list, workspace membership. Private and Tor windows are never carried over. | P0 |
| FR-B4 | Login carry-over: cookies via each engine's runtime API; localStorage, sessionStorage and IndexedDB for the origins of open tabs; chunked transfer; per-origin size cap, default 50 MB [CONFIRM]. Service workers and caches are not migrated. | P0 |
| FR-B5 | Migration staging data is encrypted, excludes private windows, and is deleted once restored (or after 24 hours). | P0 |
| FR-B6 | Per-site engine memory, per-tab "Open in other engine", broken-site popup. | P0 |
| FR-B7 | One shared encrypted database for bookmarks, history, passwords, settings, site preferences, stats, notes, workspaces. | P0 |
| FR-B8 | After a successful switch, the old engine's stored site data (cookies, storage) is wiped once the new engine has finished restoring [CONFIRM]. | P1 |
| FR-B9 | If the browser crashes, the next launch offers to restore the last session. | P0 |

### C. Privacy
| ID | Requirement | Pri |
|---|---|---|
| FR-C1 | Zero telemetry. All Google, Mozilla and Chromium background services disabled (Safe Browsing, sync, translate, component updater, Normandy, Pocket, sponsored content, hyperlink auditing, captive-portal and connectivity checks, and similar). | P0 |
| FR-C2 | Encrypted DNS in secure mode only. Resolvers: Quad9 first, then Cloudflare, then Mullvad. Never fall back to plaintext DNS. | P0 |
| FR-C3 | ECH enabled. Per-site indicator shows whether ECH was used. | P0 |
| FR-C4 | HTTPS-only mode with a warning page when HTTPS fails. | P0 |
| FR-C5 | Tracker and ad blocking (EasyList, EasyPrivacy, Peter Lowe's list), cosmetic filtering, per-site shield toggle. The toggle never affects the adult shield, encrypted DNS or HTTPS-only. | P0 |
| FR-C6 | Third-party cookies blocked by default. First-party storage partitioning. | P0 |
| FR-C7 | Fingerprint protection per engine. WebRTC local-IP leak protection. User agent: blend in, never randomize. Use each engine's reduced or uniform user agent. User-agent, client hints and JavaScript-visible values must stay consistent with each other. A per-site override is allowed only for sites that break, and it must also be consistent. | P0 |
| FR-C8 | Clipboard protection: pages cannot read the OS clipboard except in response to the user's own paste action. The clipboard-read permission is denied by default. [CONFIRM] (a fully virtual internal clipboard is avoided because it would break copy and paste with other apps). | P0 |
| FR-C9 | Private window: RAM only, all protections remain active. | P0 |
| FR-C10 | Fast Private: SOCKS5 or HTTP(S) proxy, DNS through the proxy, kill switch, status indicator. WireGuard support is planned after v1.0 [CONFIRM]. | P0 |
| FR-C11 | Tor window (Gecko only): bundled Tor Expert Bundle (pinned version, signature verified), started on demand and stopped when the last Tor window closes, bridges (obfs4, Snowflake) when Tor is blocked, `.onion` support, real circuit display, per-window stream isolation, WebRTC blocked. | P0 |
| FR-C12 | Traffic noise: on by default, intensity Low, user can turn it off. Decoy HTTPS requests to neutral sites with 10 to 90 second jitter, under 1% CPU and 100 KB/s. Never hits search engines or the user's frequent sites. Pauses when battery saver is on. Always labeled "supplementary". | P0 |
| FR-C13 | Local privacy stats (counts only, daily aggregates). The live interception log shows domains only, stays in memory and is cleared when the window closes. | P0 |
| FR-C14 | Clear browsing data (time ranges), clear on exit option, cookie manager. | P0 |
| FR-C15 | Local database encrypted at rest (SQLCipher). Key kept in the OS keystore. Optional master password. | P0 |
| FR-C16 | Network and security Inspector page (requests, TLS version and cipher, ECH status, blocked items). | P1 |

### D. Content protection (adult shield)
| ID | Requirement | Pri |
|---|---|---|
| FR-D1 | Layer 1: local blocklist of 2,000,000+ domains, with parent-domain matching, under 1 ms per lookup. | P0 |
| FR-D2 | Layer 2: URL path and query pattern matching. | P0 |
| FR-D3 | Layer 3: HTML meta rating inspection (for example RTA labels). | P0 |
| FR-D4 | SafeSearch enforcement for Google, Bing, DuckDuckGo and YouTube Restricted Mode. | P0 |
| FR-D5 | Layer 4: local text classifier on titles, headings and leading text. | P1 |
| FR-D6 | Layer 5: local image classifier, blurs unsafe images, never blocks rendering. | P1 |
| FR-D7 | Block page with "Go back" only. No bypass, override, password prompt, setting, flag or environment variable. Applies in every window type and on both engines, including Tor windows. | P0 |
| FR-D8 | All classification is on-device. Models and lists ship with the app. Blocked pages are never written to history. | P0 |
| FR-D9 | Performance gate: layers 1 to 4 add under 200 ms on average across 100 page loads. If layers 4 or 5 cannot meet their budgets they are not shipped in v1.0 (they are never shipped as optional). | P0 |

### E. Browsing essentials
| ID | Requirement | Pri |
|---|---|---|
| FR-E1 | History: page, dropdown, delete by item, by domain and by range. | P0 |
| FR-E2 | Bookmarks: star button, bookmarks bar, manager with folders, import and export (HTML). | P0 |
| FR-E3 | Downloads manager: list, pause, resume, cancel, open folder. Safe filenames. | P0 |
| FR-E4 | Find in page, per-domain zoom with badge, print and Save as PDF, view source, full screen, context menus. | P0 |
| FR-E5 | Password manager: save, generate, autofill, edit, delete, CSV import and export. | P0 |
| FR-E6 | Per-site permissions (camera, microphone, location, notifications, popups, clipboard) and a site-info panel. | P0 |
| FR-E7 | Popup blocker and autoplay control. | P0 |
| FR-E8 | Startup options: new tab, restore last session, custom page. | P0 |
| FR-E9 | Default-browser registration and opening links from other apps. | P0 |
| FR-E10 | Spellcheck. | P1 |
| FR-E11 | Reader mode (shared), Picture-in-Picture and PDF viewer (engine built-ins where available). | P1 |

### F. Productivity and extras
| ID | Requirement | Pri |
|---|---|---|
| FR-F1 | Workspaces and tab manager (grid, tree, timeline) with tab hibernation ("HyperDrop"): idle tabs (default 30 min, configurable) are unloaded and keep a thumbnail. | P1 |
| FR-F2 | Full-page screenshot (PNG or PDF, copy to clipboard). | P1 |
| FR-F3 | QR code for the current page, generated offline. | P1 |
| FR-F4 | Quick Notes sidebar, bound to URL or domain, Markdown, shortcut `Ctrl+Alt+N`. | P1 |
| FR-F5 | Import wizard for Chrome, Firefox, Edge and Brave: bookmarks and history (P1), saved passwords where feasible (P2). | P1 |
| FR-F6 | Encrypted backup export and import of the whole database. | P1 |
| FR-F7 | Local wallpaper and appearance customization. | P1 |
| FR-F8 | Link preview on hover. Opt-in only, because it makes network requests. [CONFIRM] | P2 |
| FR-F9 | Page archive (single-file HTML, an offline saved copy of a page). | P2 |
| FR-F10 | Extensions on Gecko: Firefox add-ons (WebExtensions) from addons.mozilla.org, with an install and manage UI. | P1 |
| FR-F11 | Extensions on Blink: Chrome extensions (Manifest V3) with their own install and manage UI. Extensions are separate per engine: the same extension must be installed once in Gecko and once in Blink, and the UI shows which engine each belongs to. Install from a local file, or from the Chrome Web Store if its terms allow use outside Chrome (to be checked in S8). Spike S8 runs in Phase 1 because it decides how the Blink host is built. If S8 shows this cannot be done well in a multi-tab design, work stops and the owner chooses the fallback. | P0, gated by S8 |

### G. Security and maintenance
| ID | Requirement | Pri |
|---|---|---|
| FR-G1 | Engine sandboxes enabled. `--no-sandbox` is forbidden on both engines. | P0 |
| FR-G2 | Security-update target: critical upstream fixes shipped within 14 days on both engines. | P0 |
| FR-G3 | Update notification via GitHub Releases (user-visible, can be turned off). In-app auto-apply is out of scope for v1.0. | P0 |
| FR-G4 | No remote crash reporting. A "Copy crash log" button is user-initiated and the log contains no URLs. | P0 |

### H. Distribution
| ID | Requirement | Pri |
|---|---|---|
| FR-H1 | Windows: per-user NSIS installer (no admin needed), x64, both engines bundled. | P0 |
| FR-H2 | Linux: `.deb` only, x86_64, Debian 12+ and Ubuntu 22.04+ [CONFIRM], both engines bundled. | P0 |
| FR-H3 | Public GitHub Releases with checksums, a licenses and SBOM page inside the app. | P0 |

## 8. Content-protection pipeline (summary)

```
Navigation / sub-frame request
  -> Layer 1: domain blocklist       (match -> block page)
  -> Layer 2: URL pattern matcher    (match -> block page)
  -> SafeSearch rewrite of the request
  -> Layer 3: meta rating tags       (match -> block page)
  -> Layer 4: local text classifier  (match -> block page)
  -> Layer 5: local image classifier (unsafe -> blur element)
```

Details and the per-engine hooks are in `Architecture.md`.

## 9. Allowed background connections

Nothing else may leave the machine. A packet-capture test enforces this.

| Connection | When | User control |
|---|---|---|
| Encrypted DNS to Quad9, Cloudflare, Mullvad | While browsing | None (always encrypted) |
| Navigation and sub-resources the user requested | While browsing | n/a |
| Tor network or bridges | Only while a Tor window is open | Open or close the window |
| User-configured proxy | Only when Fast Private is on | Settings |
| Filter-list updates (static file downloads, no identifiers, no cookies) | At most weekly | Can be turned off |
| Update check against GitHub Releases | At most daily | Can be turned off |
| Decoy requests from the noise generator | While the browser is open | Can be turned off |
| Extension installs and updates (Firefox add-on site for Gecko, Chrome Web Store or a local file for Blink) | Only when the user installs or clicks "check" | Manual by default [CONFIRM] |

## 10. Non-functional requirements

| Area | Target | How it is verified |
|---|---|---|
| Filter overhead | Under 200 ms average added page-load latency (layers 1 to 4) | Benchmark of 100 page loads versus the unfiltered engine |
| Engine switch | Under 15 seconds for 20 tabs | Timed test |
| Hibernation | At least 60% RAM reduction with 15 idle tabs | Process memory measurement |
| Traffic noise | Under 1% CPU, under 100 KB/s | Performance counters |
| Database lookups | Under 10 ms for indexed queries | Unit benchmark |
| UI | No outbound requests from any UI screen. No UI layout shift on resize | Packet capture, manual test |
| Baseline test machine | 4 cores, 8 GB RAM, SSD, Windows 10 or Ubuntu 22.04 | Used for all budgets |
| Security | Sandbox on, strict CSP, pinned dependencies, no secrets in repo | CI checks |

## 11. v1.0 release acceptance

1. Both engines launch from the same installer on Windows 10/11 and Ubuntu 22.04, show the same UI and browse HTTPS sites.
2. Switching engines with 10 mixed tabs restores all tabs, and the tested sites (a list of 20 in `tools/site-matrix.md`) keep their login at the documented rate.
3. Packet capture at idle shows only the connections in section 9.
4. DNS capture shows zero plaintext DNS. Browserleaks-style tests show fingerprint protection active on both engines.
5. The adult shield blocks a test set, cannot be disabled by any UI, flag, pref, or environment variable (code audit script passes), and meets its latency budget.
6. Private and Tor windows leave nothing on disk (file-system diff test).
7. Every P0 requirement above passes its test. Unfinished P1 and P2 items are listed in the release notes.
8. Installers install, upgrade and uninstall cleanly. Checksums are published.

## 12. Risks

| Risk | Mitigation |
|---|---|
| Two engines double the security-patch workload | Firefox ESR base, pinned CEF, nightly CI that tries upstream updates, the 14-day target, a banner if one engine lags |
| The Firefox build is too heavy for free CI runners (6-hour job limit, small disk) | Fast artifact builds for UI changes, caching, split jobs, fall back to another build machine (spike S3) |
| CEF limits: overlays above web views, and extensions only work in Chrome style, which allows one Chrome-style view per window (conflicts with a custom multi-tab UI) | Spikes S2 and S8 in Phase 1, decided before the Blink host is built out, owner picks the fallback if S8 fails |
| Login migration is incomplete for some sites | Clear UX warning, a measured success rate, documented limits |
| Scope too large for one developer | Five phases, P1 and P2 cut first, one milestone per session |
| Adult-filter false positives (for example medical sites) | Test set of safe sites, parent-domain and allow-pattern review, bug reports are local-only |
| Installers are unsigned, so SmartScreen warns | Investigate free code signing for open-source projects (spike S10), document the warning |
| Licensing and trademarks | No "Firefox" or "Chrome" branding or logos. Dependency license check (Rules.md section 8). Avoid GPL components |
| AI-assistant quality drops as context grows | Small milestones, `memory.md`, short docs, read only the needed sections |

## 13. Assumptions to confirm (owner)

1. Blink gets its own separate extension support (Chrome extensions, Manifest V3), as you requested. This is the highest-risk item: CEF supports extensions only in Chrome style, which conflicts with a custom multi-tab UI. Spike S8 (Phase 1) decides how. As far as I know, older Manifest V2 extensions (including full uBlock Origin) no longer run on current Chromium, so Blink users get Manifest V3 extensions (for example the lighter uBlock Origin Lite) alongside Xenon's built-in blocker. Gecko has full Firefox add-on support.
2. Fast Private supports SOCKS5 and HTTP(S) proxies in v1.0. WireGuard comes later.
3. Clipboard protection is permission-based, not a virtual clipboard.
4. Link preview is opt-in and P2. Page archive (single-file HTML) is P2.
5. After a switch, the old engine's site data is wiped once the new engine finishes restoring.
6. Per-origin migration size cap is 50 MB.
7. Linux support is Debian 12+ and Ubuntu 22.04+, x86_64 only.
8. Add-on updates are manual by default.
9. English-only UI in v1.0.
10. Tagline: suggested "Private by default." Avoid claims such as "browse without a trace", which overpromise. Final choice is the owner's.
11. Xenon Route (section 14) is a post-v1 research track. v1.0 ships Fast Private and Tor as described, with no exposure-aware routing.

## 14. Post-v1 research track: Xenon Route

**Status:** research only. Not part of the v1.0 acceptance criteria. It starts after v1.0 ships, or earlier only if the owner decides. It must never delay v1.0.

**Problem.** Sending everything through a proxy, VPN or Tor is private but slower. Going direct is fast, but the ISP sees IP addresses and, without ECH, the site name.

**Idea: exposure-aware routing.** For each destination, estimate what the ISP could learn by connecting directly, and use the relay only when that is too much.

| Destination | What the ISP learns if direct | Route |
|---|---|---|
| ECH available and the IP is shared by many sites (a large CDN) | Only that you reached that CDN | Direct (fast) |
| ECH available but the IP belongs to one site | The site, from the IP | Via relay |
| No ECH (site name visible in the TLS handshake) | The site name | Via relay |

**Requirements (all P2, post-v1)**

| ID | Requirement |
|---|---|
| FR-R1 | Modes: Off, Balanced (exposure-aware), Strict (everything through the relay, same as Fast Private). Default Off until the research shows it is worth shipping. |
| FR-R2 | Needs a relay the user configures (the Fast Private proxy) or, in a Gecko window, Tor. Without a relay the feature is unavailable. |
| FR-R3 | An honest per-site meter shows what the ISP could see for the current site (site name, IP only, or neither), computed from real data. No scores or percentages. |
| FR-R4 | Direct connections must keep full ECH and encrypted DNS. If routing would disable ECH on direct connections, that design is rejected. |
| FR-R5 | Classification data (ECH availability, published CDN address ranges) is cached locally. Range lists ship with releases, and optional updates use the existing filter-list update setting. |
| FR-R6 | Never described as anonymity. The UI states the limit: the routing pattern itself leaks a little, and the relay operator sees the destinations it carries. |

**Known limits.** Published CDN ranges are incomplete, so some classifications will be wrong. The relay (or Tor) still sees the traffic it carries. A user agent or fingerprint protection is separate from this feature and stays in FR-C7.

**Research done when:** speed is measured against Strict and against Standard on the 20-site matrix; there is zero ECH regression and zero DNS leakage on direct connections; and the owner receives a written go or no-go report (spike S11 in `Architecture.md`).
