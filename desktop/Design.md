# Design.md — Xenon Browser (Dual-Engine) UI/UX Guide

**Audience:** the AI coding assistant and any contributor.
**Status:** v1 draft. Items marked **[CONFIRM]** are defaults chosen by Claude that the owner has not yet approved (see section 11).

---

## 1. Purpose and source-of-truth order

The Stitch export lives in the `Design/` folder next to this file (screen folders such as `Design/xenon_desktop_workspace_startpage/` and `Design/tactile_cyber_onyx/`) and holds the visual design. It is a **mockup, not production code**. This file says what to keep, what to change, and what to add.

When sources disagree, use this order:

1. **This file**, especially section 4 (mandatory corrections).
2. `screen.png` and `code.html` in each screen folder (layout and visuals).
3. `tactile_cyber_onyx/DESIGN.md` (design language, tokens, component rules).
4. Any extra `.md` files the owner adds inside `Design/` (read them all before building).

---

## 2. What is in `Design/`

| Folder | Platform | What it shows | Maps to product feature |
|---|---|---|---|
| `xenon_desktop_workspace_startpage` | Desktop | Start page: logo, search with engine pills, route card, shield card, speed-dial, device strip | New Tab page, privacy stats, speed dial, connection route |
| `xenon_desktop_tab_manager_clusters` | Desktop | Workspaces and tab manager: grid/tree/timeline, workspace chips, RAM card, tab cards, Ghost card, bulk actions | Tab manager, Workspaces, tab hibernation, Incognito |
| `xenon_desktop_shield_telemetry_defenses` | Desktop | Shield dashboard: integrity dial, 4 metric cards, 24h chart, 6 switches, live interception log | Privacy Shield page |
| `xenon_desktop_devtools_security_inspector` | Desktop | Network and security inspector | Lightweight network/privacy inspector |
| `xenon_browser_workspace` | Android | Main browse screen with bottom nav | Android browse screen |
| `xenon_browser_tab_manager` | Android | Tab switcher (Open / Vault / Desktop) | Android tab switcher |
| `xenon_browser_shield_telemetry` | Android | Shield screen with toggles | Android Privacy Shield |
| `xenon_browser_feature_drawer` | Android | "Developer Drawer" with many tool cards | Android menu/sidebar |
| `tactile_cyber_onyx/DESIGN.md` | Both | Tokens and component rules | Design system |

Each screen folder has `code.html` (Tailwind markup) and `screen.png` (the reference image).

---

## 3. Design language: "Tactile Cyber-Onyx"

Dark tactile neomorphism with electric-violet light. Matte onyx surfaces, extruded and recessed controls, violet glow only for active/focus states.

- **Theme:** dark only in v1 **[CONFIRM]**. No light theme and no separate "Minimalist" style yet.
- **Fonts:** Space Grotesk (headlines), Geist (body and controls), JetBrains Mono (URL bar, labels, numbers, shortcuts).
- **Icons:** Material Symbols as used in the mockups, self-hosted and subset to the icons actually used.
- **Radii:** 8px standard (buttons, tabs, inputs, omnibox), 16px cards and panels, full for pills.

### 3.1 Canonical color tokens

Use the Tailwind theme in `code.html` as canonical, because it is what the screenshots show. Convert it to CSS variables.

| Token | Value | Use |
|---|---|---|
| `surface` / `background` | `#131318` | Page background |
| `surface-container-lowest` | `#0e0e13` | Sidebar, header, recessed troughs (omnibox) |
| `surface-container-low` | `#1b1b20` | Tab strip, toolbar |
| `surface-container` | `#1f1f24` | Cards, buttons |
| `surface-container-high` | `#2a292f` | Active tab, hover, raised widgets |
| `surface-container-highest` | `#35343a` | Highest elevation |
| `primary` | `#dfb7ff` | Text/icons that are active |
| `primary-container` | `#9d00ff` | Primary fills, glows (brand violet) |
| `secondary` | `#e8b3ff` | Hover pings |
| `on-surface` | `#e4e1e9` | Main text |
| `on-surface-variant` | `#d1c1d9` | Secondary text |
| `outline` | `#9a8ca2` | Muted labels (about 5.9:1 on surface, passes) |
| `outline-variant` | `#4e4356` | Dividers, disabled |
| `error` | `#ffb4ab` | Blocked/danger |

