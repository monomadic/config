#!/usr/bin/env python3
"""
audit-filenames.py
Audit directory entries for strict UTF-8 validity, Unicode normalization,
and control characters.

Usage:
  python3 audit_filenames.py /path/to/dir
"""

from __future__ import annotations

import os
import sys
import unicodedata
from pathlib import Path

def has_control_chars(s: str) -> bool:
    return any(unicodedata.category(ch).startswith("C") for ch in s if ch not in "\t\n\r")

def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {Path(sys.argv[0]).name} DIR", file=sys.stderr)
        return 2

    root = os.fsencode(sys.argv[1])

    try:
        entries = os.listdir(root)
    except OSError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    bad_utf8 = []
    non_nfc = []
    controly = []

    for entry_b in entries:
        if not isinstance(entry_b, bytes):
            entry_b = os.fsencode(entry_b)

        try:
            name = entry_b.decode("utf-8", "strict")
        except UnicodeDecodeError:
            bad_utf8.append(entry_b)
            continue

        if unicodedata.normalize("NFC", name) != name:
            non_nfc.append(name)

        if has_control_chars(name):
            controly.append(name)

    if bad_utf8:
        print("Invalid UTF-8:")
        for b in bad_utf8:
            print("  ", repr(b))

    if non_nfc:
        print("\nValid UTF-8 but not NFC:")
        for s in non_nfc:
            print("  ", s)

    if controly:
        print("\nContains control/invisible chars:")
        for s in controly:
            print("  ", repr(s))

    if not (bad_utf8 or non_nfc or controly):
        print("All filenames passed.")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
