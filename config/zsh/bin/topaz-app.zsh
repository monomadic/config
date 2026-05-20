topaz_known_apps() {
  emulate -L zsh
  print -r -- "/Applications/Topaz Video.app"
  print -r -- "/Applications/Topaz Video AI.app"
}

topaz_app_help() {
  emulate -L zsh
  print -r -- "Set TOPAZ_APP to override. Expected one of:"
  topaz_known_apps | sed 's/^/  /'
}

topaz_resolve_app() {
  emulate -L zsh
  local app
  local -a candidates

  if [[ -n "${TOPAZ_APP:-}" ]]; then
    app="${TOPAZ_APP%/}"
    if [[ -d "$app" ]]; then
      print -r -- "$app"
      return 0
    fi

    print -u2 -- "Topaz app not found from TOPAZ_APP: $TOPAZ_APP"
    topaz_app_help >&2
    return 1
  fi

  candidates=("${(@f)$(topaz_known_apps)}")
  for app in "${candidates[@]}"; do
    [[ -d "$app" ]] || continue
    print -r -- "$app"
    return 0
  done

  print -u2 -- "Topaz app not found."
  topaz_app_help >&2
  return 1
}

topaz_resolve_macos_dir() {
  emulate -L zsh
  local app="${1:-}"
  [[ -n "$app" ]] || app="$(topaz_resolve_app)" || return
  app="${app%/}"
  print -r -- "$app/Contents/MacOS"
}

topaz_resolve_ffmpeg() {
  emulate -L zsh
  local app="${1:-}"
  [[ -n "$app" ]] || app="$(topaz_resolve_app)" || return
  app="${app%/}"
  print -r -- "$app/Contents/MacOS/ffmpeg"
}

topaz_resolve_resources_dir() {
  emulate -L zsh
  local app="${1:-}"
  [[ -n "$app" ]] || app="$(topaz_resolve_app)" || return
  app="${app%/}"
  print -r -- "$app/Contents/Resources"
}

topaz_resolve_models_dir() {
  emulate -L zsh
  local app="${1:-}"
  [[ -n "$app" ]] || app="$(topaz_resolve_app)" || return
  app="${app%/}"
  print -r -- "$app/Contents/Resources/models"
}

topaz_resolve_gui_binary() {
  emulate -L zsh
  local app="${1:-}"
  [[ -n "$app" ]] || app="$(topaz_resolve_app)" || return
  app="${app%/}"
  local name="${app:t:r}"
  print -r -- "$app/Contents/MacOS/$name"
}
