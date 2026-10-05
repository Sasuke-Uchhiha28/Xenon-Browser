# Architecture.md — Xenon Browser, Desktop

**Scope:** Desktop (Windows x64, Linux x86_64). Android gets its own architecture document later.
**Rule:** where this file says **(verify)**, the detail comes from research and must be checked by a spike before it is relied on. Spikes (S1 to S11) are listed in section 13 and scheduled in `Phases.md`.

---

## 1. Overview

```
 user
  |
  v
 xenon (launcher)  -- picks the engine, supervises it, performs the switch
  |                         |
  | engine = gecko          | engine = blink
  v                         v
 GECKO HOST                 BLINK HOST
 Firefox 153 ESR fork       CEF app (C++)
  - shared UI (web)          - shared UI (web)
  - XenonBridge (JS)         - BridgeHandler (C++)
  - built-in add-ons         - request / response handlers
        |   JSON-RPC (stdio)       |   JSON-RPC (stdio) + C ABI hot path
        +------------+-------------+
                     v
               xenon-core (Rust, child process of the running host)
   database | session | migration | filter | lists | proxy | tor | noise | stats
        |                         |              |
   SQLCipher DB          tor (Expert Bundle)   user proxy
```

Key properties:
- **Only one engine runs at a time.** That is why a restart-based switch works and why two engines can share one database file safely.
- **One shared UI** (HTML/CSS/TypeScript) talks to the engine only through the **Bridge API** (section 5).
- **One shared core** holds all engine-neutral logic. It is a child process of whichever host is running, so there is no network port and no other app can talk to it.

## 2. Components and languages

| Component | Folder | Language | Responsibility |
|---|---|---|---|
| UI | `desktop/ui/` | TypeScript, Tailwind (build-time), HTML | All screens, popovers, components, Mock host for development |
| Core | `desktop/core/` | Rust | Database, session, migration, filter lists, adult shield, proxy and Tor management, noise generator, stats |
| Launcher | `desktop/launcher/` | Rust | Start, supervise and switch engines, rollback |
| Gecko host | `desktop/gecko/` | JS/CSS/XHTML patches, small C++/Rust only if unavoidable | Firefox ESR patch set, built-in add-ons, XenonBridge |
| Blink host | `desktop/blink/` | C++ (CEF sample-app based) | Window and tab views, Bridge handler, request handlers, renderer hooks |
| Tools | `desktop/tools/` | Python | Blocklist builder, site-matrix tests, packet-capture helpers, release scripts |
| Packaging | `desktop/packaging/` | NSIS script, `.deb` control files | Installers |

Python is for tooling only. It is never shipped inside the browser.

## 3. Repository layout

```
xenon/                               (repo root)
  .github/workflows/                 CI (ui, core, blink, gecko, release)
  desktop/
    PRD.md  Architecture.md  Rules.md  Phases.md  Design.md   the documents
    memory.md                        maintained by the AI assistant (Rules.md section 12)
    Design/                          Stitch export (screen folders and tactile_cyber_onyx/)
    ui/  core/  launcher/  gecko/  blink/  tools/  packaging/
    resources/                       filter lists, adult-domain FST, models (generated, not committed if large)
  android/                           later, separate documents (will reuse the Android screens in desktop/Design/)
```

## 4. Process model and lifecycle

1. The user starts `xenon` (launcher). It reads the engine choice from the settings file (written by core) and starts the host `xenon-gecko` or `xenon-blink`.
2. The host starts `xenon-core` as a child and talks to it over **JSON-RPC 2.0, one message per line, on stdin/stdout**. Core exits when stdin closes. Messages above 1 MB are chunked.
3. The host opens its window and loads the shared UI. The UI calls the Bridge API; the host forwards storage and logic calls to core.
4. **Switch protocol.** The host writes `state/switch-request.json` (target engine, snapshot id) and exits with code 75. The launcher sees it and starts the other host with `--restore-switch`. If the new host fails to start twice, the launcher restarts the previous engine with `--rollback` and the UI shows an error. The previous engine's data is untouched until the new engine finishes restoring.
5. Tor: core spawns `tor` on demand (first Tor window) and stops it when the last Tor window closes.

## 5. Bridge API (shared UI to engine)

Single source of truth: `desktop/ui/src/bridge/types.ts`. Both hosts implement it. A **MockHost** implements it in memory so UI work needs no engine build. A conformance test suite runs against all three implementations.