Prose values in `tactile_cyber_onyx/DESIGN.md` (`#0D0D12`, `#14141B`, `#1B1B26`, `#B829FF`) are close to these. Use them only where this table has no equivalent. Add `success` (green) and `warning` (amber) tokens because the mockups lack them but the product needs them (HTTPS status, mixed content).

### 3.2 Shadow recipes (from the design system)

- **Recessed (omnibox, inputs, switch tracks):** `inset 2px 2px 4px #060608, inset -2px -2px 4px #262536`
- **Level 1 extruded (buttons, inactive tabs):** `3px 3px 6px #07070B, -2px -2px 5px #222230`
- **Level 2 floating (active tab, popovers, shield cards):** `0 8px 24px rgba(157,0,255,0.18), 0 2px 6px rgba(0,0,0,0.7)`
- **Active glow:** `0 0 12px rgba(184,41,255,0.45)`, used only on focus or active execution.
- **Bevel border:** 1px gradient `to bottom right, #2E2D40, #101016`.

---

## 4. Mandatory corrections to the mockups

These override the mockups. Do not copy the mockup HTML into production as is.

### 4.1 Zero external requests (critical)

Every `code.html` loads things from the internet. A privacy browser must not.

- `https://cdn.tailwindcss.com` (Tailwind compiled in the browser at runtime): replace with build-time Tailwind, output as a local CSS file.
- Google Fonts links (Geist, JetBrains Mono, Space Grotesk, Material Symbols): self-host the font files locally. All are open-licensed (OFL / Apache 2.0, verify when bundling).
- `lh3.googleusercontent.com` images (artwork, tab thumbnails, avatar): remove. Use local assets, generated thumbnails, or nothing.
- Add a strict Content Security Policy to the UI: no remote scripts, styles, fonts, or images.
- Verify with a packet capture that opening every UI screen makes zero outbound requests.

### 4.2 Chrome is too tall

The mockup header is three rows: window bar 32px + tab strip 40px + toolbar 48px = **120px**. That is too tall for a browser (Brave is about 68px, Chrome about 72px).

Required desktop chrome: **two rows, 72px max.**

```
┌────────────────────────────────────────────────────────────────────┐
│ [Tab 1 ✕][Tab 2 ✕][+]  (drag area)                    [—] [□] [✕] │ 32px
├────────────────────────────────────────────────────────────────────┤
│ [←][→][↻] [🛡 Omnibox ........................ ★ ] [tools…] [⋮]    │ 40px
└────────────────────────────────────────────────────────────────────┘
```

- Remove the standalone title bar row and the fake macOS traffic-light dots. Windows and Linux use real window controls on the right.
- Remove the avatar (no accounts) and the top text nav (Workspace / Shields / Console / Config). The sidebar and menu cover them.
- The `+` button sits right after the last tab, not at the far right.
- Closing the last tab opens a fresh New Tab. The window never has zero tabs.
- Sidebar: **collapsed to a 56px icon rail by default**, expandable to 256px, and the choice is remembered. The mockup's always-open 256px sidebar is too wide for small laptop screens.

### 4.3 No fake data

The mockups show invented numbers and claims. Every number, status, and badge in production must come from real data, or be removed. Never ship placeholder metrics.

Remove or rewrite these:

| Mockup text | Problem | Action |
|---|---|---|
| "Kernel Mode // Level 4 Guard", "Hardware-isolated Ring-0 Guard", "eBPF sandbox / eBPF Audit" | Not something a browser can truthfully claim | Remove |
| "Integrity 99.8%", "94 BPS", "128-bit Entropy Spoof", "Rules v4.81" | Invented | Remove, or show a real value (for example the real blocklist version) |
| "-34.2% DOM latency", "0.40s faster execution" | Invented | Remove unless measured; if shown, label "estimated" |
| "V8 Core Debugger", "V8 isolates", "V8 Secure Sandbox" | Gecko uses SpiderMonkey, not V8 | Use "JS engine" or the real engine name |
| "JIT Compilation Bypass", "Widevine DRM Hard Sandbox" | Not planned | Remove from v1 |
| "ZKP proof latency", "Subnet Consensus Stream", "Krypton Portal", "Multi-Chain portfolio, Send/Recv" | Crypto/blockchain content, not our product | Remove |
| "Oblivious DoH, Cloudflare ODoH", "Cloudflare 1.1.1.3", "Relay: Zurich #02" | We use Quad9 (primary), Cloudflare and Mullvad fallbacks | Show the resolver actually in use |
| "Dual Tor-WireGuard hybrid tunnel" with Zurich / Reykjavik / Frankfurt nodes | Not our architecture | See 6.1 |
| "Zero-Log", "Zero-Knowledge", "Hardware-enforced" | Marketing claims we cannot verify | Use plain, true statements |

