"""Enforce the repository file-length limit.

Implements the pre-commit "file-length check" from milestone M1.1 and
Rules.md 4.1: at most 300 lines per file, except generated files and
tests. Long files are split into modules instead.

    python tools/check-file-length.py

Exit code 0 when every source file is within the limit; exit code 1
listing the offenders with their line counts, so the author knows what
to split. The tool never edits files.

Scope: source-code suffixes only (see SOURCE_SUFFIXES). Markdown, YAML,
JSON/TOML configuration, lock files and the license are not code. Test
files are exempt (Rules.md 4.1), as are generated files (detected by
conventional names: *.min.*, *generated*, files under node_modules,
dist, build, out, target).
"""

from __future__ import annotations

import fnmatch
import subprocess
import sys
from pathlib import Path

MAX_LINES = 300

SOURCE_SUFFIXES = {
    ".ts", ".tsx", ".js", ".mjs", ".cjs", ".jsx", ".rs", ".py",
    ".cpp", ".cc", ".cxx", ".hpp", ".hh", ".h", ".c", ".xhtml",
    ".css", ".sh", ".ps1", ".sql",
}

# Filename or path patterns that mark a file as a test (exempt).
TEST_PATTERNS = ["*.test.ts", "*.test.tsx", "*.test.js", "*_test.py",
                 "test_*.py", "*_test.rs", "*_test.*", "*/tests/*",
                 "*/test/*"]

# Filename or path patterns that mark a file as generated or vendored.
GENERATED_PATTERNS = ["*.min.*", "*generated*", "*/node_modules/*",
                      "*/dist/*", "*/build/*", "*/out/*", "*/target/*",
                      "*/vendor/*", "*/.git/*"]


def list_files() -> list[Path]:
    """Return files to scan: git-tracked files if in a repo, else a walk."""
    if Path(".git").exists():
        result = subprocess.run(
            ["git", "ls-files", "-z"], capture_output=True, check=True
        )
        names = result.stdout.decode("utf-8", errors="replace").split("\0")
        paths = [Path(n) for n in names if n]
    else:
        paths = [p for p in Path(".").rglob("*") if p.is_file()]
    return [p for p in paths
            if p.suffix.lower() in SOURCE_SUFFIXES]


def is_exempt(path: Path) -> bool:
    """Return True for test and generated files, which have no length limit."""
    as_posix = path.as_posix()
    name = path.name
    return (any(fnmatch.fnmatch(name, pat) for pat in TEST_PATTERNS)
            or any(fnmatch.fnmatch(as_posix, pat) for pat in TEST_PATTERNS)
            or any(fnmatch.fnmatch(as_posix, pat) for pat in GENERATED_PATTERNS))


def count_lines(path: Path) -> int | None:
    """Count lines in a file; return None if it cannot be read as text."""
    try:
        return len(path.read_text(encoding="utf-8", errors="replace")
                   .splitlines())
    except OSError as err:
        print(f"error: cannot read {path}: {err}", file=sys.stderr)
        return None


def main() -> int:
    """Check all source files; print offenders and return the exit code."""
    offenders = 0
    for path in list_files():
        if is_exempt(path):
            continue
        lines = count_lines(path)
        if lines is None:
            return 1
        if lines > MAX_LINES:
            offenders += 1
            print(f"{path}: {lines} lines (limit {MAX_LINES}). "
                  "Split it into modules (Rules.md 4.1).")
    if offenders:
        print(f"\ncheck-file-length: {offenders} file(s) over the limit.",
              file=sys.stderr)
        return 1
    print(f"check-file-length: all source files within {MAX_LINES} lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
