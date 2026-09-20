#!/usr/bin/env zsh

typeset -ga INDEX_DEFAULT_IGNORE_PATTERNS=(
  '.DS_Store'
  '._*'
  '.AppleDB'
  '.AppleDesktop'
  '.AppleDouble'
  '.DocumentRevisions-V100'
  '.Spotlight-V100'
  '.TemporaryItems'
  '.Trash-*'
  '.Trashes'
  '.VolumeIcon.icns'
  '.apdisk'
  '.com.apple.timemachine.donotpresent'
  '.fseventsd'
  '.localized'
)

index_default_icloud_home() {
  print -r -- "${ICLOUD_HOME:-$HOME/Library/Mobile Documents/com~apple~CloudDocs}"
}

index_root_dir() {
  if [[ -n "${INDEX_ROOT_DIR:-}" ]]; then
    print -r -- "$INDEX_ROOT_DIR"
  else
    print -r -- "$(index_default_icloud_home)/Sync/Indexes"
  fi
}

index_ensure_root_dir() {
  local dir
  dir="$(index_root_dir)"
  mkdir -p -- "$dir" || return 1
}

index_script_name() {
  print -r -- "${ZSH_ARGZERO:t}"
}

index_die() {
  print -u2 -- "$*"
  exit 1
}

index_log() {
  print -u2 -- "$*"
}

index_parse_value_opt() {
  local opt_name="$1"
  local token="$2"
  local next="${3-}"

  if [[ "$token" == *=* ]]; then
    print -r -- "${token#*=}"
    return 0
  fi

  [[ -n "$next" ]] || {
    print -u2 -- "Missing value for $opt_name"
    return 1
  }

  print -r -- "$next"
  return 0
}

index_volume_name_from_arg() {
  local input="${1%/}"

  [[ "$1" == "/" ]] && {
    print -r -- "root"
    return 0
  }

  if [[ "$input" == /Volumes/* ]]; then
    print -r -- "${input:t}"
  elif [[ "$input" == /* ]]; then
    print -r -- "${input:t}"
  else
    print -r -- "$input"
  fi
}

index_resolve_source_path() {
  local input="$1"
  local candidate=""

  [[ -n "$input" ]] || return 1

  if [[ "$input" == "/" ]]; then
    candidate="/"
  elif [[ "$input" == /Volumes/* || "$input" == /* ]]; then
    candidate="${input%/}"
  else
    candidate="/Volumes/${input%/}"
  fi

  [[ -d "$candidate" ]] || return 1
  print -r -- "$candidate"
}

index_physical_path() {
  local candidate_path="$1"

  (
    builtin cd -- "$candidate_path" >/dev/null 2>&1 &&
    pwd -P
  )
}

index_child_path() {
  local root_path="$1"
  local child_name="$2"

  if [[ "$root_path" == "/" ]]; then
    print -r -- "/$child_name"
  else
    print -r -- "${root_path%/}/$child_name"
  fi
}

index_relative_path() {
  local root_path="$1"
  local candidate_path="$2"
  local prefix=""

  if [[ "$root_path" == "/" ]]; then
    prefix="/"
  else
    prefix="${root_path%/}/"
  fi

  if [[ "$candidate_path" == "$prefix"* ]]; then
    print -r -- "${candidate_path#$prefix}"
  else
    print -r -- "$candidate_path"
  fi
}

index_is_macos_system_volume() {
  local volume_path="$1"
  local system_version_plist=""

  system_version_plist="$(index_child_path "$volume_path" 'System/Library/CoreServices/SystemVersion.plist')"
  [[ -f "$system_version_plist" ]]
}

index_search_roots_for_volume() {
  local volume_path="$1"
  local -a search_roots=()
  local users_root applications_root

  if index_is_macos_system_volume "$volume_path"; then
    users_root="$(index_child_path "$volume_path" 'Users')"
    applications_root="$(index_child_path "$volume_path" 'Applications')"

    [[ -d "$users_root" ]] && search_roots+=("$users_root")
    [[ -d "$applications_root" ]] && search_roots+=("$applications_root")
  fi

  (( ${#search_roots[@]} > 0 )) || search_roots+=("$volume_path")
  printf '%s\0' "${search_roots[@]}"
}

index_file_for_volume() {
  local volume_name="$1"
  print -r -- "$(index_root_dir)/${volume_name}.txt"
}

index_data_stream() {
  sed '1,/^--$/d' "$1"
}

index_is_valid_utf8() {
  printf '%s' "$1" | iconv -f UTF-8 -t UTF-8 >/dev/null 2>&1
}

index_is_safe_text_path() {
  local candidate_path="$1"

  index_is_valid_utf8 "$candidate_path" || return 1

  [[ "$candidate_path" != "${candidate_path//$'\n'/}" ]] && return 1
  [[ "$candidate_path" != "${candidate_path//$'\r'/}" ]] && return 1

  return 0
}

index_sort_value_valid() {
  case "$1" in
    mtime|path|none)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

index_should_ignore_path() {
  local file_path="$1"
  local volume_path="${2-}"
  local ignore_pattern
  local file_name="${file_path:t}"
  local relative_path=""

  for ignore_pattern in "${INDEX_DEFAULT_IGNORE_PATTERNS[@]}"; do
    if [[ "$file_name" == $ignore_pattern || "$file_path" == */$ignore_pattern || "$file_path" == */$ignore_pattern/* ]]; then
      return 0
    fi
  done

  if [[ -n "$volume_path" ]]; then
    relative_path="$(index_relative_path "$volume_path" "$file_path")"
    if [[ "$relative_path" == Users/*/Library || "$relative_path" == Users/*/Library/* ]]; then
      return 0
    fi
  fi

  return 1
}
