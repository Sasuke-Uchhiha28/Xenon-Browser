# Project state

- **Phase 1, Milestone M1.1 (repo and tooling): in progress.** All local work done
  2026-10-05: folder structure, root files (LICENSE, README, .gitignore,
  .gitattributes), pre-commit checks, lint CI workflow, logo saved, toolchain
  checked. Remaining: owner sets git identity (commits), owner creates the
  GitHub repo and pushes (so the lint workflow actually runs green).
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
  be public, so no identity was invented. Owner sets identity, then commits
  (commands in Known issues). The three check scripts scan `git ls-files`
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
  does not exist (use `python` in commands and docs).
- git 2.54.0.windows.1 — installed, no user.name/user.email set yet.
- Node v24.18.0, npm 11.16.0 — installed.
- Rust 1.99.0 stable-x86_64-pc-windows-msvc — installed but CANNOT LINK
  (no link.exe; verified with a throwaway crate). Needs VS Build Tools.
- Python 3.12.10, pip 25.0.1 — installed (meets the 3.11+ floor).
- Visual Studio 2022 folder exists but is EMPTY: no MSVC compiler, no Windows
  SDK, no cmake, no ninja.
- ruff 0.16.10 installed (pip).
- No `gh` CLI installed.

# Dependencies added

- ruff 0.16.10 — MIT — Python linter for `desktop/tools/`, pinned in
  `desktop/tools/requirements-dev.txt`, runs in CI and optionally locally.
- pyyaml — installed locally on the owner's machine only to validate
  `lint.yml` syntax; NOT a repo dependency (not in requirements-dev.txt).

# Known issues and blockers

1. **C++ build tools missing (blocks M1.3 and M1.4 local work).** Cargo fails
   with "`link.exe` returned an unexpected error". Owner fix: install "Build
   Tools for Visual Studio 2022" from visualstudio.microsoft.com with the
   "Desktop development with C++" workload (includes MSVC compiler, Windows
   SDK, cmake and ninja; roughly 7 GB). Then record the versions here.
2. **No commits yet** — git identity missing. Owner runs:
   `git config --global user.name "Your Name"` and
   `git config --global user.email "you@example.com"` (their real data; the
   repo will be public). Then `git commit -m "chore(repo): M1.1 repo and tooling"` —
   files are already staged. If they prefer their name on the history as a
   single clean commit: `git reset --soft $(git rev-list --max-parents=0 HEAD)` after
   the first commit is unnecessary — simply commit; author identity is taken
   at commit time.
3. **CI has not run yet** — there is no GitHub remote. Owner creates an empty
   public repo on github.com, then:
   `git remote add origin <repo-url>`, `git push -u origin main`.
   The lint workflow should run green; that completes M1.1 acceptance.

# Next steps

1. Owner: set git identity (commands above), commit; create GitHub repo and
   push; confirm the lint workflow is green. Also install VS Build Tools.
2. Next milestone M1.2 (shared UI foundation): read Design.md sections
   3, 4.1, 4.2, 4.7, 4.8, 8, 9; Vite + TypeScript + Tailwind; tokens.css;
   component kit; bridge/types.ts v0 + MockHost; shell layout.
3. Record installed VS Build Tools / MSVC version in this file once installed.
4. M1.2 choice to prepare: small state store library (Rules.md 4.2) —
   propose candidates with licenses during M1.2.

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