| Group | Methods (summary) |
|---|---|
| `window` | `create({kind: normal\|private\|tor})`, `close`, `list`, `setFullscreen`, `onChrome` layout rects |
| `tabs` | `create`, `close`, `activate`, `move`, `pin`, `mute`, `duplicate`, `reopenClosed`, `list`, `discard` (hibernate), `wake`, `thumbnail` |
| `nav` | `navigate`, `back`, `forward`, `reload`, `stop`, `find`, `zoom`, `print`, `viewSource`, `savePdf` |
| `overlay` | `showPopover(id, anchorRect)`, `hidePopover` (see S1 and S2) |
| `engine` | `current`, `requestSwitch(target)`, `openInOtherEngine(tabId)` |
| `data` | forwards to core: `bookmarks`, `history`, `passwords`, `settings`, `sitePrefs`, `notes`, `workspaces`, `stats` |
| `privacy` | `shield.get/set(site)`, `connection.status`, `proxy.set`, `tor.open`, `log.subscribe` (domains only) |
| `downloads` | `list`, `pause`, `resume`, `cancel`, `openFolder` |
| `permissions` | `get(site)`, `set(site, perm, value)`, prompt events |
| `events` | `tab.updated`, `tab.loading`, `nav.blocked`, `download.updated`, `window.layout`, `engine.switchProgress` |

Rules: the Bridge exists **only** inside UI documents. Web content never receives it (Gecko: chrome-privileged documents only. Blink: exposed only to the UI browser, checked by origin `xenon://ui`).

## 6. Core service

### 6.1 Modules and RPC namespaces

| Module | Purpose |
|---|---|
| `db` | SQLCipher database, migrations, all queries (prepared statements only) |
| `settings` | Settings and site preferences |
| `session` | Window and tab snapshots, crash recovery |
| `migration` | Staging of cookies and storage during an engine switch, TTL cleanup |
| `filter` | Adult shield layers 1 to 5, SafeSearch rewriting, block decisions |
| `lists` | Ad and tracker filter lists, update job, list compilation |
| `proxy` | Fast Private proxy config, kill-switch state |
| `tor` | Tor Expert Bundle lifecycle, bridges, circuit info via the Tor control port |
| `noise` | Traffic noise scheduler and HTTP client |
| `stats` | Daily aggregate counters |
| `keystore` | DB key in the OS keystore (Windows Credential Manager/DPAPI, Linux Secret Service), optional master password (Argon2id wrap) |

### 6.2 Database (SQLCipher, one file, `xenon.db`)

Tables: `settings`, `site_prefs` (domain, zoom, engine, permissions JSON, shield overrides), `bookmarks`, `history`, `passwords`, `notes`, `workspaces`, `workspace_tabs`, `session_snapshots`, `migration_staging` (cookies and storage chunks, with `expires_at`), `daily_stats`, `downloads_meta`, `speed_dial`, `search_engines`.
Not stored: private-window data, URLs of blocked adult pages, a per-request log.
The adult-domain list is separate and read-only: `resources/adult-domains.fst` (an FST built at build time by `tools/`, suffix matching, tens of MB, sub-millisecond lookups). The 2,000,000+ domain source lists must pass the license check in Rules.md section 8.

### 6.3 Hot paths

Per-request decisions cannot wait on JSON-RPC.
- **Blink host:** links `xenon-core` as a library through a small C ABI (`xenon_adblock_check`, `xenon_filter_check_nav`).
- **Gecko host:** ad blocking runs in a built-in add-on using a JavaScript filter engine (Ghostery adblocker, MPL-2.0, pinned) (verify). Adult-shield checks happen per navigation and sub-frame only, so one RPC call per navigation is acceptable.

## 7. Engine implementation matrix

