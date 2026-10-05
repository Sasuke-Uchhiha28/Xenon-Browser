# Xenon Browser (desktop)

Xenon is a privacy-first desktop browser for Windows and Linux that ships two
real browser engines: **Gecko** (a Firefox 153 ESR fork) and **Blink** (CEF).
You can switch engines per session and keep your tabs and, where possible,
your logins. All privacy features — encrypted DNS, tracker and ad blocking,
the permanent adult-content shield, private and Tor windows — run locally.
There is no telemetry and no account.

**Status:** Phase 1, Milestone M1.1 (repository and tooling). There is no
browser to run yet; the first runnable skeleton arrives in M1.4/M1.5.

## Repository layout

| Folder | Purpose |
|---|---|
| `desktop/ui/` | Shared TypeScript UI, rendered identically by both engines |
| `desktop/core/` | Rust core: database (SQLCipher), settings, filter engine |
| `desktop/launcher/` | Rust launcher: picks the engine, handles switching |
| `desktop/gecko/` | Gecko host: Firefox fork, patches and built-in add-ons |
| `desktop/blink/` | Blink host: CEF-based host application |
| `desktop/tools/` | Python development and test tools (never shipped) |
| `desktop/packaging/` | Installer and release packaging |
| `desktop/resources/` | Branding (the owner's logo), static resources |
| `desktop/*.md` | Product requirements, architecture, design, rules, phases |
| `android/` | Out of scope for the desktop repository; never modified |

## Development checks

The Python tools in `desktop/tools/` are stdlib-only, so the pre-commit hook
works without installing anything:

```bash
git config core.hooksPath .githooks   # once, after cloning
python desktop/tools/check-secrets.py
python desktop/tools/check-no-url-logging.py
python desktop/tools/check-file-length.py
```

Optional linting for the tools themselves:

```bash
python -m pip install -r desktop/tools/requirements-dev.txt
python -m ruff check desktop/tools
```

The same checks run in CI (`.github/workflows/lint.yml`).

## License

[MPL-2.0](LICENSE). The product uses Xenon branding only; Firefox, Mozilla,
Chromium and Google marks never appear in the product (see
`desktop/Rules.md` section 8).
