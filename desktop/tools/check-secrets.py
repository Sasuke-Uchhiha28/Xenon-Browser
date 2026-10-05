"""Scan the repository for accidentally committed secrets.

Implements the pre-commit "secrets scan" check from milestone M1.1 and
PRIV-09 (secrets are never committed). Run it manually or from the
pre-commit hook / CI:

    python tools/check-secrets.py

Exit code 0 means nothing suspicious was found; exit code 1 lists every
match with file, line and pattern name so the author can remove the
secret from the commit. The tool reports findings only — it never edits
files and never "fixes" them by masking.

Scope: when run inside a git repository only git-tracked files are
scanned, which automatically excludes engines and build trees (ENG-05).
Outside a repository it walks the current directory with standard
excludes. Test fixtures are scanned too (PRIV-09 has no test exception).
"""

from __future__ import annotations

import fnmatch
import re
import subprocess
import sys
from pathlib import Path

# (name, compiled pattern). Keep patterns conservative: a scan that cries
# wolf gets ignored, and the file-length and URL-logging checks cover the
# rest of hygiene. Generic key=value matches require a value that looks
# machine-generated (long enough) to avoid flagging documentation text.
SECRET_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("pem-private-key", re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----")),
    ("aws-access-key", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("github-token", re.compile(r"\bgh[pousr]_[A-Za-z0-9]{36,}\b")),
    ("slack-token", re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b")),
    ("google-api-key", re.compile(r"\bAIza[0-9A-Za-z_\-]{35}\b")),
    (
        "generic-key-assignment",
        re.compile(
            r"""(?ix)\b(api[_-]?key|secret|access[_-]?token|auth[_-]?token|password|passwd)\b"""
            r"""\s*[:=]\s*["']([A-Za-z0-9_\-/+=]{12,})["']"""
        ),
    ),
]

# Values that look like keys but are documentation placeholders, not secrets.
PLACEHOLDER_VALUES = {
    "changeme", "change-me", "example", "placeholder", "your-key-here",
    "insert-key-here", "dummy", "sample", "test-value", "xxxxxxxxxxxx",
}

# File types that never contain text secrets worth scanning.
BINARY_SUFFIXES = {
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".pdf", ".zip",
    ".tar", ".gz", ".bz2", ".xz", ".7z", ".exe", ".dll", ".so", ".dylib",
    ".woff", ".woff2", ".ttf", ".otf", ".mp3", ".mp4", ".wasm",
}

# Directories excluded when no git repository is available.
WALK_EXCLUDES = {".git", "node_modules", "target", "dist", "build", "out",
                 "__pycache__", ".venv"}


class UnreadableFile(Exception):
    """Raised when a file in scope cannot be read."""


def list_files() -> list[Path]:
    """Return the files to scan: git-tracked files if in a repo, else a walk.

    Returns a sorted list of paths. Git failures raise SystemExit so a
    broken checkout stops the commit instead of scanning nothing.
    """
    if Path(".git").exists():
        result = subprocess.run(
            ["git", "ls-files", "-z"], capture_output=True, check=True
        )
        names = result.stdout.decode("utf-8", errors="replace").split("\0")
        return [Path(n) for n in names if n]
    files: list[Path] = []
    for path in Path(".").rglob("*"):
        if any(fnmatch.fnmatch(part, pat) for part in path.parts
               for pat in WALK_EXCLUDES):
            continue
        if path.is_file():
            files.append(path)
    return sorted(files)


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    """Scan one file; return (line number, pattern name, matched text) hits.

    Raises UnreadableFile if the file exists but cannot be read, so the
    caller can fail the check instead of silently skipping content.
    Binary files (by suffix) are skipped.
    """
    if path.suffix.lower() in BINARY_SUFFIXES:
        return []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as err:
        raise UnreadableFile(f"{path}: {err}") from err
    hits: list[tuple[int, str, str]] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        for name, pattern in SECRET_PATTERNS:
            for match in pattern.finditer(line):
                if name == "generic-key-assignment" and is_placeholder(match.group(2)):
                    continue
                hits.append((line_number, name, match.group(0)))
    return hits


def is_placeholder(value: str) -> bool:
    """Return True for values that are clearly documentation placeholders."""
    return value.lower() in PLACEHOLDER_VALUES or set(value) <= set("x0")


def main() -> int:
    """Scan all in-scope files; print findings and return the exit code."""
    findings = 0
    for path in list_files():
        try:
            hits = scan_file(path)
        except UnreadableFile as err:
            print(f"error: cannot read file in scope: {err}", file=sys.stderr)
            return 1
        for line_number, name, matched in hits:
            findings += 1
            shown = matched if len(matched) <= 60 else matched[:57] + "..."
            print(f"{path}:{line_number}: {name}: {shown}")
    if findings:
        print(
            f"\ncheck-secrets: {findings} finding(s). Remove the secret from "
            "the commit (PRIV-09). If a match is a false positive, tighten "
            "the pattern instead of excluding the file.",
            file=sys.stderr,
        )
        return 1
    print("check-secrets: no findings")
    return 0


if __name__ == "__main__":
    sys.exit(main())
