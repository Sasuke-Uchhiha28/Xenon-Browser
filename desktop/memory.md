# Project state

- **Phase 1, Milestone M1.1 (repo and tooling): DONE and accepted 2026-10-05.**
  Folder structure, root files, pre-commit checks, lint CI workflow, logo
  saved, toolchain complete, committed on `main` and pushed to the public
  repo https://github.com/Sasuke-Uchhiha28/Xenon-Browser — the `lint`
  workflow ran green on GitHub (verified via API).
- **Next: M1.2 (shared UI foundation)** — start in a fresh session with the
  standard start prompt.
- **No browser exists yet.** First runnable skeleton: M1.4 (Blink) / M1.5 (Gecko).

# Decisions

- 2026-10-05: Repo root is the `Xenon V3` folder (parent of `desktop/` and the
  empty `android/`). All desktop code lives under `desktop/` per Phases.md M1.1.
  `android/` is empty and untouched; git cannot track empty folders, so it will
  simply not appear in the repository.
- 2026-10-05: Owner's logo saved byte-identical (md5 23d63d19d5f6eac46338acbac4d19002)
  as `desktop/resources/branding/logo-original.png` — 1024x1024 PNG, RGBA.
  Other sizes will be generated from it by script (M5.6), never redrawn.
- 2026-10-05: Pre-commit checks are stdlib-only Python in `desktop/tools/` so
  the hook works on a fresh clone with zero installs. ruff is CI-only plus
  optional locally (requirements-dev.txt).
- 2026-10-05: Lint workflow runs on ubuntu-latest only: the checks are plain
  Python and platform independent; a second OS doubles CI minutes for nothing.
- 2026-10-05: GitHub Actions pinned by commit SHA with the release tag in a
  comment (Rules.md 8.3): checkout v4 = 11d5960a326750d5838078e36cf38b85af677262,
  setup-python v5 = a26af69be951a213d495a4c3e4e4022e16d87065.
- 2026-10-05: `.gitattributes` forces LF for text files (bash pre-commit hook
  and Linux CI break with CRLF); .bat/.ps1 keep CRLF.
- 2026-10-05: NO commits made yet: git had no author identity and the repo will
  be public, so no identity was invented. Owner set their own identity,
  committed on `main` (first-commit exception; milestone branches start at
  M1.2) and pushed to https://github.com/Sasuke-Uchhiha28/Xenon-Browser.
  The three check scripts scan `git ls-files`
  (tracked) when `.git` exists, and walk the tree otherwise.

# How to build and test

Nothing compiles yet. From the repo root (`Xenon V3/`):

```bash
# repository checks (same as pre-commit hook and CI)
python desktop/tools/check-secrets.py
python desktop/tools/check-no-url-logging.py
python desktop/tools/check-file-length.py

# optional Python style check
python -m pip install -r desktop/tools/requirements-dev.txt
python -m ruff check desktop/tools

# pre-commit hook (already configured locally via core.hooksPath=.githooks)
bash .githooks/pre-commit
```

First real builds arrive with M1.2 (npm/Vite in `desktop/ui/`) and M1.3
(cargo in `desktop/core/`). Engine builds run in GitHub Actions only
(Rules.md 7.1), never on this machine.

# Environment

Owner's machine (local dev only; engine builds happen in CI):
- Windows 11 (build 26200), shell: Git Bash. `python` works, `python3` alias
  does not exist (use `python` in commands and docs). Owner often works in
  PowerShell: give commands one per line, no `&&` (PowerShell 5 rejects it).
- git 2.54.0.windows.1 — installed, no user.name/user.email set yet.
- Node v24.18.0, npm 11.16.0 — installed.
- Rust 1.99.0 stable-x86_64-pc-windows-msvc — WORKS (link verified 2026-10-05
  with a throwaway crate).
- Visual Studio Build Tools 2026 (v18.10.12224.181) with "Desktop development
  with C++": MSVC 14.51.36231, Windows SDK, cmake.exe and ninja.exe (bundled
  under `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\`,
  not on PATH). Installed 2026-10-05; unblocks M1.3/M1.4.
- Python 3.12.10, pip 25.0.1 — installed (meets the 3.11+ floor).
- ruff 0.16.10 installed (pip). No `gh` CLI installed.

# Dependencies added

- ruff 0.16.10 — MIT — Python linter for `desktop/tools/`, pinned in
  `desktop/tools/requirements-dev.txt`, runs in CI and optionally locally.
- pyyaml — installed locally on the owner's machine only to validate
  `lint.yml` syntax; NOT a repo dependency (not in requirements-dev.txt).

# Known issues and blockers

None. (C++ tools resolved 2026-10-05; M1.1 committed and pushed; CI green.)

# Next steps

1. Start M1.2 (shared UI foundation) in a fresh session: read Design.md
   sections 3, 4.1, 4.2, 4.7, 4.8, 8, 9; Vite + TypeScript + Tailwind;
   tokens.css; component kit; bridge/types.ts v0 + MockHost; shell layout.
2. Branch name for M1.2: `phase-1/m1.2-shared-ui-foundation` (branch off main).
3. During M1.2: propose the small state store library (Rules.md 4.2) with
   license checks before adding it.

# File map

- `desktop/Rules.md`, `PRD.md`, `Architecture.md`, `Design.md`, `Phases.md` — governing docs (read per milestone only)
- `desktop/memory.md` — this file; the only cross-session state
- `desktop/tools/check-secrets.py` — secrets scan (PRIV-09), pre-commit + CI
- `desktop/tools/check-no-url-logging.py` — no-URL-logging lint (PRIV-05), pre-commit + CI
- `desktop/tools/check-file-length.py` — 300-line limit (Rules.md 4.1), pre-commit + CI
- `desktop/tools/requirements-dev.txt` — pinned dev tool versions (ruff)
- `.github/workflows/lint.yml` — lint-only CI workflow (M1.1 acceptance)
- `.githooks/pre-commit` — runs the three checks locally
- `.gitignore`, `.gitattributes`, `LICENSE` (MPL-2.0), `README.md`, `pyproject.toml` — repo root files
- `desktop/resources/branding/logo-original.png` — THE logo, never modify
- `desktop/ui/ core/ launcher/ gecko/ blink/ packaging/` — empty (.gitkeep) until M1.2+
- `desktop/Design/` — Stitch screens (8 screens + DESIGN.md), input for Design.md
