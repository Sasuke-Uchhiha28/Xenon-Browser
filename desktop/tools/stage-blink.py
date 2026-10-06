"""Stage the Xenon Blink host into the fetched CEF distribution.

The CEF distribution's root CMakeLists.txt builds every application under
tests/ as a first-class target (that is how cefsimple itself builds).
This script copies `desktop/blink/app/` into the distribution as
`tests/xenon/` and registers it in the root CMakeLists.txt with one
`add_subdirectory` line. The distribution tree is a git-ignored build
artifact (ENG-05); staging is deterministic and re-runnable.

    python desktop/tools/stage-blink.py --platform linux64
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
APP_SRC = REPO_ROOT / "blink" / "app"
CEF_ROOT = REPO_ROOT / "blink" / "cef"

MARKER = "# xenon-blink (added by desktop/tools/stage-blink.py)"


def stage(platform: str) -> int:
    """Copy the app sources in and register the target in the root CMake."""
    cef_dir = CEF_ROOT / platform
    root_cmake = cef_dir / "CMakeLists.txt"
    if not root_cmake.is_file():
        print(f"error: {root_cmake} not found — run fetch-cef.py first")
        return 1

    target_dir = cef_dir / "tests" / "xenon"
    if target_dir.exists():
        shutil.rmtree(target_dir)
    target_dir.mkdir(parents=True)
    for source in sorted(APP_SRC.iterdir()):
        if source.is_file():
            shutil.copy2(source, target_dir / source.name)
    print(f"staged {len(list(target_dir.iterdir()))} files into "
          f"tests/xenon ({platform})")

    text = root_cmake.read_text(encoding="utf-8")
    if MARKER not in text:
        line = f"\n{MARKER}\nadd_subdirectory(tests/xenon)\n"
        anchor = "add_subdirectory(tests/cefsimple)\n"
        if anchor not in text:
            print("error: could not find the cefsimple add_subdirectory "
                  "anchor in the root CMakeLists.txt")
            return 1
        text = text.replace(anchor, anchor + line, 1)
        root_cmake.write_text(text, encoding="utf-8")
        print("registered add_subdirectory(tests/xenon) in the root CMakeLists")
    else:
        print("root CMakeLists already registers tests/xenon")
    return 0


def main() -> int:
    """Stage the app for the requested platform."""
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--platform", required=True,
                        choices=["windows64", "linux64"])
    args = parser.parse_args()
    return stage(args.platform)


if __name__ == "__main__":
    sys.exit(main())