| Feature | Gecko | Blink |
|---|---|---|
| Hosting the shared UI | Patch `browser.xhtml`: hide native toolbar and tab strip, load the UI from `chrome://xenon/content/ui/index.html` in a privileged `<browser>`. Bridge via `XenonBridge.sys.mjs` (S1) | CEF Views framework. UI is an Alloy-style BrowserView. Each tab is an Alloy-style BrowserView (S2). The tab view style may change to Chrome style depending on S8 |
| Tabs and hibernation | `gBrowser` APIs, `discardBrowser` | View lifecycle. Close the view, keep a thumbnail and URL |
| Popovers above web content | Native `<panel>` hosting a UI popover document | CEF overlay views hosting a UI popover document (S2) |
| Private window | Native private window | New window with an in-memory request context (empty cache path) |
| Ad and tracker blocking | Built-in add-on, `webRequest` blocking, content script for cosmetic rules | `OnBeforeResourceLoad` calls core through the C ABI. CSS injected on load start |
| Adult shield hooks | Non-removable built-in add-on (`main_frame`, `sub_frame`), locked prefs and policies | `OnBeforeBrowse` and sub-frame `OnBeforeResourceLoad` |
| Meta-tag layer | `onHeadersReceived` plus `filterResponseData` to read the first KB of the document | `CefResponseFilter` |
| Text and image layers | Content script extracts text. Image bytes via `filterResponseData`, results back through the add-on | Renderer hook extracts text. Image bytes via `CefResponseFilter` |
| Encrypted DNS | TRR mode 3 (TRR only). **One** TRR endpoint at a time: core health-checks Quad9, Cloudflare, Mullvad and switches the endpoint, never to plaintext (S4) | Secure DNS mode with the three templates in order, set through request-context preferences (verify) |
| ECH | Firefox ECH prefs, locked | Chromium ECH feature flag, requires secure DNS (verify) |
| HTTPS-only | Firefox HTTPS-only mode, locked, error page replaced by Xenon's | Upgrade in `OnBeforeBrowse`, own warning page |
| Fingerprint protection | Firefox built-in fingerprinting protection prefs (verify exact names for ESR 153) | Injected randomization of canvas, audio, WebGL, font probing at context creation, seeded per session and site |
| Cookies and storage | Total Cookie Protection | Third-party cookies blocked via cookie access filter, Chromium storage partitioning |
| WebRTC IP leak | Firefox prefs (default-address-only, no host candidates) | Chromium WebRTC IP handling policy preference |
| Fast Private proxy | Built-in add-on `proxy.onRequest` with kill-switch | Proxy preference on the window's request context, invalid proxy blocks traffic |
| Tor window | Dedicated user context plus proxy rule to the local Tor SOCKS port, remote DNS, per-window SOCKS auth for stream isolation (S9) | Not available |
| Downloads | `Downloads.sys.mjs` | `CefDownloadHandler` |
| PDF viewer | Built-in pdf.js | CEF PDF viewer |
| Extensions | WebExtensions with install UI | Chrome extensions (Manifest V3) with install UI. Mechanism and tab-view style decided by S8 |
| Autofill | Shared content script from `ui/autofill/` via built-in add-on | Same script injected at context creation, messages via `CefMessageRouter` |
| Permissions | Permission manager hooks | `CefPermissionHandler` |

Engine profile data never holds history or bookmarks: Gecko's own history is disabled (`places.history.enabled=false`) and Xenon's database is the only store.

## 8. Engine switch flow

1. The UI calls `engine.requestSwitch(target)` and shows the warning dialog.
2. **Snapshot:** the host sends all non-private windows and tabs to `session.snapshot` (URL, title, order, pinned, workspace, own history list).
3. **Cookies:** the host reads all cookies through the engine's runtime API (Gecko cookie service, CEF cookie manager) and sends them to `migration.stageCookies` in an engine-neutral shape (domain, name, value, path, expires as Unix time, secure, httpOnly, sameSite, partition key). This avoids reading Chromium's encrypted cookie file.
4. **Storage:** for each origin in open tabs, the host runs a privileged script that serializes localStorage, sessionStorage and IndexedDB (type-tagged serializer for Date, Blob, ArrayBuffer, Map, Set; blobs base64, 1 MB chunks, 50 MB per-origin cap) into `migration.stageStorage`. Origins over the cap are skipped and reported.
5. **Handoff:** the host writes the switch request and exits (section 4). The launcher starts the other host.
6. **Restore:** before any page loads, the new host imports cookies. Tabs are created lazily (active tab first). For each origin, a document-start script restores the staged storage once, then core deletes that origin's staging rows.
7. **Cleanup:** after restore finishes, core deletes any remaining staging data. Staging also expires after 24 hours. [CONFIRM] The old engine's cookies and site storage are wiped at this point.
8. **Not migrated:** service workers, Cache Storage, unsent form text, in-page JavaScript state, private and Tor windows.

Success is measured per site in `tools/site-matrix.md` and reported honestly in the UI copy.

## 9. Network paths

- **DNS:** always encrypted. Gecko uses a single health-checked TRR endpoint, Blink uses the ordered template list. No path ever allows plaintext fallback. A packet-capture test proves it.
- **Standard windows:** direct, plus DoH, ECH and HTTPS-only.
- **Fast Private:** all web traffic and DNS go to the configured proxy. If the proxy is unreachable, requests fail (kill switch).
- **Tor window (Gecko only):** its own user context. Traffic and DNS go to `127.0.0.1:<tor-port>` SOCKS5 with remote DNS. Each Tor window uses its own SOCKS credentials so circuits are isolated per window. WebRTC is blocked. The strictest fingerprinting settings available are applied, with the documented caveat that it is not Tor Browser's identical-fingerprint set.
- **Noise:** core makes decoy HTTPS requests with its own client through the same proxy settings, scheduled with jitter, capped at 1% CPU and 100 KB/s, excluding search engines and the user's frequent domains.
- **Allowed background connections:** only those in `PRD.md` section 9.

