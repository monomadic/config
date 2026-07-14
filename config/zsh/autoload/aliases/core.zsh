# ============================================================================
# Environment Variables
# ============================================================================

export DJ_VISUALS_PATH=$ICLOUD_HOME/Movies/Visuals

alias .send-incoming-flac-to-mixed-in-key="mixed-in-key-drop scan"

alias .tidal-download="tiddl download --track-quality=max --video-quality=fhd --path \"$HOME/Music/Mixed In Key Drop\" --scan-path $HOME/Music/Downloads --output \"{item.title} - {item.artist} \({item.bpm}\)\" url "
alias .tidal-music-search="tiddl download --track-quality=max --video-quality=fhd --path $HOME/Music/Downloads --scan-path $HOME/Music/Downloads --output \"{item.title} - {item.artist} \({item.bpm}\)\" --videos=none search"
alias .tidal-video-search="tiddl download --track-quality=max --video-quality=fhd --path $HOME/Music/Downloads --scan-path $HOME/Music/Downloads --output \"{item.title} - {item.artist} \({item.bpm}\)\" --videos=only search"
alias .tidal-dl-video="tiddl download --videos=allow url"

alias flac-print-tags="metaflac --list --block-type=VORBIS_COMMENT"
alias .flac-print-tags=flac-print-tags


alias rtsp-url-brute="nmap --script=/opt/homebrew/share/nmap/scripts/rtsp-url-brute.nse -p554"

gh-repo-create() {
  gh repo create $1 --private --source=. --remote=upstream
}

alias rclone-diff-size="rclone check --size-only"
alias diff-dir="diff -rq "

# ============================================================================
# Utility Functions
# ============================================================================

# Strips trailing / from directories, preserves NUL termination
strip_fd_slash() {
  emulate -L zsh -o noglob
  local p
  while IFS= read -r -d '' p; do
    print -rn -- "${p%/}"$'\0'
  done
}

# Strips trailing / from directories, expects newline-terminated input
strip-slash() {
  emulate -L zsh -o noglob
  local p
  while IFS= read -r p; do
    print -r -- "${p%/}"
  done
}

# # Read NUL-terminated paths, sort by creation time (newest first)
# sort_by_creation_date() {
#   local gstat="/opt/homebrew/opt/coreutils/libexec/gnubin/stat"
#   local file ctime
#   while IFS= read -r -d '' file; do
#     ctime="$("$gstat" -c '%W' -- "$file" 2>/dev/null)"
#     [[ $ctime == "-1" || -z $ctime ]] && ctime="$("$gstat" -c '%Y' -- "$file" 2>/dev/null)"
#     printf '%s\t%s\0' "$ctime" "$file"
#   done | LC_ALL=C sort -z -n -r -k1,1 | perl -0pe 's/^\d+\t//'
# }

lock() {
  ssh "$1" 'open -a ScreenSaverEngine'
}

# check filenames are valid utf-8
validate-filenames-stdin() {
  while IFS= read -r f; do
    local dir="${f:h}" base="${f:t}"
    local newbase=$(printf '%s' "$base" | iconv -f latin-1 -t utf-8 -c)
    [[ "$base" != "$newbase" ]] && mv "$f" "$dir/$newbase"
  done
}

alias fd-media-check-filenames="fd -t f . --exclude '*[{]*' --exclude '*[}]*'"

# check filenames are valid utf-8
check-invalid-filenames-stdin() {
  while IFS= read -r f; do
    local base="${f:t}"
    local cleaned=$(printf '%s' "$base" | iconv -f UTF-8 -t UTF-8 -c)
    [[ "$base" != "$cleaned" ]] && printf '%s\n' "$f"
  done
}

pause() {
  read -sk '?Press any key to continue...'
  echo
}

[[ -r "${ZSH_DOTFILES_DIR:-${DOTFILES_DIR:-$HOME/config}/config/zsh}/bin/lib/topaz-app.zsh" ]] &&
  source "${ZSH_DOTFILES_DIR:-${DOTFILES_DIR:-$HOME/config}/config/zsh}/bin/lib/topaz-app.zsh"

topaz-list-filters() {
  local ffmpeg
  ffmpeg="$(topaz_resolve_ffmpeg)" || return
  [[ -x "$ffmpeg" ]] || {
    print -u2 "Topaz ffmpeg not found or not executable: $ffmpeg"
    topaz_app_help >&2
    return 1
  }
  "$ffmpeg" -hide_banner -filters | rg 'tvai|veai|topaz'
}

topaz-list-models() {
  local resources
  resources="$(topaz_resolve_resources_dir)" || return
  [[ -d "$resources" ]] || {
    print -u2 "Topaz resources dir not found: $resources"
    topaz_app_help >&2
    return 1
  }
  fd -e json . "$resources" \
    | rg '/(apo|rxl|amq|chr|prob|iris|nyx|theia|proteus)-[^/]+\.json$'
}

# ============================================================================
# Queue Management Functions
# ============================================================================

alias q="pueue"
alias q-service-start="brew services start pueue"
alias q-start="/opt/homebrew/opt/pueue/bin/pueued --verbose"
alias q-log="q log"
alias q-add="q add -- "
alias q-status="q status"
alias qs="pueue status"
alias ql=q-status
alias q-url="pueue add yt-url"
alias qu=q-url
alias md-leaf=leaf



# ============================================================================
# File Management Functions
# ============================================================================

rename-extension() {
  local e1=$1 e2=$2 f new
  for f in *.$e1; do
    [[ -e "$f" ]] || continue
    new="${f%.$e1}.$e2"
    if [[ -e "$new" ]]; then
      print -P "%F{yellow}Skipping:%f $f → $new (already exists)"
    else
      mv -- "$f" "$new"
      print -P "%F{green}Renamed:%f  $f → $new"
    fi
  done
}

rename-m4v-to-mp4() {
  rename-extension m4v mp4
}
