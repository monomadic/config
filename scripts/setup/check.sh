#!/usr/bin/env bash

set -euo pipefail

# The repo root is wherever this script physically lives, so the checkout can be
# cloned anywhere. Never assume ~/config.
DOTFILES_DIR="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
GLOBAL_CONFIG="$DOTFILES_DIR/dotter/global.toml"
LOCAL_CONFIG="$DOTFILES_DIR/dotter/local.toml"
ICON_SCRIPT="$DOTFILES_DIR/scripts/setup/apply-file-icons.sh"

errors=0
warnings=0

note() {
  printf '%s\n' "$*"
}

warn() {
  printf 'Warning: %s\n' "$*" >&2
  warnings=$((warnings + 1))
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  errors=$((errors + 1))
}

check_command() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    fail "missing required command: $cmd"
  fi
}

collect_packages() {
  local file="$1"

  awk '
    /^\[/ {
      section = $0
      gsub(/^\[/, "", section)
      gsub(/\]$/, "", section)
      split(section, parts, ".")
      root = parts[1]
      if (root != "helpers" && root != "settings") {
        print root
      }
    }
  ' "$file" | sort -u
}

collect_selected_packages() {
  local file="$1"

  awk '
    /^[[:space:]]*packages[[:space:]]*=[[:space:]]*\[/ { in_packages = 1; next }
    in_packages {
      line = $0
      sub(/#.*/, "", line)
      if (line ~ /\]/) {
        in_packages = 0
      }
      while (match(line, /"[^"]+"/)) {
        pkg = substr(line, RSTART + 1, RLENGTH - 2)
        print pkg
        line = substr(line, RSTART + RLENGTH)
      }
    }
  ' "$file" | sort -u
}

collect_mapped_sources() {
  local file="$1"

  awk '
    function print_inline_sources(text, rest, entry, parts) {
      rest = text
      while (match(rest, /"[^"]+"[[:space:]]*=/)) {
        entry = substr(rest, RSTART, RLENGTH)
        split(entry, parts, "=")
        gsub(/^[[:space:]]*"/, "", parts[1])
        gsub(/"[[:space:]]*$/, "", parts[1])
        print parts[1]
        rest = substr(rest, RSTART + RLENGTH)
      }
    }

    /^\[/ {
      section = $0
      in_files = (section ~ /^\[[^]]+\.files\]$/)
    }

    in_files && /^[[:space:]]*"/ {
      if (match($0, /"[^"]+"/)) {
        entry = substr($0, RSTART + 1, RLENGTH - 2)
        print entry
      }
      next
    }

    /files[[:space:]]*=[[:space:]]*{/ {
      print_inline_sources($0)
    }

    /^\[files\]$/ {
      in_local_files = 1
      next
    }

    in_local_files && /^\[/ {
      in_local_files = 0
    }

    in_local_files && /^[[:space:]]*"/ {
      if (match($0, /"[^"]+"/)) {
        entry = substr($0, RSTART + 1, RLENGTH - 2)
        print entry
      }
    }
  ' "$file" | sort -u
}

check_source_paths() {
  local manifest="$1"
  local source

  while IFS= read -r source; do
    [[ -n "$source" ]] || continue

    if [[ "$source" = /* ]]; then
      if [[ ! -e "$source" ]]; then
        fail "$manifest references missing absolute source path: $source"
      fi
      continue
    fi

    if [[ ! -e "$DOTFILES_DIR/$source" ]]; then
      fail "$manifest references missing source path: $source"
    fi
  done < <(collect_mapped_sources "$manifest")
}

check_selected_packages_exist() {
  local selected
  local known_packages

  known_packages="$(collect_packages "$GLOBAL_CONFIG")"

  while IFS= read -r selected; do
    [[ -n "$selected" ]] || continue
    if ! grep -Fxq "$selected" <<<"$known_packages"; then
      fail "dotter/local.toml selects unknown package: $selected"
    fi
  done < <(collect_selected_packages "$LOCAL_CONFIG")
}

main() {
  if [[ ! -d "$DOTFILES_DIR" ]]; then
    fail "dotfiles directory not found: $DOTFILES_DIR"
  fi

  if [[ ! -f "$GLOBAL_CONFIG" ]]; then
    fail "Dotter global config not found: $GLOBAL_CONFIG"
  fi

  if [[ ! -f "$LOCAL_CONFIG" ]]; then
    fail "Dotter local config not found: $LOCAL_CONFIG"
    note "Create it with: cp dotter/local.toml.example dotter/local.toml"
  fi

  check_command git
  check_command dotter

  if [[ "$OSTYPE" == darwin* ]]; then
    check_command brew
  fi

  if [[ -f "$GLOBAL_CONFIG" ]]; then
    check_source_paths "$GLOBAL_CONFIG"
  fi

  if [[ -f "$LOCAL_CONFIG" ]]; then
    check_source_paths "$LOCAL_CONFIG"
    check_selected_packages_exist
  fi

  if [[ "$OSTYPE" == darwin* ]] && [[ -f "$ICON_SCRIPT" ]] && ! command -v fileicon >/dev/null 2>&1; then
    warn "fileicon is not installed; macOS app icon overrides will be skipped"
  fi

  # yt-dlp must resolve to the uv build — that is the one carrying curl_cffi, and
  # without it extractors needing impersonation fail obscurely (a dead HLS format
  # is chosen and the download dies with "HTTP Error 474"). `brew upgrade yt-dlp`
  # relinks brew's own copy over the shim and silently reintroduces that.
  #
  # `-ef` (same file, symlinks followed) rather than a string compare: the install
  # deliberately leaves a shim at $(brew --prefix)/bin/yt-dlp, so the resolved path
  # is usually NOT ~/.local/bin/yt-dlp even when everything is correct.
  #
  # Deliberately not `yt-dlp --list-impersonate-targets`: invoking yt-dlp would
  # load the user config and pull cookies from the browser on every deploy.
  # Keyed off what `yt-dlp` actually resolves to, NOT off the uv build existing:
  # gating on ~/.local/bin/yt-dlp would skip the whole check on exactly the
  # machine that needs it most — a fresh one carrying only brew's broken copy.
  # Silent when yt-dlp is absent entirely, since that machine simply isn't using
  # it; a wrong yt-dlp is a problem, a missing one is a choice.
  local resolved_ytdlp
  resolved_ytdlp="$(command -v yt-dlp 2>/dev/null || true)"
  if [[ -n "$resolved_ytdlp" ]]; then
    if [[ ! -x "$HOME/.local/bin/yt-dlp" ]]; then
      warn "yt-dlp is $resolved_ytdlp but the uv build is not installed; run scripts/install/install-yt-dlp.sh"
    elif [[ ! "$resolved_ytdlp" -ef "$HOME/.local/bin/yt-dlp" ]]; then
      warn "yt-dlp resolves to $resolved_ytdlp, not the uv build; run scripts/install/install-yt-dlp.sh"
    else
      # Resolving to the uv build proves which file runs, not that it can
      # impersonate. A plain `uv tool install yt-dlp` (no --with curl_cffi)
      # satisfies every check above and still fails on the sites that need it.
      # An unmatched glob stays literal in bash, so -d is false either way.
      local curl_cffi_dir
      curl_cffi_dir=( "$HOME"/.local/share/uv/tools/yt-dlp/lib/python*/site-packages/curl_cffi )
      if [[ ! -d "${curl_cffi_dir[0]}" ]]; then
        warn "yt-dlp (uv) has no curl_cffi, so impersonation is unavailable; run scripts/install/install-yt-dlp.sh"
      fi
    fi
  fi

  if (( errors > 0 )); then
    note
    note "Bootstrap health check failed with $errors error(s) and $warnings warning(s)."
    exit 1
  fi

  note "Bootstrap health check passed."
  if (( warnings > 0 )); then
    note "Warnings: $warnings"
  fi
}

main "$@"
