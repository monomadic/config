#!/usr/bin/env python3
"""
sanitize-filenames.py
Sanitize filenames safely on macOS/APFS without triggering Unicode
normalization collision loops.

What it does:
- strips control chars
- strips invisible/formatting chars, including bidi controls
- strips surrogate/private-use/unassigned codepoints
- collapses pathological whitespace to a single normal space
- trims leading/trailing whitespace
- replaces '/' with '_'
- avoids fake collisions from canonically equivalent Unicode names on APFS

What it does NOT do:
- force NFC/NFD renames in place on macOS/APFS

Usage:
  sanitize-filenames.py ROOT
  sanitize-filenames.py --apply ROOT

Examples:
  sanitize-filenames.py .
  sanitize-filenames.py --apply .
"""

from __future__ import annotations

import argparse
import re
import sys
import unicodedata
from pathlib import Path


WHITESPACE_RE = re.compile(r"\s+")


def is_bad_char(ch: str) -> bool:
    cat = unicodedata.category(ch)

    # Cc = control chars
    # Cf = format chars (includes bidi controls, zero-width chars, BOM, etc.)
    # Cs = surrogate
    # Co = private use
    # Cn = unassigned
    return cat in {"Cc", "Cf", "Cs", "Co", "Cn"}


def clean_name(name: str) -> str:
    cleaned = "".join(ch for ch in name if not is_bad_char(ch))

    # Normalize all whitespace runs to a single plain space, then trim ends.
    cleaned = WHITESPACE_RE.sub(" ", cleaned).strip()

    # Filenames cannot contain slash as a path component separator.
    cleaned = cleaned.replace("/", "_")

    # Avoid empty filename after sanitization.
    return cleaned or "_"


def canonical_key(name: str) -> str:
    # Use NFC only for comparison/collision detection, not for renaming.
    return unicodedata.normalize("NFC", name)


def same_logical_name(a: str, b: str) -> bool:
    return canonical_key(a) == canonical_key(b)


def unique_target(src: Path, desired_name: str) -> Path:
    candidate = src.with_name(desired_name)

    # No-op if the desired name is canonically equivalent to the current name.
    # This avoids APFS normalization-insensitive collision loops.
    if same_logical_name(src.name, desired_name):
        return src

    if not candidate.exists():
        return candidate

    try:
        if candidate.samefile(src):
            return src
    except (FileNotFoundError, OSError):
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
        except (FileNotFoundError, OSError):
            pass

        n += 1


def process_entry(entry: Path, apply_changes: bool) -> bool:
    cleaned_name = clean_name(entry.name)
    target = unique_target(entry, cleaned_name)

    if target == entry:
        return False

    print(f"{str(entry)!r} -> {str(target)!r}")

    if apply_changes:
        entry.rename(target)

    return True


def iter_entries_deepest_first(root: Path) -> list[Path]:
    # Rename deepest paths first so directory renames do not break traversal.
    return sorted(root.rglob("*"), key=lambda p: len(p.parts), reverse=True)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Sanitize filenames safely on macOS/APFS."
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="perform renames; without this, only print planned changes",
    )
    parser.add_argument(
        "root",
        help="root directory to scan",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    root = Path(args.root)

    if not root.exists():
        print(f"error: not found: {root}", file=sys.stderr)
        return 1

    if not root.is_dir():
        print(f"error: not a directory: {root}", file=sys.stderr)
        return 1

    changed = 0

    for entry in iter_entries_deepest_first(root):
        try:
            if process_entry(entry, args.apply):
                changed += 1
        except OSError as exc:
            print(f"error: failed to process {entry!r}: {exc}", file=sys.stderr)

    if changed == 0:
        print("No changes needed.")
    elif not args.apply:
        print("\nDry run only. Re-run with --apply to rename.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