## 10. Adult-content shield

Order and placement:
1. **Layer 1 and 2** (core): before the request is sent, on `main_frame` and `sub_frame`. A match cancels the request and shows `xenon://blocked`.
2. **SafeSearch** (core): rewrites the request URL (Google `safe=active`, Bing `adlt=strict`, DuckDuckGo `kp=1`, YouTube `YouTube-Restrict: Strict` header).
3. **Layer 3** (host response hook): reads the head of the HTML document for rating meta tags.
4. **Layer 4** (core, local model): text from title, headings and the first paragraphs, sent after DOM content is loaded. Runs off the main thread, with a hard latency budget.
5. **Layer 5** (core, local model): images above a minimum size, classified asynchronously. Unsafe images are blurred by the host. Rendering is never blocked.

Immutability (enforced by tests, see Rules.md ENG and PRIV rules):
- No setting, preference, command-line flag, or environment variable can disable it.
- Gecko: the add-on is a built-in, non-removable system add-on. Relevant prefs are locked in the distribution config and policies.
- It applies in normal, private and Tor windows on both engines.
- `tools/audit-shield.py` fails CI if any code path contains a bypass condition.

Models and lists are bundled (no remote inference). Specific model choices are made in spike S7 and must be local, small, and license-compatible.

## 11. Build, CI and packaging

**Local development (weak machine):** most work happens in `ui/` and `core/` with the MockHost. Engine builds happen in CI.

| Workflow | What it does |
|---|---|
| `ui.yml` | Lint, test, build the UI, run the no-external-requests check |
| `core.yml` | `cargo test`, `clippy`, `cargo audit`, license check |
| `blink.yml` | Download the pinned CEF standard distribution (checksum verified), build the host on Windows and Linux |
| `gecko.yml` | Surfer-based. Download the pinned Firefox ESR source, import patches, build. Fast "artifact build" for UI-only changes, full build nightly and for releases, with `sccache` and caches |
| `release.yml` | Assemble the installers, checksums, SBOM, attach to a GitHub Release |

Facts to respect (verified during research, re-check when starting):
- Standard GitHub-hosted runners are free for public repositories, but each job is limited to 6 hours. The Windows runner has 4 vCPU, 16 GB RAM and about 14 GB of disk, which may be too small for a full Firefox build (S3). Fallbacks: split the build, use the temporary drive, or build on another machine.
- Gecko base: **Firefox 153 ESR** (released 21 July 2026, security updates for about 15 months). Mozilla is moving to a 2-week release cadence from September 2026, which is why the base is ESR and not the release channel.
- Fork tooling: Surfer (from the Zen Browser project, MPL-2.0, described as prerelease) or the same pattern by hand. Pin its version.
- Blink: **CEF**, pinned, using the Chrome runtime bootstrap with **Alloy-style** views (the Alloy bootstrap itself was removed in M128). In Alloy style, extension APIs beyond the PDF viewer are not supported. A Chrome-style window can host at most one Chrome-style view, which is the main obstacle to a multi-tab design with extensions. Because Blink extensions are a requirement, spike S8 in Phase 1 decides the tab view style before the Blink host is built out.
- Tor: the Tor Project's **Tor Expert Bundle** (tor, pluggable transports, bridge lines, geoip), pinned and signature-verified, for Windows and Linux x86_64.

**Packaging**
- Windows: per-user NSIS installer to `%LOCALAPPDATA%\Programs\Xenon\` (`gecko/`, `blink/`, `core/`, `tor/`, `resources/`). User data in `%APPDATA%\Xenon\` (`xenon.db`, `gecko-profile/`, `blink-profile/`, `state/`).
- Linux: `.deb` installing to `/opt/xenon/`, launcher at `/usr/bin/xenon-browser`, desktop entry and MIME types. User data in `~/.config/xenon/`.
- Update notification only (GitHub Releases API, static request, can be turned off). The user downloads the new installer.

## 12. Security model

- UI documents are privileged. They load only local files, under a strict Content Security Policy (no remote scripts, styles, fonts or images). They never embed web content in the same document.
- Web content gets no Bridge, no `xenon://` access except the block, error and new-tab pages, and no core access.
- Core accepts messages only from its parent (stdio) and validates every parameter (types, sizes, allowed values).
- The database key never touches disk in plaintext, is never logged, and is zeroized in memory when possible.
- Logging: no URLs, queries, titles, credentials or note text in any log (lint-enforced).
- Dependencies pinned and audited. Engines and Tor are verified by checksum or signature before use.
- Sandboxes on for both engines. `--no-sandbox` is forbidden.

