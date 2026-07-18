# ============================================================================
# Media Selection & Playback
# ============================================================================

mpv-compare() {
  if [[ $# -ne 2 || $1 == (-h|--help) ]]; then
    echo "Usage: mpv-ab <file1> <file2>"
    echo ""
    echo "Play file1 and file2 in A/B test mode, comparing the two against each other."
    echo ""
    echo "Arguments:"
    echo "  file1  First file (usually original)"
    echo "  file2  Second file (usually new master)"
    return ${${(M)$#:#0}:+1}  # return 1 if wrong arg count, 0 if --help
  fi

  mpv --profile=ab --external-file="$1" "$2"
}

typeset -ga DJ_VISUALS_PATHS
DJ_VISUALS_PATHS=("$ICLOUD_HOME/Movies/Visuals")

.play-tower-with-mount() {
  MEDIA_PATH="/Volumes/Tower/Movies/Porn"

  if [[ ! -d "/Volumes/Tower" ]]; then
     osascript -e 'tell application "Finder" to mount volume "smb://nom@m4.local/Tower"'
  fi

  if [[ ! -d "$MEDIA_PATH/.index" ]]; then
     fd-media --path "$MEDIA_PATH" --sort created > "$MEDIA_PATH/.index"
  fi

  mpv-send play < "$MEDIA_PATH/.index"
}

fd-tower() {
  fd-media $@ /Volumes/Tower/Movies/Porn/
}

alias media-index-update="fd-media --progress | media-index write"
alias media-index-play="media-index read | mpv-send play"
alias media-index-switchblade="media-index read | switchblade"
alias media-index-search="media-index read | fzf-multiselect"

alias @play="mpv-send play"
alias @select="fzf-select | mpv-send play"
alias .select-and-play="fzf-select --color | mpv-send play"
alias .keyframe-cut="ffmpeg-lossless-cut --force"
alias .batch="yt-dlp-porn-batch-file /Users/nom/Library/Mobile\ Documents/com\~apple\~CloudDocs/Sync/links.txt"

alias .audit-new="fd-media --sort-created | media-audit"
alias .audit-new-pwd="fd-media --sort-created --path . | media-audit"
#
# ALIASES
#
alias .trim="ffmpeg-lossless-cut"
alias .trim-end="ffmpeg-lossless-cut --reverse"

alias .ls="fd-media"
alias .ls-sorted="fd-media --sort=created"
alias .ls-pwd="fd-media . ."
alias .ls-sorted-pwd=".ls-sorted . ."
alias .ls-local="fd-media . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Porn(N)"

# alias .play="fd-media --print0 . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Porn(N) | mpv-send play -0"
alias .play="fd-media --print0 . {${HOME},/Volumes/*}/Movies/Porn(N) | mpv-send play -0"
alias play="fd-media --print0 . . | mpv-send play -0"
alias .new="fd-media --sort created --print0 . . | mpv-send play -0"
alias .new-local="fd-media --sort created --print0 . $HOME/Movies/Porn | mpv-send play -0"

# alias .select="fd-media --print0 . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Porn(N) | fzf-select -0 --print0 --stream | mpv-send play -0"
alias .select="fd-media --print0 . {${HOME},/Volumes/*}/Movies/Porn(N) | fzf-select -0 --print0 --stream | mpv-send play -0"

alias .play-all-sorted=".ls-sorted | mpv-send play"
alias .play-pwd="fd-media --sort-created --print0 . . | mpv-send play -0 && mpv-send sort && mpv-send goto 1"
alias .pwd=.play-pwd
alias .play-local=".ls-local | mpv-send play"
alias .play-tower="fd-media --print0 . /Volumes/Tower/Movies/Porn(N) | mpv-send play -0"
alias .play-tower-clips="fd-media --print0 . /Volumes/Tower/Movies/Porn/Clips(N) | mpv-send play -0"
alias .play-tower-clips-landscape="fd-media --print0 . /Volumes/Tower/Movies/Porn/Clips/Landscape(N) | mpv-send play -0"

.play-tower-downloads-indexed() {
  MOUNT_PATH="/Volumes/Tower"
  MEDIA_PATH="/Movies/Porn/Downloads"
  INDEX_DIR="${MOUNT_PATH}${MEDIA_PATH}"
  SMB_URL="smb://nom@m4.local/Tower"

  if [[ ! -d "$MOUNT_PATH" ]]; then
    echo "Mounting $SMB_URL..."
    osascript -e 'tell application "Finder" to mount volume "$SMB_URL"'
  fi

  echo "$SMB_URL mounted."

  if [[ ! -d "$INDEX_DIR/.index" ]]; then
    echo "Index not found at ${INDEX_DIR}/.index"
    echo "Creating new index..."
    fd-media --path "${INDEX_DIR}" --sort created > "${INDEX_DIR}/.index"
    create-index /Volumes/Tower/Movies/Porn/Downloads
  fi

  echo "Using index at $INDEX_DIR/.index"

  mpv-send play < "$INDEX_DIR/.index"
}

filter-icloud-files() {  
  while IFS= read -r -d '' f; do
    status="$(mdls -raw -name kMDItemDownloadedDate "$f" 2>/dev/null || true)"
    [[ "$status" != "(null)" && -n "$status" ]] && print -r -- "$f"
  done
}

alias .select-all-sorted=".ls-sorted | fzf-select | mpv-send play"
alias .select-pwd=".ls-pwd | fzf-select | mpv-send play"
alias .select-pwd-sorted=".ls-sorted-pwd | fzf-select | mpv-send play"

alias \%select="cat $HOME/.indexes/Tower | fzf-select | mpv-send play"

alias .play-downloads="fd-media --print0 . {/Users/nom/Library/Mobile\ Documents/com~apple~CloudDocs,/Users/nom,/Volumes/*}/Movies/Porn/Downloads(N) | mpv-send play -0"
alias .play-local-downloads="fd-media --print0 . ~/Movies/Porn/Downloads | mpv-send play -0"
.play-newest-local-downloads() {
  emulate -L zsh
  setopt extended_glob
  printf '%s\n' ~/Movies/Porn/Downloads/**/*.part~*Frag*(N.om) | mpv-send play
}
alias .play-local-incomplete-downloads="fd -t f -g '*.part' -E '*Frag*.part' . ~/Movies/Porn/Downloads | mpv-send play"

alias .select-local="fd-media . $LOCAL_MEDIA_PATHS | fzf-select | mpv-send play"

alias .select-downloads="fd-media . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"
alias .select-local-downloads-sort="fd-media --sort created . $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"
alias .select-downloads-sort="fd-media --sort created . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"
alias .select-local-downloads="fd-media . $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"

alias .play-suki="fd-media --match-string #suki | mpv-send play"
alias \#suki=.play-suki
alias .select-suki="fd-media --match-string #suki | fzf-select | mpv-send play"
alias .play-60fps="fd-media --match-string 60fps | mpv-send play"
alias \#60fps=.play-60fps
alias .play-4k60fps="fd-media --match-string 60fps --match-string 2160p | mpv-send play"
alias .play-4k60fps-top="fd-media --match-string 60fps --match-string 2160p --match-string ★★★ | mpv-send play"
alias .select-4k60fps-top="fd-media --match-string 60fps --match-string 2160p --match-string ★★★ | fzf-select | mpv-send play"
alias .play-best="fd-media --match-string ★★★ | mpv-send play"
alias \#★★★="fd-media --match-string ★★★ | mpv-send play"
alias \#★★★★★="fd-media --match-string ★★★★★ | mpv-send play"
alias .select-best="fd-media --match-string ★★★ | fzf-select | mpv-send play"

alias \$=.select
alias \$.=.select-pwd

alias .play-clips="fd-media --match-string /Clips/ | mpv-send play"
alias .select-clips="fd-media --match-string /Clips/ | fzf-select | mpv-send play"

alias .select-visuals="fd-media . {${ICLOUD_HOME},${HOME},/Volumes/*}/Movies/Visuals(N) | fzf-select | mpv-send play"

alias .play-visuals="fd-media --print0 . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Visuals(N) | mpv-vj play -0"
alias .play-visuals-bg-black="fd-media . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Visuals(N) | grep '#bg-black' | mpv-vj play"
alias \#bg-black=".play-visuals-bg-black"

alias .select-external="fd-media . /Volumes/*/Movies/Porn(N) | fzf-select | mpv-send play"
alias .select-masters="fd-media . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | fzf-select | mpv-send play"
alias .play-masters="fd-media . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | mpv-send play"

alias .select-tower-downloads-queue="fd-media --path /Volumes/Tower/Movies/Porn/Downloads/_queue | fzf-select | mpv-send play"
alias .play-tower-downloads-60fps="fd-media --match-string 60fps --path /Volumes/Tower/Movies/Porn/Downloads | mpv-send play"
alias .play-tower-downloads-2160p="fd-media --match-string 2160p --path /Volumes/Tower/Movies/Porn/Downloads | mpv-send play"
alias .play-tower-masters="fd-media --path=/Volumes/Tower/Movies/Porn/Masters | mpv-send play"
alias .play-tower-masters-sorted="fd-media --path=/Volumes/Tower/Movies/Porn/Masters --sort created | mpv-send play"
alias .play-tower-downloads="fd-media . /Volumes/Tower/Movies/Porn/Downloads | mpv-send play"
alias .local-sorted="fd-media --sort created . $LOCAL_MEDIA_PATHS | fzf-select | mpv-send play"
alias .play-local-sorted="fd-media --sort created . $LOCAL_MEDIA_PATHS | fzf-select | mpv-send play"

# Media search shortcuts
alias @=".play"
alias @@=".play-all-sorted"
@@@() {
  emulate -L zsh
  setopt null_glob
  printf '%s\0' $~ADULT_GLOBS | fzf-select -0 --print0 --color | mpv-send play -0
}
alias @unc="fd-media . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-send play"
alias @towerlocal="fd-media . /Volumes/Tower/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-send play"
alias @unique='fd-media . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | awk -F/ '"'"'!seen[$NF]++'"'"' | mpv-send play'
alias @full-path="fd-media . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-send play"
alias @clips="fd --absolute-path --exact-depth=1 --color=never . /Volumes/*/Movies/Porn/Masters/Clips/*/(N) $HOME/Movies/Porn/Masters/Clips/*/(N) | mpv-send play"
alias @pwd="fd-media . . | mpv-send play"
alias @@@pwd="fd-media --absolute-path --print0 | fzf-select -0 --print0 | mpv-send play -0"
alias @loop="fselect-porn -0 | fzf-select -0 --print0 --tac | mpv-vj play -0 --shuffle"
alias @pwd-sort="fselect-pwd-sort -0 | fzf-play --hide-path --tac"
alias @queue="fd-media --print0 . $HOME/Movies/Porn/Queue/(N) | fzf-select -0 --print0 | mpv-send play -0"
alias @tutorials="fd-media . $TUTORIALS_PATH | fzf-select | mpv-send play"

alias mount-tower="open -j smb://nom@m4.local/Tower"
alias .mount-tower=mount-tower
alias unmount-tower="diskutil unmount /Volumes/Tower"

# list all unique tags found in files under the present directory
fd-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

fd-creators() {
  setopt +o nomatch
  fd -td -d1 . {/Volumes/*/Movies/Porn/*/creators,$HOME/Movies/Porn/*/creators} 2>/dev/null
}

fzf-creators() {
  fd-creators | while read -r path; do
    name=${path%/}   # Remove trailing slash
    name=${name##*/} # Get last component
    echo "$name	$path"
  done | fzf --exit-0 --with-nth=1 --delimiter="\t" --preview="ls '{2}'" | cut -f2
}

cd-creators() {
  cd $(fzf-creators)
}

media-cache-clear() {
  cd $LOCAL_CACHE_PATH && rm -rf **/*
}

# ============================================================================
# MPV / Media / FFmpeg (re-homed from alias.zsh)
# ============================================================================
alias mpv-vj='mpv-send --socket /tmp/mpv-vj.sock --profile vj'
alias mpv-vj-start='mpv-vj start -- --image-display-duration=inf splash.png'

alias .hevc="ffmpeg-convert-to-hevc"
# ============================================================================
# Media Discovery Functions
# ============================================================================

fd-video-color() {
  { fd -e mp4 $1 } | sd '\]\[' '] [' | sd '\[([^\]]+)\]' $'\e[32m''$1'$'\e[0m' | sd '\{([^}]*)\}' $'\e[33m''$1'$'\e[0m' | sd '(^|/)\(([^)]*)\)' '${1}'$'\e[36m''$2'$'\e[0m' | rg --passthru --color=always -N -r '$0' -e '#\S+' --colors 'match:fg:magenta'
}

fd-visuals() {
  local print0=false
  if [[ ${1:-} == "-0" || ${1:-} == "--print0" ]]; then
    print0=true
    shift
  fi

  local query=${1:-.}
  local -a roots
  [[ -n $DJ_VISUALS_PATH ]] && roots+=($DJ_VISUALS_PATH)
  roots+=($HOME/Movies/Visuals(N) /Volumes/*/Movies/Visuals(N))

  if $print0; then
    fd-media --print0 -- "$query" "${roots[@]}"
  else
    fd-media -- "$query" "${roots[@]}"
  fi
}

select-visuals() {
  local query=$1
  fd-visuals "$query" | fzf-select | mpv-vj play --shuffle
}

# ============================================================================
# MPV Functions
# ============================================================================

mpv-play-visuals() {
  local query=$1
  local -a files
  while IFS= read -r -d '' f; do files+=("$f"); done < <(fd-visuals -0 "$query")

  (( ${#files} )) || { print -r -- "no visuals found"; return 1 }

  mpv-vj play --shuffle -- "${files[@]}"
}

mpv-select-all-v2() {
  kitty-exec "  fd-media" "#A442F3" --shell "fd-media | fzf-select | mpv-send play"
}

kitty-mpv-tab() {
  kitty @ launch --type=tab --cwd=current env PATH="$PATH" kitty-exec "  all" "#A442F3" "$@"
}

mpv-select-queue() {
  kitty @ set-tab-title "mpv:queue"
  kitty @ set-tab-color --match title:"mpv" active_bg="#A442F3" active_fg="#050F63" inactive_fg="#A442F3" inactive_bg="#030D43"
  fd-media | fzf-select | mpv-send play
}
# ============================================================================
# FFmpeg/FFprobe Functions
# ============================================================================

vp9-repack-to-webm() {
  local count=0
  for file in *.mp4(N); do
    local codec=$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$file" 2>/dev/null)
    if [[ $codec == vp9 ]]; then
      print "Repacking: $file"
      mv "$file" "${file:r}.webm"
      (( count++ ))
    fi
  done
  print "Repacked $count file(s)"
}
alias .repack=vp9-repack-to-webm

ffprobe-tags-as-json() {
  local file="$1"
  ffprobe -v quiet -show_entries format_tags -of json $file
}

ffmpeg-tags-write-artist() {
  local artist="$1" input="$2" output="$3"
  ffmpeg -i $input -c copy -metadata artist="$artist" $output
}

mp4-check-faststart() {
  if xxd "$1" | head -n 640 | grep -q moov; then
    echo -e "\e[32m✓ FastStart enabled\e[0m"
    return 0
  else
    echo -e "\e[31m✗ FastStart NOT enabled (moov at end)\e[0m"
    return 1
  fi
}

mp4-enable-faststart() {
  local file="$1"
  
  if xxd "$file" | head -n 640 | grep -q moov; then
    echo -e "\e[32m✓ FastStart already enabled\e[0m"
    return 0
  fi
  
  echo -e "\e[31m✗ FastStart not enabled\e[0m"
  read -p "Enable faststart for $file? (y/N): " confirm
  
  if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
    local temp="${file%.*}_faststart.${file##*.}"
    echo "Processing..."
    
    if ffmpeg -i "$file" -c copy -movflags +faststart "$temp" -y 2>/dev/null; then
      mv "$temp" "$file"
      echo -e "\e[32m✓ FastStart enabled successfully\e[0m"
    else
      echo -e "\e[31m✗ Failed to enable faststart\e[0m"
      rm -f "$temp"
      return 1
    fi
  else
    echo "Cancelled"
    return 1
  fi
}
# ============================================================================
# Media Player Aliases
# ============================================================================

alias mp="mpv-send play"

alias mpv-with-config="mpv --profile=fast --video-sync=display-resample --hwdec=auto-safe --shuffle --no-native-fs --macos-fs-animation-duration=0 --mute"
alias mpv-without-config="mpv --profile=fast --video-sync=display-resample --hwdec=auto-safe --no-config --shuffle --no-native-fs --macos-fs-animation-duration=0 --mute"
alias mpv-auto-safe="mpv --hwdec=auto-safe --vo=libmpv"
alias mpv-fs="mpv --macos-fs-animation-duration=0 --no-native-fs --fs"
alias mpv-debug="mpv --msg-level=all=debug"
alias mpv-verbose="mpv --msg-level=all=v"
alias mpv-image-viewer='mpv-stdin --image-display-duration=inf'
alias mpv-image-slideshow='mpv-stdin --image-display-duration=5'

mpv-focus() {
  osascript -e 'tell application "System Events" to set frontmost of process "mpv" to true'
}

alias iina-shuffle="iina --mpv-shuffle --mpv-loop-playlist"


mpv-play-porn() {
  emulate -L zsh
  setopt null_glob
  mpv-send play $~ADULT_GLOBS
}
alias mpv-play-volumes="fd-media --print0 . /Volumes/*/Movies/Porn(N) | mpv-send play -0"
alias mpv-play-tower="fd-media --print0 . /Volumes/Tower/Movies/Porn | mpv-send play -0"
alias .tower=mpv-play-tower

# ============================================================================
# Media & Image Tools
# ============================================================================

alias img="chafa --format=symbols"
alias sixel="chafa --clear --format=symbol --center=on --scale=max"
alias sixel-sixel="chafa --clear --format=sixel --center=on --scale=max"
alias sixel-kitty="chafa --clear --format=kitty --center=on --scale=max"
alias v="viu --height 20"

alias sips-to-webp-lossy='sips -s format webp -s formatOptions 75'
# ============================================================================
# FFmpeg Shortcuts
# ============================================================================

alias .demux="ffmpeg-demux"
alias .demux-video="ffmpeg-demux --video"
alias .demux-audio="ffmpeg-demux --audio"
