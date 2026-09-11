#!/bin/sh
# Install yt-dlp from uv with curl_cffi, and unlink Homebrew's copy so it cannot
# shadow it. Idempotent — safe to re-run, and re-run is how you repair the drift
# described below.
#
# Why not just `brew install yt-dlp`: brew ships yt-dlp as a venv with no
# curl_cffi, so it reports zero impersonation targets. Extractors that rely on
# impersonation (PornHub and friends) then fail in a way that does NOT look like
# a dependency problem — extraction "succeeds", a dead HLS format is chosen, and
# the download dies later with "HTTP Error 474".
#
# Why unlink rather than uninstall: mpv *hard*-depends on the yt-dlp keg, so brew
# refuses to remove it. Unlinking drops $(brew --prefix)/bin/yt-dlp while leaving
# the keg installed, which keeps mpv's dependency satisfied.
#
# Why the brew symlink has to go at all: bin/init-path APPENDS, so the inherited
# /opt/homebrew/bin sits ahead of ~/.local/bin. A uv install alone would be
# silently shadowed by the brew copy.
#
# Why a shim is then put BACK at $(brew --prefix)/bin/yt-dlp: ~/.local/bin is only
# on PATH for zsh sessions, because it comes from .zshenv -> bin/init-path.
# /opt/homebrew/bin is system-wide via /etc/paths.d/homebrew, so anything not
# started through the zsh config (TUIs spawned by a launcher, GUI-parented
# processes) sees the brew dir and not ~/.local/bin. Removing the brew copy
# without replacing it breaks those callers — tagform shelling out to yt-dlp is
# the case that surfaced it. The shim points at the uv build, so every context
# gets one yt-dlp and it is the one with curl_cffi.
#
# Note that `brew upgrade yt-dlp` RELINKS the brew copy and silently reintroduces
# the bug. setup/macos/check.sh warns when that has happened; re-run this script
# to repair it.

set -e

UV_BIN="$HOME/.local/bin/yt-dlp"

if ! command -v uv >/dev/null 2>&1; then
  echo "Error: uv not found on PATH (needed to install yt-dlp)" >&2
  exit 1
fi

# "Working" means impersonation targets are actually available. A target line
# ends in `curl_cffi`; the broken build lists the same clients as
# `curl_cffi (unavailable)`, so anchor the match to end-of-line.
ytdlp_has_impersonation() {
  [ -x "$1" ] || return 1
  "$1" --list-impersonate-targets 2>/dev/null | grep -qE 'curl_cffi$'
}

if ytdlp_has_impersonation "$UV_BIN"; then
  echo "yt-dlp (uv) already has impersonation support"
else
  echo "Installing yt-dlp with curl_cffi via uv..."
  uv tool install --force yt-dlp --with curl_cffi
fi

# Drop the Homebrew symlink if the keg is installed and currently linked, then
# put our own shim in its place. `brew unlink` removes the whole set of links it
# owns, so it has to happen before the shim is written, not after.
if command -v brew >/dev/null 2>&1; then
  brew_bin="$(brew --prefix)/bin/yt-dlp"

  if brew list --versions yt-dlp >/dev/null 2>&1 && [ -e "$brew_bin" ] && [ ! -L "$brew_bin" ]; then
    echo "Unlinking Homebrew's yt-dlp (keg stays installed for mpv)..."
    brew unlink yt-dlp
  elif [ -L "$brew_bin" ] && [ ! "$brew_bin" -ef "$UV_BIN" ]; then
    # A symlink that is not ours — brew relinked over the shim.
    echo "Replacing Homebrew's yt-dlp link..."
    brew list --versions yt-dlp >/dev/null 2>&1 && brew unlink yt-dlp
  fi

  if [ ! "$brew_bin" -ef "$UV_BIN" ]; then
    echo "Linking $brew_bin -> $UV_BIN"
    ln -sfn "$UV_BIN" "$brew_bin"
  fi
fi

resolved="$(command -v yt-dlp || true)"

if ! ytdlp_has_impersonation "$UV_BIN"; then
  echo "Error: $UV_BIN still reports no impersonation targets" >&2
  exit 1
fi

# `-ef` not a string compare: resolving via the shim is the expected outcome, so
# what matters is that PATH lands on the same file, not on the same spelling.
if [ -z "$resolved" ] || [ ! "$resolved" -ef "$UV_BIN" ]; then
  echo "Warning: yt-dlp on PATH is ${resolved:-<none>}, which is not $UV_BIN" >&2
  echo "         Something ahead of it on PATH is shadowing it." >&2
  exit 1
fi

echo "yt-dlp: $resolved ($("$UV_BIN" --version))"
