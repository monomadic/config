#!/usr/bin/env python3
"""
sanitize_filenames.py
Sanitize filenames safely on macOS/APFS.

What it does:
- removes control characters
- removes selected invisible formatting characters
- collapses repeated whitespace
- trims leading/trailing whitespace
- preserves extension
- avoids fake collisions on macOS caused by Unicode normalization

What it does NOT do:
- force NFC renames on macOS/APFS
  (that can cause repeated " (2)" suffixes because APFS is normalization-insensitive)

Usage:
  python3 sanitize_filenames.py /path/to/root
  python3 sanitize_filenames.py --apply /path/to/root
"""

from __future__ import annotations

import re
import sys
import unicodedata
from pathlib import Path

APPLY = "--apply" in sys.argv

# Control chars + a few invisible formatting chars that are often annoying in filenames.
STRIP_CODEPOINTS = {
    "\u200b",  # zero width space
    "\u200c",  # zero width non-joiner
    "\u200d",  # zero width joiner
    "\ufeff",  # BOM / zero width no-break space
    "\u2060",  # word joiner
}

def is_bad_char(ch: str) -> bool:
    cat = unicodedata.category(ch)
    if cat == "Cc":
        return True
    if ch in STRIP_CODEPOINTS:
        return True
    return False

def clean_name(name: str) -> str:
    # Do NOT normalize to NFC here on macOS/APFS.
    # That is what caused the fake collision loop.
    cleaned = "".join(ch for ch in name if not is_bad_char(ch))
    cleaned = re.sub(r"\s+", " ", cleaned).strip()
    cleaned = cleaned.replace("/", "_")
    return cleaned or "_"

def same_logical_name(a: str, b: str) -> bool:
    # Treat canonically equivalent names as "the same".
    # This prevents fake collisions like:
    #   engañadola  vs  engañadola
    return unicodedata.normalize("NFC", a) == unicodedata.normalize("NFC", b)

def unique_path_for_rename(src: Path, desired_name: str) -> Path:
    candidate = src.with_name(desired_name)

    # If the destination name is logically the same as the current filename,
    # treat it as no-op and do not suffix.
    if same_logical_name(src.name, desired_name):
        return src

    # If path does not exist, fine.
    if not candidate.exists():
        return candidate

    # If it does exist and is the same file, also fine.
    try:
        if candidate.samefile(src):
            return src
    except FileNotFoundError:
        pass
    except OSError:
        pass

    stem = candidate.stem
    suffix = candidate.suffix
    n = 2

    while True:
        alt = candidate.with_name(f"{stem} ({n}){suffix}")
        if not alt.exists():
            return alt
        try:
            if alt.samefile(src):
                return src
        except FileNotFoundError:
            pass
        except OSError:
            pass
        n += 1

def process_entry(p: Path) -> None:
    cleaned = clean_name(p.name)
    target = unique_path_for_rename(p, cleaned)

    if target == p:
        return

    print(f"{str(p)!r} -> {str(target)!r}")
    if APPLY:
        p.rename(target)

def main() -> int:
    argv = [a for a in sys.argv[1:] if a != "--apply"]
    if len(argv) != 1:
        print(f"usage: {Path(sys.argv[0]).name} [--apply] ROOT", file=sys.stderr)
        return 2

    root = Path(argv[0])

    if not root.exists():
        print(f"error: not found: {root}", file=sys.stderr)
        return 1

    # deepest-first so directory renames do not break traversal
    entries = sorted(root.rglob("*"), key=lambda p: len(p.parts), reverse=True)

    for p in entries:
        process_entry(p)

    return 0

if __name__ == "__main__":
    raise SystemExit(main())
