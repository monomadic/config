#!/usr/bin/env bash
#
# Inspect and edit the package selection in dotter/local.toml.
#
#   packages.sh list              show every package in global.toml, on or off
#   packages.sh enable <name>...  turn packages on
#   packages.sh disable <name>... comment packages out (kept as a reminder)
#
# Changes take effect on the next `setup/macos/deploy.sh`.

set -euo pipefail

DOTFILES_DIR="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
GLOBAL_CONFIG="$DOTFILES_DIR/dotter/global.toml"
LOCAL_CONFIG="$DOTFILES_DIR/dotter/local.toml"

usage() {
  sed -n '3,9p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

known_packages() {
  awk '
    /^\[/ {
      section = $0
      gsub(/^\[/, "", section)
      gsub(/\]$/, "", section)
      split(section, parts, ".")
      if (parts[1] != "helpers" && parts[1] != "settings") {
        print parts[1]
      }
    }
  ' "$GLOBAL_CONFIG" | sort -u
}

# Prints enabled package names (uncommented entries in the packages array).
enabled_packages() {
  awk '
    /^[[:space:]]*packages[[:space:]]*=[[:space:]]*\[/ { in_packages = 1; next }
    in_packages {
      line = $0
      sub(/#.*/, "", line)
      if ($0 ~ /\]/) in_packages = 0
      while (match(line, /"[^"]+"/)) {
        print substr(line, RSTART + 1, RLENGTH - 2)
        line = substr(line, RSTART + RLENGTH)
      }
    }
  ' "$LOCAL_CONFIG" | sort -u
}

require_local_config() {
  if [[ ! -f "$LOCAL_CONFIG" ]]; then
    echo "Error: $LOCAL_CONFIG does not exist." >&2
    echo "Create it with: cp dotter/local.toml.example dotter/local.toml" >&2
    exit 1
  fi
}

require_known() {
  local name="$1"
  if ! known_packages | grep -Fxq "$name"; then
    echo "Error: no [$name.files] section in dotter/global.toml" >&2
    exit 1
  fi
}

cmd_list() {
  require_local_config

  local enabled name
  enabled="$(enabled_packages)"

  while IFS= read -r name; do
    if grep -Fxq "$name" <<<"$enabled"; then
      printf '  on   %s\n' "$name"
    else
      printf '  off  %s\n' "$name"
    fi
  done < <(known_packages)
}

cmd_enable() {
  require_local_config

  local name
  for name in "$@"; do
    require_known "$name"

    if enabled_packages | grep -Fxq "$name"; then
      echo "already enabled: $name"
      continue
    fi

    # Uncomment an existing disabled entry, or append a new one.
    if grep -Eq "^[[:space:]]*#[[:space:]]*\"$name\"," "$LOCAL_CONFIG"; then
      sed -i '' -E "s|^([[:space:]]*)#[[:space:]]*(\"$name\",)|\1\2|" "$LOCAL_CONFIG"
    else
      sed -i '' -E "s|^(packages = \[)$|\1\n  \"$name\",|" "$LOCAL_CONFIG"
    fi
    echo "enabled: $name"
  done
}

cmd_disable() {
  require_local_config

  local name
  for name in "$@"; do
    if ! enabled_packages | grep -Fxq "$name"; then
      echo "already disabled: $name"
      continue
    fi

    sed -i '' -E "s|^([[:space:]]*)(\"$name\",)|\1# \2|" "$LOCAL_CONFIG"
    echo "disabled: $name"
  done
}

main() {
  local command="${1:-list}"
  shift || true

  case "$command" in
    list)
      cmd_list
      ;;
    enable | disable)
      if (( $# == 0 )); then
        echo "Error: $command needs at least one package name." >&2
        exit 2
      fi
      "cmd_$command" "$@"
      ;;
    -h | --help | help)
      usage
      ;;
    *)
      echo "Error: unknown command: $command" >&2
      usage >&2
      exit 2
      ;;
  esac
}

main "$@"
