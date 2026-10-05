# Rules.md — Xenon Browser, Desktop

**Audience:** the AI coding assistant (any model) and any contributor. Read this file first in every session.
**Status:** Mandatory. Rules marked **[HARD]** can never be relaxed for convenience, testing or speed.

---

## 0. Documents and precedence

When documents disagree, follow this order and **stop and ask the owner** if the conflict affects behavior:

1. `Rules.md` (this file)
2. `PRD.md` (what to build)
3. `Architecture.md` (how it fits together)
4. `Design.md` and the Stitch screens inside `Design/` (how it looks)
5. `Phases.md` (order of work)

Never silently change the architecture, a requirement or a rule. Propose the change, record it in `memory.md`, and wait for the owner's answer.

## 1. Work protocol

1. **One milestone per session.** Do exactly the milestone you were given in `Phases.md`. Do not start the next one.
2. **Start of a session:** read `memory.md`, this file, the current milestone in `Phases.md`, and only the sections of `PRD.md`, `Architecture.md` and `Design.md` that the milestone names. Do not load whole documents or the whole repository.
3. **Plan before coding:** write a short plan (files to touch, tests to add) in the session, then implement in small steps.
4. **Definition of done:** every acceptance check in the milestone ran and passed, tests were added, `memory.md` was updated, and nothing outside the milestone's scope changed.
5. **Evidence, not claims:** report what you ran and the results (test output, capture results). Never say something works if you did not run it. If you cannot run something (for example an engine build on the owner's machine), say so and give the exact command for the owner or CI.
6. **Stop and report** at the end of the milestone. Wait for the owner.
7. The owner knows only Python and is not a systems programmer. Explain errors and next steps in plain language, with exact commands to copy.
8. **End-of-session report, always in this format:** (a) what changed (files), (b) commands run and the last lines of their output, (c) what is NOT done or NOT verified, (d) sizes of any artifact built, (e) "How you can check it": 2 to 4 exact commands or clicks the owner can do.
9. **No stand-ins presented as the real thing.** Mocks and stubs are allowed only in tests and the MockHost development harness, must be named as such, and must never be shipped or described as a finished feature. If the real implementation is blocked, stop and report instead of substituting a stub.

## 2. Privacy and security rules

| ID | Rule |
|---|---|
| **PRIV-01 [HARD]** | No telemetry, analytics, crash upload or usage pings. The only background connections allowed are in `PRD.md` section 9. Adding any other outbound connection needs the owner's written approval and a PRD change. |
| **PRIV-02 [HARD]** | The adult-content shield has no switch, setting, preference, command-line flag, environment variable, debug mode or password override, on either engine, in any window type. Never add `enableAdultFilter`-style conditions or "skip in tests" branches. Tests use test lists, not bypasses. |
| **PRIV-03 [HARD]** | All content classification is local. Never call a remote API (including any AI service) to inspect pages, images or URLs. |
| **PRIV-04 [HARD]** | Encrypted DNS only. No plaintext DNS fallback, ever, on either engine. |
| **PRIV-05 [HARD]** | No logging of URLs, queries, page titles, search text, passwords, note text or cookie values, to console, files or any store except the encrypted database where the feature requires it. The live interception log is in-memory, domains only. |
| **PRIV-06 [HARD]** | Private and Tor windows write nothing to disk (no history, cache, cookies, thumbnails, session snapshots, stats entries with URLs). |
| **PRIV-07 [HARD]** | Web content never receives the Bridge, core access or any privileged API. |
| **PRIV-08** | UI claims must be true. No invented metrics, no marketing superlatives in the UI ("zero-leak", "military-grade", "hardware-enforced"). Every number shown comes from real data or is hidden. |
| **PRIV-09** | Secrets (keys, tokens, passwords) are never committed, logged or placed in test fixtures. |
| **SEC-01 [HARD]** | Engine sandboxes stay on. `--no-sandbox` and equivalents are forbidden on both engines. |
| **SEC-02** | Validate every message that crosses a boundary (UI to host, host to core, core to Tor): types, sizes, allowed values. |
| **SEC-03** | Verify downloaded engines, Tor and CEF by pinned checksum or signature before use. |

## 3. Engine and architecture rules

1. **ENG-01 One UI.** No engine-specific UI forks. Differences live behind the Bridge API.
2. **ENG-02 Bridge only.** The UI talks to the engine only through the Bridge API (`ui/src/bridge/types.ts`). Every Bridge change updates the types, the MockHost, both hosts and the conformance tests together.
3. **ENG-03 Parity.** A feature is not done until it works on both engines, unless `PRD.md` section 5.2 lists the exception. Write the engine-neutral logic once in core.
4. **ENG-04 Core owns data.** All database access is in `core/` through prepared statements. Hosts and UI never open the database.
5. **ENG-05 No engine source in git.** Firefox and Chromium sources, CEF binaries, Tor binaries and model weights are fetched by pinned scripts, never committed.
6. **ENG-06 Pinned engines.** Firefox is pinned to **153 ESR** (point release pinned in config). CEF and Tor versions are pinned. Upgrades are their own milestone with the full test suite.
7. **ENG-07 Patch discipline (Gecko).** Keep the Firefox patch set small and grouped by feature, one patch per concern, each documented. Prefer prefs, policies, built-in add-ons and `chrome://` files over C++ changes.
8. **ARCH-01 Process model** is as in `Architecture.md` section 4. Do not add network listeners (ports) for IPC.
9. **ARCH-02 Hot paths** (per-request checks) must not use JSON-RPC. Use the in-process library or the add-on engine.
10. **ARCH-03 Modular features.** Each feature lives in its own module with its own tests.
11. **ENG-08 Only the two engines.** The shipped browser uses exactly two engines: the Gecko host (Firefox ESR fork) and the Blink host (CEF). Never use Electron, Tauri, WebView2, the system WebView, QtWebEngine, Servo or any other web view as a shortcut or placeholder, not even temporarily.
12. **ENG-09 Size sanity.** A build that contains both real engines is hundreds of megabytes. CI fails a release artifact that lacks the `gecko/` or `blink/` engine files or is smaller than the minimum size recorded in M1.6. A tiny executable means an engine is missing.

## 4. Code standards

### 4.1 All languages
- Maximum **300 lines per file** (except generated files and tests). Split into modules instead.
- Every exported function, class and component has a doc comment: purpose, parameters, return value, errors.
- Names are descriptive. No dead code, no commented-out code, no TODOs without an issue note in `memory.md`.
- Errors are handled and surfaced. No empty catch blocks, no swallowed errors.
- No new global mutable state without a comment explaining why.

### 4.2 TypeScript (UI)
- Strict mode on. No `any` unless commented. ES modules only.
- Functional components. State with a single small store library chosen in milestone 1.2 and then kept. No class components.
- File names: components `PascalCase.tsx`, modules `kebab-case.ts`, hooks `useThing.ts`, tests `*.test.ts` next to the code.
- No `innerHTML` with dynamic content. No `eval`, no inline event-handler strings.
- CSS from tokens only (`tokens.css`). No hard-coded colors, shadows, fonts or radii.

### 4.3 Rust (core, launcher)
- Stable toolchain pinned in `rust-toolchain.toml`. `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` must pass.
- No `unwrap()` or `expect()` outside tests, except on proven invariants with a comment.
- `unsafe` only in the C ABI layer, each block commented with its safety argument.
- Secrets use zeroizing types where practical.

### 4.4 C++ (Blink host)
- Based on CEF's sample application structure. Keep it thin: windowing, views, handlers, and forwarding. Logic goes to core.
- C++17 or newer as CEF requires. RAII, no raw owning pointers outside CEF's ref-counting types.
- No logging of URLs or data (PRIV-05).

### 4.5 JavaScript/XHTML (Gecko host)
- Keep Xenon code in its own directory and namespace (`chrome://xenon/`). Avoid editing Firefox files except through documented patches.
- Built-in add-ons follow the same file-size and doc rules.

### 4.6 Python (tools only)
- Python 3.11+, type hints, `ruff` clean. Never imported by the shipped browser.

## 5. UI rules

1. **UI-01 [HARD]** Zero external requests from any UI screen. Tailwind is compiled at build time. Fonts, icons and images are local. A strict CSP is set. CI checks it.
2. **UI-02** Follow `Design.md` section 4 (mandatory corrections) over the raw Stitch mockups. Never copy mockup HTML as-is.
3. **UI-03** Tokens in section 3 of Design.md are the only source of colors, shadows, radii, fonts.
4. **UI-04** Support Reduced-effects mode and `prefers-reduced-motion`. No infinite animations by default.
5. **UI-05** Accessibility: contrast 4.5:1 for readable text, minimum 12px text, visible focus rings, keyboard access, tooltips on icon-only buttons.
6. **UI-06** Plain-language labels from Design.md section 4.6.
7. **UI-07** Popovers and menus use the overlay mechanism and never share layout space with the web view.
8. **UI-08** Do not show data that does not exist yet. Hide the element instead of showing placeholders.

## 6. Testing and verification

1. Every module has unit tests. Target at least 80% coverage on `core/` and on `ui/src/bridge`.
2. **Bridge conformance tests** run against MockHost, Gecko host and Blink host.
3. **Filter conformance tests:** the same URL list must give the same block decisions on both engines.
4. **Network tests** (`tools/`): packet capture at idle and during browsing. They assert encrypted DNS only and only the allowed connections.
5. **Shield audit:** `tools/audit-shield.py` scans for bypass conditions and fails CI.
6. **File-system diff test** for private and Tor windows.
7. **Performance benchmarks** for the budgets in `PRD.md` section 10, using the baseline test machine description.
8. **Site matrix** (`tools/site-matrix.md`): 20 real sites used to measure login carry-over and compatibility.
9. A milestone is not done if any previously passing test now fails.

## 7. Build and CI

1. Local machines may be weak. Do not require a local full Firefox or Chromium build. Local work uses `ui/`, `core/` and the MockHost. Engine builds run in GitHub Actions.
2. Workflows are in `.github/workflows/` as described in `Architecture.md` section 11. Keep each workflow readable and commented in plain language.
3. Never put secrets in workflows or the repo. Use GitHub secrets only when the owner adds them.
4. If a CI run fails, read the logs, fix, and explain the cause in plain language. If it needs a decision from the owner (for example a runner limit), stop and ask.

## 8. Dependencies and licensing

1. **Allowed licenses:** MIT, Apache-2.0, BSD-2/3, ISC, MPL-2.0, Zlib, CC0, public domain. LGPL only if dynamically linked and the owner approves. **GPL, AGPL, SSPL, and unclear licenses are not allowed** in shipped code (this excludes bundling GPL-licensed tools such as uBlock Origin).
2. Before adding any dependency: check the license, maintenance status, size, and that it has no telemetry or network calls. Record it in `memory.md`.
3. **Pin exact versions** (lock files committed). No `^` or `~` ranges, no `latest`.
4. Block and filter list sources (ads, trackers, adult domains) must be license-checked and recorded with source URL, license and date in `desktop/resources/SOURCES.md`.
5. Branding: no Firefox, Mozilla, Chrome, Chromium or Google names, logos or icons in the product. Use Xenon branding only. Use only the logo the owner provides, stored in `desktop/resources/branding/`. Never redraw, recolor or invent logos or icons. Create other sizes and formats from the original with a script.
6. Prefer few, well-known dependencies over many small ones.

## 9. Git and releases

- Conventional Commits: `feat(core):`, `fix(ui):`, `refactor(blink):`, `test(filter):`, `docs(arch):`, `chore(ci):`, `perf(db):`.
- Small commits tied to a milestone. Branch per milestone: `phase-1/m1.3-core-skeleton`.
- Never commit: databases, user data, engine sources or binaries, model weights, generated filter lists above 5 MB, `node_modules`, build outputs, secrets.
- The repository will be public. Treat every commit as public.

## 10. Never do this

- Never add any way to disable the adult shield (PRIV-02).
- Never add telemetry, analytics SDKs, remote fonts, CDNs or remote images (PRIV-01, UI-01).
- Never use `--no-sandbox` (SEC-01).
- Never send page content or URLs to any remote service (PRIV-03).
- Never log URLs, queries or credentials (PRIV-05).
- Never expose the Bridge to web content (PRIV-07).
- Never invent metrics, claims or placeholder numbers in the UI (PRIV-08).
- Never commit secrets, engine sources, or model files.
- Never touch the `android/` folder.
- Never claim a task is done without running its checks.

## 11. Handling uncertainty

- If a requirement, document or technical detail marked **(verify)** is unclear or turns out wrong, stop, explain the problem in plain language, offer 2 or 3 options with a recommendation, and wait.
- If a spike fails its pass condition, report it, record it in `memory.md`, and ask before using the fallback.
- If you cannot finish a milestone in one session, finish a clean, working subset, record exactly what remains in `memory.md`, and stop.

## 12. Memory protocol (`desktop/memory.md`)

**The AI assistant creates `desktop/memory.md` itself at the start of the first session** if it does not exist, and keeps it current. It is the only thing that survives between sessions, so write it for a reader with no context.

Required sections, in this order:

1. `# Project state`: current phase and milestone, status (not started / in progress / done), date.
2. `# Decisions`: dated one-line decisions (include spike results and the owner's answers).
3. `# How to build and test`: exact commands for the UI, core, hosts and CI. Keep them correct.
4. `# Environment`: pinned versions (Firefox ESR point release, CEF, Tor, Rust, Node, Python), paths, notes about the owner's machine.
5. `# Dependencies added`: name, version, license, why (one line each).
6. `# Known issues and blockers`: what is broken and what is needed.
7. `# Next steps`: at most 5 items, the first is the immediate next action.
8. `# File map`: one line per important module and where it lives.

Rules:
- Update `memory.md` at the end of every milestone, after every decision, and whenever something blocks you.
- Keep it **under 200 lines**. When it grows, summarize older entries into one line each.
- Never store secrets, personal data, URLs the owner visited, or long logs. Put long logs in `desktop/logs/` (git-ignored) and link by filename.
- If `memory.md` disagrees with the code, trust the code and fix `memory.md`.

## 13. Context hygiene (keeps the assistant effective)

- Read only what the milestone needs. Use headings to jump to sections.
- Do not paste large files, logs or build output into the conversation. Summarize and save logs to files.
- Prefer many small files (under 300 lines) over large ones.
- When a session becomes long or confused, finish the current step, update `memory.md`, and tell the owner to start a fresh session with the standard start prompt.
