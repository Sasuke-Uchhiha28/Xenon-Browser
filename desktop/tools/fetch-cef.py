"""Fetch, verify and extract the pinned CEF binary distributions.

Implements the download side of M1.4 and Architecture.md 11: CEF pinned,
checksum-verified, never committed (ENG-05, SEC-03). Run from the
repository root:

    python desktop/tools/fetch-cef.py --platform linux64
    python desktop/tools/fetch-cef.py                # both platforms

The pinned version is the newest CEF stable whose standard distribution
exists for BOTH windows64 and linux64 at the time of pinning (2026-10-06).
Checksums are SHA-1 as published by the official CEF builds CDN; CEF does
not publish stronger digests. The pinned hash still detects corruption
and accidental version drift (SEC-03). Extracted trees land in
`desktop/blink/cef/<platform>/` with the tarball's top-level directory
stripped, so build paths stay stable across CEF point updates.
"""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
import tarfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CEF_OUT = REPO_ROOT / "blink" / "cef"
CDN = "https://cef-builds.spotifycdn.com"

CEF_VERSION = "154.0.34+g14c5a08+chromium-154.0.8037.98"

# Platform -> (tarball file name, SHA-1, size in bytes from the CEF index).
SOURCES: dict[str, dict[str, str | int]] = {
    "windows64": {
        "file": f"cef_binary_{CEF_VERSION}_windows64.tar.bz2",
        "sha1": "8c705a2ef1ce5f7b9655729a2d3de5d033a24d8b",
        "size": 365_400_000,
    },
    "linux64": {
        "file": f"cef_binary_{CEF_VERSION}_linux64.tar.bz2",
        "sha1": "d0a822afcf769371e961344b64f5a461066b6a0f",
        "size": 689_400_000,
    },
}


def sha1_of(path: Path) -> str:
    """Stream a file through SHA-1 and return the hex digest."""
    digest = hashlib.sha1()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(4 * 1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def download(platform: str) -> Path:
    """Download one pinned CEF tarball and verify its SHA-1."""
    source = SOURCES[platform]
    file_name = str(source["file"])
    url = f"{CDN}/{file_name}"
    target = CEF_OUT / f"{platform}.tar.bz2"
    CEF_OUT.mkdir(parents=True, exist_ok=True)

    print(f"downloading {file_name} ...")
    result = subprocess.run(
        ["curl", "-sSL", "--max-time", "1800", "-o", str(target), url],
        check=False,
    )
    if result.returncode != 0:
        raise SystemExit(f"download failed for {platform} (curl exit {result.returncode})")

    actual = sha1_of(target)
    if actual != source["sha1"]:
        target.unlink(missing_ok=True)
        raise SystemExit(
            f"checksum mismatch for {platform}: expected {source['sha1']}, got {actual}. "
            "Refusing to build against unverified CEF (SEC-03)."
        )
    print(f"  sha1 verified: {actual}")
    return target


def extract(tarball: Path, platform: str) -> Path:
    """Extract into blink/cef/<platform>/, stripping the top-level dir."""
    out_dir = CEF_OUT / platform
    out_dir.mkdir(parents=True, exist_ok=True)
    print(f"extracting into {out_dir.relative_to(REPO_ROOT)} ...")
    with tarfile.open(tarball, "r:bz2") as tar:
        for member in tar.getmembers():
            parts = member.name.split("/", 1)
            if len(parts) != 2 or not parts[1]:
                continue  # skip the top-level directory entry itself
            member.name = parts[1]
            tar.extract(member, out_dir, filter="data")
    print(f"  extracted {sum(1 for _ in out_dir.rglob('*'))} entries")
    return out_dir


def main() -> int:
    """Download, verify and extract the requested CEF distributions."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--platform",
        choices=[*SOURCES.keys(), "all"],
        default="all",
        help="which distribution to fetch (default: all)",
    )
    args = parser.parse_args()
    platforms = list(SOURCES) if args.platform == "all" else [args.platform]

    for platform in platforms:
        tarball = download(platform)
        extract(tarball, platform)
        tarball.unlink()  # keep the checked-out tree small
    print("CEF fetch complete.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