## 13. Spikes and decision gates

Each spike has a pass condition and a fallback. The AI assistant must complete the spike, record the result in `memory.md`, and ask the owner before choosing the fallback.

| ID | Question | Pass condition | Fallback | Phase |
|---|---|---|---|---|
| S1 | Can Firefox host the shared UI with hidden native chrome and `gBrowser` tabs? | Tabs, navigation and popovers driven from the shared UI | Zen-style XUL patches for the chrome, shared components for panels | 1 |
| S2 | Can CEF show popovers above web views, and manage multiple Alloy views per window? | Popover renders above tab content on Windows and Linux | Native popup windows for menus | 1 |
| S3 | Can CI build Firefox on both OSes within limits? | Windows and Linux full builds under 6 hours | Split jobs, temp drive, another build machine | 1 |
| S4 | Can Gecko keep encrypted DNS with failover across three resolvers? | Failover works, capture shows no plaintext | Single resolver plus manual switch | 3 |
| S5 | How well do cookie and storage migrations work? | At least the target rate on the 20-site matrix (set in the milestone) | Document limits, narrow the origin set | 2 |
| S6 | Do the two ad-blocking engines agree and perform? | Same decisions on the shared test list (within 2%), under 5 ms per request | Use one engine for both | 3 |
| S7 | Can local text and image classifiers meet the budgets? | Under 200 ms average, no render blocking, low false positives | Ship layers 1 to 3 only (never optional) | 4 |
| S8 | Can Blink support Chrome extensions (Manifest V3) in a multi-tab design with the shared UI? Candidate approaches: Chrome-style views with the CEF toolbar hidden (verify it can be hidden), one Chrome-style view per tab window arranged by the shell, or a small adapter implementing the most-used extension APIs | Install a test extension and a content-blocking extension, run on 2 tabs in one window, popups and options page work, UI stays the shared UI | Owner decides: ship Blink without extensions, or accept a reduced UI for extension tabs | 1 (decided before the Blink host is built out); extension UI built in 5 |
| S9 | Can Gecko isolate Tor windows (proxy, DNS, circuits)? | No leaks in capture, circuits differ per window | Single shared circuit with warning | 3 |
| S10 | Free code signing for Windows? | Signed installer without SmartScreen warning, or documented path | Ship unsigned with a documented warning | 5 |
| S11 | Xenon Route: can routing be decided per connection without breaking ECH or encrypted DNS on direct connections, while being measurably faster than Strict and hiding more than Standard? | Report with measurements on the 20-site matrix, zero ECH or DNS regressions | Drop the idea; keep Fast Private and Tor as they are | post-v1 |

## 14. Post-v1 research notes: Xenon Route

Requirements are in `PRD.md` section 14. This section only lists candidate designs for spike S11. Nothing here is built in v1.0, and no new protocol or cryptography is invented: only standard SOCKS5, HTTP CONNECT or the existing Tor path are used.

**Decision inputs (computed in core, cached locally):**
- Whether the name has an ECH configuration, from the DNS HTTPS record fetched over the encrypted resolver.
- Whether the destination IP is in a published CDN range (shared by many sites) or not.

**Candidate A: per-request proxy decision inside the engine.**
- Gecko: the built-in add-on's `proxy.onRequest` listener is asynchronous, so it can ask core for the decision. Returning "direct" keeps native ECH and DNS.
- Blink: a proxy auto-config script cannot call core. Core would generate and refresh PAC rules for the hostnames it has already classified, or the host would use a precomputed classification list. Needs testing.

**Candidate B: a local loopback proxy inside core that decides per connection.**
- Simple to build, but when an engine is pointed at a proxy it normally lets the proxy resolve names. The engine then skips its own DNS lookup and may not get the ECH configuration, which would remove ECH from the "direct" connections. This would break FR-R4, so B is rejected unless the spike shows the engines can still use ECH.

**Candidate C: static classification lists only.**
- Bundled lists of hostnames or ranges known to support ECH on shared CDNs. Least accurate, smallest, fastest. Good fallback.

**Output of the spike:** a written report (design chosen, measured speed versus Strict and Standard, ECH and DNS regression results, and a go or no-go recommendation). No product code is merged until the owner approves.
