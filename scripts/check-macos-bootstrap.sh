#!/usr/bin/env bash

set -euo pipefail

DOTFILES_DIR="${DOTFILES_DIR:-$HOME/config}"
GLOBAL_CONFIG="$DOTFILES_DIR/dotter/global.toml"
LOCAL_CONFIG="$DOTFILES_DIR/dotter/local.toml"
ICON_SCRIPT="$DOTFILES_DIR/scripts/macos-apply-file-icons.sh"

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
    /^\s*packages\s*=\s*\[/ { in_packages = 1; next }
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
      while (match(rest, /"[^"]+"\s*=/)) {
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
