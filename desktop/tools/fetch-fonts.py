"""Fetch, verify and build the self-hosted UI fonts (Design.md 4.1).

Downloads pinned font sources, verifies their SHA-256 against
`desktop/tools/fonts-manifest.json`, and writes the final web fonts into
`desktop/ui/public/fonts/`. The generated woff2 files and their licenses
are committed, so normal builds never run this script; re-run it only
when the pinned sources move or when new Material Symbols icons are used.

Usage (from the repository root):
    python desktop/tools/fetch-fonts.py                 # download + verify + build
    python desktop/tools/fetch-fonts.py --write-manifest  # regenerate the checksum manifest

Licenses: Space Grotesk, Geist and JetBrains Mono are SIL OFL 1.1;
Material Symbols is Apache-2.0. License files are copied next to the
fonts. NOTE: OFL is not yet in the Rules.md 8.1 allowed list — recorded
in memory.md for owner ratification.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = Path(__file__).resolve().parent / "fonts-manifest.json"
FONTS_OUT = REPO_ROOT / "ui" / "public" / "fonts"

# Icon names used by the UI (Material Symbols Rounded). Add a name here,
# re-run this script, and commit the new subset + manifest + glyph map.
ICON_NAMES = [
    "add", "close", "arrow_back", "arrow_forward", "refresh", "search",
    "shield", "more_vert", "public", "workspaces", "terminal", "key",
    "settings", "history", "download", "extension",
    "chevron_left", "chevron_right",
]

# Emitted TypeScript map: ui/src/components/icon-glyphs.ts
GLYPHS_OUT = REPO_ROOT / "ui" / "src" / "components" / "icon-glyphs.ts"

# Every pinned source: key, download URL, sha256 (hex). The Google sources
# are pinned to a specific commit so the bytes never change underneath us.
SOURCES: dict[str, dict[str, str]] = {
    "space-grotesk-ttf": {
        "url": "https://raw.githubusercontent.com/google/fonts/"
               "9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/spacegrotesk/"
               "SpaceGrotesk%5Bwght%5D.ttf",
    },
    "space-grotesk-ofl": {
        "url": "https://raw.githubusercontent.com/google/fonts/"
               "9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/spacegrotesk/OFL.txt",
    },
    "geist-tgz": {
        "url": "https://registry.npmjs.org/geist/-/geist-1.7.2.tgz",
    },
    "jetbrains-zip": {
        "url": "https://github.com/JetBrains/JetBrainsMono/releases/download/"
               "v2.304/JetBrainsMono-2.304.zip",
    },
    "material-ttf": {
        "url": "https://raw.githubusercontent.com/google/material-design-icons/"
               "737e3324305806514d7909874fa1818ae1808232/variablefont/"
               "MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf",
    },
    "material-codepoints": {
        "url": "https://raw.githubusercontent.com/google/material-design-icons/"
               "737e3324305806514d7909874fa1818ae1808232/variablefont/"
               "MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.codepoints",
    },
    "apache-2": {"url": "https://www.apache.org/licenses/LICENSE-2.0.txt"},
}


def sha256(data: bytes) -> str:
    """Return the hex SHA-256 digest of a byte string."""
    return hashlib.sha256(data).hexdigest()


def download(key: str) -> bytes:
    """Download one pinned source and verify its checksum against the manifest."""
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    expected = manifest[key]["sha256"]
    result = subprocess.run(
        ["curl", "-sSL", "--max-time", "120", SOURCES[key]["url"]],
        capture_output=True, check=True,
    )
    actual = sha256(result.stdout)
    if actual != expected:
        raise SystemExit(
            f"checksum mismatch for {key}: expected {expected}, got {actual}. "
            "If the source moved on purpose, re-pin it and run with --write-manifest."
        )
    print(f"  ok {key} ({len(result.stdout)} bytes)")
    return result.stdout


def write_woff2(font: TTFont, out_path: Path) -> None:
    """Save a font as woff2 and print the resulting size."""
    font.flavor = "woff2"
    font.save(out_path)
    print(f"  wrote {out_path.name} ({out_path.stat().st_size} bytes)")


def build_material_symbols(
    material_ttf: bytes, codepoints_text: str
) -> tuple[TTFont, dict[str, int]]:
    """Instance the variable icon font at UI values and subset it to used icons.

    Returns the font and the name-to-codepoint map for the emitted TS glyph file.
    """
    codepoints: dict[str, int] = {}
    for line in codepoints_text.splitlines():
        name, _, code = line.partition(" ")
        codepoints[name] = int(code, 16)
    missing = [name for name in ICON_NAMES if name not in codepoints]
    if missing:
        raise SystemExit(f"icon names not found in codepoints file: {missing}")
    points = [codepoints[name] for name in ICON_NAMES]
    font = TTFont(io.BytesIO(material_ttf))
    instantiateVariableFont(
        font, {"FILL": 0, "GRAD": 0, "opsz": 24, "wght": 400}, inplace=True
    )
    options = subset.Options()
    options.drop_tables += ["GSUB", "GPOS"]  # icons render by codepoint, no shaping
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=points)
    subsetter.subset(font)
    return font, {name: codepoints[name] for name in ICON_NAMES}


def write_glyph_map(name_to_code: dict[str, int]) -> None:
    """Emit ui/src/components/icon-glyphs.ts (name to glyph character)."""
    GLYPHS_OUT.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "/* GENERATED by desktop/tools/fetch-fonts.py - do not edit by hand.",
        "   Maps Material Symbols icon names to glyphs of the subset font. */",
        "export const ICON_GLYPHS = {",
    ]
    for name, code in name_to_code.items():
        lines.append(f"  {name}: '\\u{{{code:x}}}',")
    lines.append("} as const")
    lines.append("")
    lines.append("export type IconName = keyof typeof ICON_GLYPHS")
    GLYPHS_OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"  wrote {GLYPHS_OUT.relative_to(REPO_ROOT)}")


def main() -> int:
    """Download pinned sources, verify checksums, write the web fonts."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--write-manifest", action="store_true",
                        help="record current source checksums and exit")
    args = parser.parse_args()

    if args.write_manifest or not MANIFEST_PATH.exists():
        if not args.write_manifest:
            print(f"{MANIFEST_PATH} missing; generating it from fresh downloads.")
        print("Downloading sources to record checksums...")
        manifest = {}
        for key in SOURCES:
            result = subprocess.run(
                ["curl", "-sSL", "--max-time", "120", SOURCES[key]["url"]],
                capture_output=True, check=True,
            )
            manifest[key] = {"url": SOURCES[key]["url"],
                             "sha256": sha256(result.stdout)}
        MANIFEST_PATH.write_text(json.dumps(manifest, indent=2) + "\n",
                                 encoding="utf-8")
        print(f"wrote {MANIFEST_PATH}")
        if not args.write_manifest:
            return 0
        return 0

    print("Downloading and verifying pinned sources...")
    blobs = {key: download(key) for key in SOURCES}

    FONTS_OUT.mkdir(parents=True, exist_ok=True)
    licenses = FONTS_OUT / "licenses"
    licenses.mkdir(exist_ok=True)

    print("Building fonts...")
    space_grotesk = TTFont(io.BytesIO(blobs["space-grotesk-ttf"]))
    write_woff2(space_grotesk, FONTS_OUT / "SpaceGrotesk-Variable.woff2")
    (licenses / "LICENSE-space-grotesk.txt").write_bytes(blobs["space-grotesk-ofl"])

    with tempfile.TemporaryDirectory():
        with zipfile.ZipFile(io.BytesIO(blobs["jetbrains-zip"])) as zipped:
            for weight in ("Regular", "Medium", "SemiBold"):
                member = f"fonts/webfonts/JetBrainsMono-{weight}.woff2"
                (FONTS_OUT / f"JetBrainsMono-{weight}.woff2").write_bytes(
                    zipped.read(member)
                )
                print(f"  wrote JetBrainsMono-{weight}.woff2 "
                      f"({(FONTS_OUT / f'JetBrainsMono-{weight}.woff2').stat().st_size} bytes)")
            license_members = [n for n in zipped.namelist() if n == "OFL.txt"]
            if license_members:
                (licenses / "LICENSE-jetbrains-mono.txt").write_bytes(
                    zipped.read(license_members[0])
                )
        with tarfile.open(fileobj=io.BytesIO(blobs["geist-tgz"]), mode="r:gz") as tar:
            member = "package/dist/fonts/geist-sans/Geist-Variable.woff2"
            geist_bytes = tar.extractfile(member)
            if geist_bytes is None:
                raise SystemExit(f"{member} missing from geist tarball")
            (FONTS_OUT / "Geist-Variable.woff2").write_bytes(geist_bytes.read())
            print(f"  wrote Geist-Variable.woff2 "
                  f"({(FONTS_OUT / 'Geist-Variable.woff2').stat().st_size} bytes)")

    icons, name_to_code = build_material_symbols(
        blobs["material-ttf"], blobs["material-codepoints"].decode("utf-8")
    )
    write_woff2(icons, FONTS_OUT / "MaterialSymbols-Subset.woff2")
    write_glyph_map(name_to_code)
    (licenses / "LICENSE-material-symbols.txt").write_bytes(blobs["apache-2"])

    print(f"Done. Fonts in {FONTS_OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
