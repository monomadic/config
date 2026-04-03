#!/usr/bin/env python3

"""
toggle_repl kitten: toggle a kitty overlay shell window

Usage:
  kitty +kitten /path/to/toggle_repl.py [--title repl] [--cmd CMD] [--keep-focus]

Notes:
  - Relies on kitty remote control (`kitty @ ...`)
  - Uses `--cwd=current` so the overlay starts in the active window's cwd
"""

from __future__ import annotations

import argparse
import os
import shlex
import subprocess
from typing import List, Any


def rc(*args: str) -> subprocess.CompletedProcess[str]:
    to = os.environ.get("KITTY_LISTEN_ON")
    base = ["kitty", "@"]
    if not to:
        base += ["--to", f"unix:/tmp/kitty-{os.environ.get('USER', '')}"]
    return subprocess.run([*base, *args], capture_output=True, text=True)


def main(args: List[str]) -> Any:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--title", default="repl")
    parser.add_argument("--cmd", default=os.environ.get("SHELL", "zsh"))
    parser.add_argument("--keep-focus", action="store_true")
    parser.add_argument("args", nargs=argparse.REMAINDER)
    ns = parser.parse_args(args)

    title = ns.title
    cmd = ns.cmd
    if ns.args:
        # Allow command after "--" to pass through
        cmd = (cmd + " " + " ".join(ns.args)).strip()

    match_expr = f"state:overlay && title:^{title}$"

    # Toggle behavior implemented here using kitty remote control
    listed = rc("ls", "--match", match_expr)
    if '"id"' in listed.stdout:
        rc("close-window", "--match", match_expr)
        return None

    launch_args = [
        "launch", "--type=overlay", "--title", title, "--cwd", "current",
    ]
    if ns.keep_focus:
        launch_args.append("--keep-focus")
    # Run via sh -lc to allow complex commands
    launch_args += ["sh", "-lc", cmd]
    rc(*launch_args)
    return None


def handle_result(args: List[str], data: Any, target_window_id: int, boss) -> None:
    # All the work is done in main via kitty @, nothing to handle post-run.
    return None