### 4.4 Out of scope for v1 (hide, do not build) **[CONFIRM]**

- Cross-Device Handoff and Swarm Sync (libp2p), Encrypted Node Fleet Mesh, device list.
- Crypto Portfolio widget.
- "Vault Files" encrypted file storage with a size meter.
- Profile avatar and identity hash card ("Xenon Vault Identity").

The project has no accounts and no cloud sync. Local encrypted backup export/import is the replacement and is designed later.

### 4.5 Search engines

The mockup pills ("DuckOnion", "SearXNG", "Brave Guard", "Qwant") do not match the product. Use: **DuckDuckGo (default), SearXNG, Startpage, Mojeek, Google.** In Tor windows, default to the DuckDuckGo onion address. The search box shows the selected engine's icon and name in its placeholder, and updates when the setting changes.

### 4.6 Plain-language labels **[CONFIRM]**

Ordinary users must understand the UI. Keep the visual style, simplify the words.

| Mockup | Use |
|---|---|
| Navigation Bay | (remove label) |
| Browser Viewport | Browse |
| Shield Telemetry | Privacy Shield |
| DevTools Core | Inspector |
| Key Vault / Warden | Passwords |
| System Settings / Config | Settings |
| Execute | Go |
| Encrypted Dial Matrix | Speed Dial |
| Orchestration Snapshot | Session summary |
| Ghost Session Chamber / Ghost+ | Private window (brand name "Ghost" may stay as a label) |
| HyperDrop | Keep as the name for tab hibernation, drop the ™ |
| Zen Reader | Reader mode |
| Purge Session & Flush Cache | Clear browsing data |

Remove the `™` marks. Telemetry-style jargon (cipher names, packet counts) belongs only inside the Inspector.

### 4.7 Performance and low-end hardware

Glows, large blurs (`blur-3xl`), `animate-pulse`, and `animate-ping` are GPU-heavy. Many users have weak laptops and phones.

- Add a **Reduced effects** setting (on automatically when the system prefers reduced motion): no glows, no blur, no looping animations, simpler flat shadows.
- No infinite animations by default. Pulses are allowed only on one live-status dot.
- Respect `prefers-reduced-motion`.
- Static shadows are fine. Never animate `box-shadow` on lists of items.

### 4.8 Accessibility

- `outline-variant` / `#58576A`-style muted text on dark surfaces is about 2.6:1, below 4.5:1. Use it only for disabled or decorative items, never for readable text.
- Minimum readable text size is 12px. The mockups use 10 to 11px in places; raise them.
- Android touch targets: 44 to 48px minimum. Several mockup chips and buttons are about 28 to 36px.
- Every icon-only button has a tooltip (desktop) or content label (Android). Full keyboard navigation on desktop. Visible focus ring: 2px violet with 2px offset.

### 4.9 Privacy rules that apply to the UI

- The live interception log shows **domains only**, lives in memory, and is cleared when the window or session closes. No URL, query, or path is written to disk. Remove "Export Audit Log" unless it exports only that in-memory domain list on demand.
- Private and Tor windows never appear in saved session snapshots, thumbnails, history, or the "recently closed" list.
- The adult-content block page has **no bypass, no override, no password prompt**, only "Go back".

### 4.10 Logo

The "X" mark in the Stitch mockups is a placeholder. Use the owner's logo from `desktop/resources/branding/` everywhere: window and taskbar icon, New Tab page, sidebar, About page, installers. Generate the needed sizes (16 to 512 px, `.ico`, `.png`) from the original with a script. Do not redraw, recolor or restyle it.

---

## 5. Layout specifications

### 5.1 Desktop (Windows and Linux, identical)

- Chrome: two rows as in 4.2, max 72px.
- Left sidebar: 56px rail (default) / 256px expanded. Items: Browse, Workspaces, Privacy Shield, Inspector, Passwords, Settings. Bottom area: current privacy level badge, History, Downloads, Extensions, engine badge.
- Content area fills the rest. Panels (menus, popovers, Settings sheets) render in a separate overlay layer above web content with `z-index` 1000+, dismiss on outside click. A panel must never share layout space with the web view.
- Three-dot menu holds: New Tab, New Private Window, New Tor Window, Zoom, History, Downloads, Bookmarks, Extensions, Find, Screenshot, QR code, Settings, About.
- Shortcuts: Private window is `Ctrl+Shift+N`; Tor window `Alt+Shift+N`; Quick Notes moves to **`Ctrl+Alt+N`**.

