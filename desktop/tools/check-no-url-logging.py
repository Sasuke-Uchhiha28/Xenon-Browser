"""Lint rule: reject logging of URLs and other sensitive user data.

Implements the pre-commit "no URL logging" check from milestone M1.1 and
PRIV-05: URLs, queries, page titles, search text, passwords, note text
and cookie values are never written to console, files or any store. The
rule fails any logging call whose argument text mentions a sensitive
identifier, for example:

    console.log("navigated to", url)        // FAIL
    log::info!("request {} finished", url); // FAIL
    logger.debug("query: %s", query)        // FAIL

Limitations (v1): the rule inspects logging calls line by line, so a
sensitive variable spread over several lines or passed via a helper will
not be caught. It reduces obvious mistakes; code review and the filter
conformance and packet-capture tests remain the real enforcement.

The rule never scans itself or the other pre-commit tools, because they
must quote the very words they look for; this is not an exemption for
product code.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

# Logging entry points per language family. Each pattern matches the call
# opening; the argument text from there to end of line is then checked.
LOG_CALL_PATTERNS: dict[str, re.Pattern[str]] = {
    "python": re.compile(
        r"\b(?:print|logging\.\w+|logger\.\w+|log\.\w+)\s*\("
    ),
    "js-ts": re.compile(
        r"\b(?:console\.(?:log|info|warn|error|debug|trace)"
        r"|(?:log|logger|logging)\.(?:debug|info|warn|error|trace|log))\s*\("
    ),
    "rust": re.compile(
        r"\b(?:println!|eprintln!|print!|eprint!|todo!|unimplemented!)\b"
        r"|\b(?:log::)?(?:trace|debug|info|warn|error)!\s*\("
    ),
    "cpp": re.compile(
        r"\b(?:printf|fprintf|puts|perror|qWarning|qDebug|qInfo|qCritical)\s*\("
        r"|\b(?:LOG|VLOG|DLOG|LOG_IF|PLOG)\s*\(|\bstd::c(?:err|log)\s*<<"
    ),
}

# Identifiers whose presence in a log argument is a PRIV-05 violation.
# Word fragments are matched in identifier position (camelCase, snake_case
# or kebab-case) or as string labels ("url:", "URL =").
SENSITIVE_FRAGMENTS: tuple[re.Pattern[str], ...] = (
    re.compile(r"(?i)\burl|\bur[il]\b"),
    re.compile(r"(?i)\buri\b"),
    re.compile(r"(?i)\bqueries?\b|\bquery[_-]?(?:text|string|param)"),
    re.compile(r"(?i)\bsearch(?:text|_text|-text|_term|-term|term)\b"),
    re.compile(r"(?i)\bpassword|passwd\b"),
    re.compile(r"(?i)\bcookie"),
    re.compile(r"(?i)\bauth[_-]?token|\btoken\b"),
    re.compile(r"(?i)\b(?:page|tab|document|window)[_-]?title\b"),
    re.compile(r"(?i)\breferrer\b"),
    re.compile(r"(?i)\bnote[_-]?text\b|\bnotetext\b"),
    re.compile(r"(?i)\bhistory[_-]?entry\b"),
)

SOURCE_SUFFIXES = {
    ".ts", ".tsx", ".js", ".mjs", ".cjs", ".jsx", ".rs", ".py",
    ".cpp", ".cc", ".cxx", ".hpp", ".hh", ".h", ".c", ".xhtml", ".kt",
}

# Files that must quote sensitive words to define the rule itself:
# this script and the secrets scanner. Matched by file name; no product
# source file carries these names.
TOOL_SELF_EXEMPT = {"check-no-url-logging.py", "check-secrets.py"}

BINARY_SUFFIXES = {
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".pdf", ".zip",
    ".woff", ".woff2", ".ttf", ".otf",
}


def family_for(path: Path) -> str | None:
    """Return the language family of a file, or None if out of scope."""
    suffix = path.suffix.lower()
    if suffix == ".py":
        return "python"
    if suffix in {".ts", ".tsx", ".js", ".mjs", ".cjs", ".jsx"}:
        return "js-ts"
    if suffix == ".rs":
        return "rust"
    if suffix in {".cpp", ".cc", ".cxx", ".hpp", ".hh", ".h", ".c"}:
        return "cpp"
    return None


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
    return [p for p in paths if p.suffix.lower() in SOURCE_SUFFIXES
            and p.name not in TOOL_SELF_EXEMPT]


def violations_in_line(line: str) -> list[str]:
    """Return the names of sensitive fragments found in a log call on a line."""
    for log_pattern in LOG_CALL_PATTERNS.values():
        match = log_pattern.search(line)
        if match is None:
            continue
        # Argument text: from the matched call to end of line. For call-style
        # loggers start after the opening parenthesis; for stream-style ones
        # (std::cerr <<) there is none, so check the rest of the line.
        call_text = line[match.start():]
        paren = call_text.find("(")
        argument_text = call_text[paren + 1:] if paren != -1 else call_text
        found = [frag.pattern for frag in SENSITIVE_FRAGMENTS
                 if frag.search(argument_text)]
        if found:
            return found
    return []


def main() -> int:
    """Scan source files; print violations and return the exit code."""
    findings = 0
    for path in list_files():
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as err:
            print(f"error: cannot read {path}: {err}", file=sys.stderr)
            return 1
        for line_number, line in enumerate(text.splitlines(), start=1):
            for fragment in violations_in_line(line):
                findings += 1
                print(f"{path}:{line_number}: PRIV-05 violation "
                      f"({fragment}) in logging call: {line.strip()[:90]}")
    if findings:
        print(
            f"\ncheck-no-url-logging: {findings} violation(s). URLs, queries, "
            "titles, search text, passwords, note text and cookie values are "
            "never logged (Rules.md PRIV-05). Log domain-only strings via the "
            "in-memory interception log instead.",
            file=sys.stderr,
        )
        return 1
    print("check-no-url-logging: no violations")
    return 0


if __name__ == "__main__":
    sys.exit(main())