### 5.2 Android

- Same design language, different shell. Match the Android mockups: top pill (shield percentage replaced by a real status), bottom nav **Browse / Tabs / + / Shield / Vault(Passwords) / Menu**.
- The Android UI may differ from desktop in layout only, never in tokens or component styling.
- One shared component library and one token file feed both desktop and Android layouts. Mobile layouts collapse the 12-column grid to 4 columns.

---

## 6. Screens

### 6.1 New Tab / Start page

Keep: centered logo, search box (recessed, large), speed-dial tiles, two summary cards.

- **Search box:** engine icon + placeholder from 4.5. Optional command mode (`Ctrl+K`) with only real commands, such as `:tor`, `:private`, `:clear`.
- **"Mesh Routing Circuit" card becomes "Connection" card**, driven by the active privacy level:
  - *Standard:* DNS resolver in use (for example Quad9), ECH status, HTTPS-only status.
  - *Fast Private:* the user's proxy endpoint and connected/disconnected state.
  - *Tor:* the real circuit (guard / middle / exit country) read from the Tor control port, plus a throughput graph. No fake nodes or IPs.
- **"Shield Integrity" card becomes "Today's protection":** trackers blocked, ads blocked, fingerprint attempts randomized, HTTPS upgrades, all from the local database. Replace the invented percentage dial with a simple ring or counter of real totals.
- **Speed Dial:** user-pinned sites with no latency numbers (they are invented). Favicons stored locally.
- Remove the device fleet strip (4.4).
- Optional wallpaper with blur/dim sliders, local images only.

### 6.2 Tab manager and Workspaces

- Keep: workspace chips with counts, Grid / Tree / Timeline switcher, search field, tab cards with thumbnail, size, status badges (audio, pinned, sleeping, blocked count), bulk-action bar.
- **HyperDrop card** = tab hibernation: show real RAM per tab and reclaimed amount. A sleeping tab shows its stored thumbnail; clicking wakes it.
- **Ghost Session card** = open a Private window.
- Remove the Cross-Device Handoff section.
- Thumbnails are generated locally. Tabs in private or Tor windows are never listed or thumbnailed in a normal window's tab manager.

### 6.3 Privacy Shield

- Keep: header, four metric cards, 24-hour bar chart, switch grid, live log.
- **Switches map to real settings:** Fingerprint scrambler (canvas, audio, WebGL), WebRTC IP protection, Cookie partitioning, Encrypted DNS (shows resolver), Per-site script blocking, ECH status, Tracker and ad blocking. Remove the JIT and Widevine switches.
- Metric cards: ads/trackers blocked, estimated data saved (label "estimated"), fingerprint attempts randomized, and a fourth real metric chosen by the developer.
- **Not toggleable:** the adult-content shield. It appears in the list as a locked, always-on row with no switch.
- Live log: domains only, in memory (4.9).

### 6.4 Inspector (replaces "DevTools Core")

A lightweight page for everyday users: request table (method, status, host, protocol, size, time), connection security (TLS version, cipher, ECH yes/no), blocked items, storage/cookies for the site. Full developer tools come from the engine itself (Gecko's own tools, Chromium DevTools via CEF) and are opened from here. Remove the V8 console, eBPF audit, "Heap Dump", and the artwork thumbnails.

### 6.5 Android drawer ("Developer Drawer" becomes Menu)

Keep the card grid for: Secure route toggle (Fast Private / Tor selection, with kill-switch when a proxy is set), Inspector, Reader mode, Passwords, desktop-site toggle, night tint. Remove: identity hash card, crypto portfolio, Vault Files.

---

## 7. New UI the mockups do not include (design these in the same style)

| Screen / component | Notes |
|---|---|
| **First-run engine picker** | Two cards. *Blink: faster, widest site compatibility, strong built-in protection.* *Gecko: maximum privacy, a little slower.* Choice is changeable in Settings (restart required). |
| **Engine badge** | Small "Gecko" or "Blink" label in the chrome and in the sidebar footer. |
| **Per-tab "open in other engine"** | Small toggle in the tab context menu and on the tab. Triggers the switch flow. |
| **Broken-site popup** | "This site may work better in Blink. Restart now?" Remembers the answer per site. |
| **Switch-engine dialog** | Short warning: tabs will reload, some sites may ask you to sign in again, unsaved form text and private tabs are lost. "Don't show again" checkbox. Progress state during migration. |
| **Privacy-level indicator** | Per window: Standard, Fast Private, Private, or Tor (Private and Tor are separate windows). Visible in the omnibox and window frame. |
| **Fast Private setup** | Add a SOCKS5 or HTTP(S) proxy (WireGuard support is planned after v1), with a recommendation to use a VPN app for whole-system protection. |
| **Tor bridges** | Option for bridges if Tor is blocked. |
| **Settings** | General, Appearance, Privacy, Search, Engines, Passwords, Downloads, Extensions, About. |
| **Bookmarks** (bar + manager), **History** (page + dropdown), **Downloads** (list, pause/resume) | |
| **Password manager and autofill prompts** | Matches "Passwords" in the nav. |
| **Site info and permissions panel** | Camera, mic, location, notifications, popups, per-site shield toggle. |
| **Find in page, zoom badge, context menus, tooltips** | |
| **Adult-content block page** | See 4.9. |
| **Extension management** | Separate per engine. A user who installs uBlock for Gecko must install it again for Blink. Show which engine each extension belongs to. |
| **Onboarding / empty states, error pages** (offline, certificate warning, HTTPS-only warning) | |

---

## 8. Motion

| Interaction | Duration | Notes |
|---|---|---|
| Button press | 80 to 100 ms | Switches to the recessed shadow |
| Hover | 120 ms | Background and border shift |
| Tab switch | 0 to 150 ms | Instant content swap |
| Tab close / open | 150 ms | Width collapse or expand |
| Dropdown / popover | 120 ms | Fade and slide of 4px |
| Sidebar expand | 180 ms | Ease-out |
| Switch toggle knob | 150 ms | Gunmetal to violet |

All motion is disabled or minimized in Reduced effects mode (4.7).

---

## 9. Implementation notes

- Build the UI as web technology (HTML, CSS, JS) so one UI serves **both engines** on desktop and the Android layouts. Structure details belong in `Architecture.md` (written later).
- Compile Tailwind at build time. Keep a single `tokens.css` generated from section 3, used by every screen.
- Build one component per mockup element (Omnibox, Tab, SidebarItem, MetricCard, Switch, Dial, Popover, and so on) with states: default, hover, active, focus, disabled, reduced-effects.
- Data comes only from local modules (blocked-request counters, resolver status, Tor control port, tab manager). If a data source does not exist yet, the element stays hidden. It never shows a placeholder number.

---

## 10. Acceptance checklist

- [ ] No outbound network requests from any UI screen (packet capture).
- [ ] Desktop chrome is 72px or less in height; sidebar collapsed by default.
- [ ] No invented metrics or claims remain (4.3 table fully applied).
- [ ] Search engine list, icon, and placeholder match 4.5 and update reactively.
- [ ] All colors, shadows, radii, and fonts come from the tokens in section 3.
- [ ] Reduced effects mode works and activates with the system motion preference.
- [ ] Text contrast of 4.5:1 or better for all readable text; 44px Android touch targets.
- [ ] Panels render in the overlay layer above web content and never overlap or clip it.
- [ ] Private and Tor windows leave no thumbnails, history, snapshots, or log entries.
- [ ] Engine picker, engine badge, per-tab engine toggle, broken-site popup, and switch dialog exist.
- [ ] Adult-content shield shows as locked and always on, with no bypass anywhere.

---

## 11. Assumptions to confirm (owner)

1. **Dark theme only for v1**, and there is no separate minimalist style. The Reduced effects setting covers low-end hardware. Light theme later, if wanted.
2. **Out-of-scope panels hidden** (4.4): device sync, crypto widget, Vault Files, identity card.
3. **Plain-language labels** (4.6), with "HyperDrop" and "Ghost" kept as feature names.
4. **Shortcut change:** Quick Notes becomes `Ctrl+Alt+N`.
5. **Sidebar collapsed by default** on desktop.
6. The Stitch export folders sit inside `desktop/Design/`, next to `Design.md`. The Android screens in it are reused later by the Android documents.
7. Private browsing and Tor open as **separate windows** (like Chrome, Firefox and Brave), not as tabs.
