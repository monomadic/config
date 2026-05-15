# ============================================================================
# Environment Variables
# ============================================================================

export DJ_VISUALS_PATH=$ICLOUD_HOME/Movies/Visuals

alias .send-incoming-flac-to-mixed-in-key="open -a /Applications/Mixed\ In\ Key\ 11.app ~/Music/Mixed\ In\ Key/**/*.flac"

alias .tidal-download="tiddl download --track-quality=max --video-quality=fhd --path $HOME/Music/Downloads --scan-path $HOME/Music/Downloads --output \"{item.title} - {item.artist} \({item.bpm}\)\" url "
alias .tidal-music-search="tiddl download --track-quality=max --video-quality=fhd --path $HOME/Music/Downloads --scan-path $HOME/Music/Downloads --output \"{item.title} - {item.artist} \({item.bpm}\)\" --videos=none search"
alias .tidal-video-search="tiddl download --track-quality=max --video-quality=fhd --path $HOME/Music/Downloads --scan-path $HOME/Music/Downloads --output \"{item.title} - {item.artist} \({item.bpm}\)\" --videos=only search"

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

topaz-list-filters() {
  "/Applications/Topaz Video.app/Contents/MacOS/ffmpeg" -hide_banner -filters | rg 'tvai|veai|topaz'
}

topaz-list-models() {  
  fd -e json . "/Applications/Topaz Video.app/Contents/Resources" \
    | rg '/(apo|rxl|amq|chr|prob|iris|nyx|theia|proteus)-[^/]+\.json$'
}

topaz-ai-list-models() {  
  fd -e json . "/Applications/Topaz Video AI.app/Contents/Resources" \
    | rg '/(apo|rxl|amq|chr|prob|iris|nyx|theia|proteus)-[^/]+\.json$'
}

topaz-ai-list-filters() {
  "/Applications/Topaz Video AI.app/Contents/MacOS/ffmpeg" -hide_banner -filters | rg 'tvai|veai|topaz'
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
alias yt-dlp-ignore-archive="yt-dlp --no-download-archive"
alias yt-list-fields="yt-dlp --skip-download --print \"%()#j\" "
alias yt-batch-edit='$EDITOR $YT_DLP_BATCH_FILE'

alias mpv-vj='mpv-send --socket /tmp/mpv-vj.sock --profile vj'
alias mpv-vj-start='mpv-vj start -- --image-display-duration=inf splash.png'

alias .hevc="ffmpeg-convert-to-hevc"

yt-dlp-list-formats() {
  (( $# )) || { print -u2 "usage: yt-dlp-list-formats <yt-dlp args>"; return 2; }
  yt-dlp --no-download-archive --list-formats "$@"
}

yt-dlp-porn-batch-file() {
  (( $# )) || { print -u2 "usage: dl-porn-batch-file <batch_file> ...<yt-dlp args>"; return 2; }
  local batch_file=$1; shift
  yt-dlp --porn --batch-file="$batch_file" "$@"
}

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
# YT-DLP
# ============================================================================

yt-retag() {
  emulate -L zsh; set -euo pipefail
  local url="$1" src="$2" tpl="${3:-'%(title)s (%(extractor)s).%(ext)s'}"
  local dir="${src:A:h}" ext="${src:A:e}"

  local target
  target="$(yt-dlp --print-name --alias porn "$tpl" "$url")"
  target="${target%%$'\n'*}"
  [[ "$target" = /* ]] || target="$dir/$target"

  # keep existing ext by default
  target="${target%.*}.$ext"
  mv -- "${src:A}" "$target"
  print -r -- "$target"
}

yt-debug-extract() {
  yt-dlp -j -v "$@" \
  | jq '{title, fulltitle, description, channel, uploader, creator, creators, cast, tags, categories}'
}

yt-queue-add() {
  print -r -- "$@" >> ~/.yt-dlp-queue
}

yt-queue-run() {
  yt-dlp \
    -a ~/.yt-dlp-queue \
    --download-archive ~/.yt-dlp-archive \
    -P ~/Movies/Porn/Downloads \
    --ignore-errors \
    --newline \
    -R 20 --fragment-retries 20 --retry-sleep 2 \
    --concurrent-fragments 4
}

yt-queue-run-ext() {
  yt-dlp \
    -a ~/.yt-dlp-queue \
    --download-archive ~/.yt-dlp-archive \
    -P "/Volumes/Footage 1tb/" \
    --ignore-errors \
    --newline \
    -R 20 --fragment-retries 20 --retry-sleep 2 \
    --concurrent-fragments 4
}

alias .qa=yt-queue-add
alias .qr=yt-queue-run-ext

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
# Audio Stem Separation
# ============================================================================

stem-mdx23() {
  local input_file="$1"
  local basename=$(basename "$input_file")
  local output_dir="$HOME/Music/Stems/$basename"
  
  mkdir -p "$output_dir"
  
  cd "$HOME/Music/Stems/MVSEP-MDX23-Colab_v2.1" || { print -u2 "missing MVSEP-MDX23 dir"; return 1; }
  source .venv/bin/activate &&
    time python inference_2.2_b1.5.1_voc_ft.py \
      --input_audio "$input_file" \
      --output_folder "$output_dir" \
      --large_gpu \
      --chunk_size 500000

  cd "$output_dir" || return 1
  
  # Rename files
  mv *vocals.wav vocals.wav 2>/dev/null
  mv *drums.wav drums.wav 2>/dev/null
  mv *bass.wav bass.wav 2>/dev/null
  mv *other.wav other.wav 2>/dev/null
  mv *instrum.wav instrumental.wav 2>/dev/null
  rm -f *instrum2.wav
}

vdjstems-check-wav-lengths() {
  for f in kick.wav other.wav vocals.wav bass.wav hihat.wav mixed.wav; do
    dur=$(ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$f")
    printf "%s: %s\n" "$f" "$dur"
  done
}

# ============================================================================
# Download & Media Aliases
# ============================================================================

alias yt-dlp-youtube-embedded="yt-dlp --cookies-from-browser brave --continue --progress --verbose --retries infinite --fragment-retries infinite --socket-timeout 15 -f bestvideo+ba/best --embed-metadata --extractor-args 'youtube:player-client=tv_embedded'"
alias yt-dlp-tui="auto-ytdlp --concurrent 2 --download-dir $HOME/Movies/Porn/Downloads --archive-file $ICLOUD_HOME/Movies/Porn/archive.txt"
alias dp="dl-porn"

alias d=download-video
alias dmv="download-video music-video"
alias dl-youtube="download-video youtube"
alias dlu="download-video-url"
alias faphouse="download-video-faphouse"
alias dl-beatport=beatportdl-darwin-arm64
alias dl-apple-music=apple-music-dl

alias url="yt-url"

alias N_m3u8DL-RE="/Users/nom/config/bin/N_m3u8DL-RE_v0.5.1-beta_osx-arm64_20251029"
alias .dl-N_m3u8DL-RE=N_m3u8DL-RE

alias .network-quality="networkQuality -v"

# ============================================================================
# Media Player Aliases
# ============================================================================

alias configure-mpv="cd $DOTFILES_DIR/config/mpv && kitty-exec '   mpv.conf ' '#A442F3' hx ."
alias configure-helix="cd $DOTFILES_DIR/config/helix && $EDITOR ."

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


alias mpv-play-porn="setopt local_options null_glob && mpv-send play $~MEDIA_GLOBS"
alias mpv-play-volumes="fd-media --print0 . /Volumes/*/Movies/Porn(N) | mpv-send play -0"
alias mpv-play-tower="fd-media --print0 . /Volumes/Tower/Movies/Porn | mpv-send play -0"
alias .tower=mpv-play-tower


# ============================================================================
# Stem Separation Aliases
# ============================================================================

alias .stem-split="demucs -d mps -n htdemucs --flac -o stems_output"
alias .stem-split-vocals="demucs -d mps -n htdemucs --flac -o stems_output --two-stems=vocals"
alias vdjstems-split-mdx23=stem-mdx23

# ============================================================================
# File Operations
# ============================================================================

alias rm="rm -i"
alias df="df -h"

alias rsync-copy='rsync -a --ignore-existing --progress'
alias cp-skip=rsync-copy
alias backup-tower="rsync-backup --delete /Volumes/Tower/ /Volumes/Tower\ Backup"

alias tag=rename-media
alias .tag=tag
alias rn="batch-rename"
alias ren="batch-rename"
alias .rename="fd-rename-all.zsh"

alias trash-undo="rip --unbury"
alias trash-view="rip --seance"

alias .dupes-check="fdupes --recurse --cache --nohidden --size ."
alias .dupes-delete="fdupes --recurse --cache --nohidden --size --delete ."
alias .dupes-delete-interactive="fdupes --recurse --deferconfirmation --cache --nohidden --size --plain ."
alias .list-moved-files="fclones group --cache --hash-fn metro --isolate --dry-run"

# ============================================================================
# Python/Development
# ============================================================================

alias .python-venv-create="python3 -m venv .venv && source .venv/bin/activate"
alias .python-venv-activate="source .venv/bin/activate"
alias .python-pip-install-requirements="pip install -r requirements.txt"

# ============================================================================
# Kitty Terminal
# ============================================================================

alias .kitty-mark-current-tab-orange="kitty @ set-tab-color active_bg=orange active_fg=white inactive_bg=orange inactive_fg=black"
alias .kitty-mark-current-tab-red="kitty @ set-tab-color inactive_bg=red inactive_fg=black"
alias .kitty-set-tab-color-orange="kitty @ set-tab-color --match id:$KITTY_WINDOW_ID active_bg=#FFA500 active_fg=#050F63 inactive_fg=#FFA500 inactive_bg=#030D43"
alias .kitty-set-tab-color-green="kitty @ set-tab-color --match id:$KITTY_WINDOW_ID active_bg=#38F273 active_fg=#050F63 inactive_fg=#38F273 inactive_bg=#030D43"
alias .kitty-reload="kitty @ set-colors --all ~/.config/kitty/kitty.conf"
alias .kitty-configure="e-kitty"
alias .kitty-kill-all-editor="kitten @ close-tab --match 'env:PROC=hx'"
alias .kitty-kill-all-nvim=.kitty-kill-all-editor
alias .nvim-kill-all=.kitty-kill-all-editor
alias .kitty-close-idle-tabs="kitty @ close-tab --match 'env:PROC=zsh'"

alias kitty-joshuto="kitty --override background=#000 --working-directory=$HOME/workspaces --single-instance joshuto"

# ============================================================================
# macOS Specific
# ============================================================================

alias .restart-window-server="sudo killall -HUP WindowServer"
alias .macos-keybindings="source $DOTFILES_DIR/setup/macos/keybindings.sh"
alias .gatekeeper-whitelist="xattr -rd com.apple.quarantine"
alias .self-sign="codesign --sign - --force --deep"
get-app-id() { osascript -e "id of app \"$1\""; }
alias .screen-sharing-kick-users="sudo /System/Library/CoreServices/RemoteManagement/ARDAgent.app/Contents/Resources/kickstart -deactivate -restart -users current"
alias passwordless-reboot="sudo fdesetup authrestart"
alias .clear-notifications="killall NotificationCenter"
alias battery='pmset -g batt'

alias topaz-video="env LC_ALL=C LC_NUMERIC=C LANG=C /Applications/Topaz\ Video\ AI.app/Contents/MacOS/Topaz\ Video\ AI"
alias .topaz-video=topaz-video
alias .brave-mp4-support="/Applications/Brave\ Browser.app/Contents/MacOS/Brave\ Browser --disable-features=MediaSource,UseModernMediaControls"

alias xdg-open=open
alias o=open
alias tab="open"

# ============================================================================
# Network & System
# ============================================================================

alias .network-detect-captive-portal=detect-captive-portal
alias .network-status=ns
alias .portal=detect-captive-portal
alias .detect-captive-portal=detect-captive-portal
alias ns="network-status.zsh"
alias .uptime="display-uptime"

alias p8="ping 8.8.8.8"
alias pc="ping cloudflare.com"
alias pg="ping google.com"

alias ls-usb="system_profiler SPUSBDataType"
alias ls-usb-ioreg="ioreg -p IOUSB -w0"
alias ls-disks="diskutil list"

# ============================================================================
# Configuration Editing
# ============================================================================

alias c=e-zsh
alias C=e-config
alias .config=e-zsh
alias .config-zsh=e-zsh
alias .config-aliases=.config-env
alias .config-bin="cd $DOTFILES_DIR/bin && $EDITOR ."
alias .config-env="cd $ZSH_DOTFILES_DIR && $EDITOR scripts/autoload/alias.zsh"
alias config-dotfiles="cd $DOTFILES_DIR && fd --type directory --max-depth=2 | fzf | xargs $EDITOR"

alias e-homebrew="cd $DOTFILES_DIR && $EDITOR Brewfile"
alias .brewfile="cd $DOTFILES_DIR && e Brewfile"
alias e-kitty="cd $DOTFILES_DIR/config/kitty && $EDITOR kitty.conf"
alias e-neovim="cd $DOTFILES_DIR/config/neovim && $EDITOR init.lua"
alias e-open="cd $DOTFILES_DIR && $EDITOR README.md"
alias e-yazi="cd $DOTFILES_DIR/config/yazi && $EDITOR yazi.toml"
alias .yazi-config="cd $DOTFILES_DIR/config/yazi && $EDITOR yazi.toml"
alias e-zellij="cd $DOTFILES_DIR/config/zellij && $EDITOR config.kdl"
alias e-zsh-keybindings="cd $DOTFILES_DIR/config/zsh && $EDITOR autoload/keybindings.zsh"
alias e-zsh="cd $DOTFILES_DIR && $EDITOR config/zsh/zshrc.zsh"
alias zsh-config="cd $DOTFILES_DIR/config/zsh/ && $EDITOR zshrc.zsh"
alias zsh-reload="source ~/.zshrc"

alias .fonts="kitty list-fonts"

# ============================================================================
# Git Shortcuts
# ============================================================================

alias g=git
alias ga="git add . && git commit --amend --no-edit"
alias gca="git add . && git commit --amend"
alias gc-update="gc update:"
alias gd="git diff"
alias gl="fzf-git-log"
alias gp="git push"
alias push="git push"
alias pull="git pull"
alias gs="git status --short"
alias gss="git status --short --untracked-files=all"
alias gb="git branch "$@" --sort=-committerdate --sort=-HEAD --format=$'%(HEAD) %(color:yellow)%(refname:short) %(color:green)(%(committerdate:relative))\t%(color:blue)%(subject)%(color:reset)' --color=always | column -ts$'\t'"
alias git-stage-last-commit="git reset --soft HEAD~"
alias branch="b"
alias lg=lazygit

# ============================================================================
# Rust/Cargo
# ============================================================================

alias cb="cargo build"
alias cc="cargo check"
alias ci="cargo install --path ."
alias cr="cargo run"
alias crr="cargo run --release"
alias ct="cargo test"
alias doc="cargo doc --open"
alias loc=tokei

# Rust docs
alias docs-bevy-cheat="open https://bevy-cheatbook.github.io/"
alias docs-rs-yew="open https://docs.rs/yew/latest/yew/"
alias docs-rustdoc="open https://doc.rust-lang.org/rustdoc/"
alias docs-rustup-cargo="rustup doc --cargo"
alias docs-rustup-core="rustup doc --core"
alias docs-wasmtime="open https://docs.wasmtime.dev/"
alias docs-yew="open https://yew.rs/docs/next/"

# ============================================================================
# File Listing & Navigation
# ============================================================================

#alias l="eza --icons --group-directories-first"
alias l="lsd --group-directories-first"
# alias la="eza --icons --group-directories-first --all"
alias lh="eza --icons --group-directories-first --all"
alias ll="echo && lsd --icon always --long --depth 1 --blocks name --group-directories-first --color always && echo"
alias lla="echo && eza --icons --group-directories-first --all --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lll="lsd --icon always --long --depth 1 --group-directories-first --color always"
alias lln="eza --icons --all -l --sort=date"
alias ll-fzf="eza --icons --color=always --group-directories-first --no-permissions --no-user -l --ignore-glob '.DS_Store' | fzf --ansi"

# 10 most-recent files (newest first by default)
# --reverse => newest at bottom
# recent
# recent ~/Downloads 20
# recent --reverse
# recent ~/Downloads 15 --reverse
# 
recent() {
  local dir="."
  local n=10
  local reverse=false

  for arg in "$@"; do
    case "$arg" in
      --reverse) reverse=true ;;
      *) 
        [[ -z "${dir_set:-}" ]] && { dir="$arg"; dir_set=1; } \
        || n="$arg"
        ;;
    esac
  done

  if command -v eza >/dev/null 2>&1; then
    local sort_flags=(--sort=modified)
    $reverse && sort_flags+=(--reverse)

    eza --icons --color=always \
      --only-files \
      "${sort_flags[@]}" \
      --long --time-style=relative \
      --no-permissions --no-user \
      -- "$dir" | head -n "$n"
  else
    # macOS BSD ls fallback
    if $reverse; then
      command ls -lt -- "$dir" | head -n $((n + 1)) | tail -n "$n"
    else
      command ls -lt -- "$dir" | head -n $((n + 1))
    fi
  fi
}
# quick alias for the common case
alias lr='recent . 10 --reverse'

alias up="cd .."
alias gr="cd /"
cd-relative() { cd "$(fd -t directory | fzf-cd)"; }
alias fd-dirs="fd -t d -d 15 -E '.*' -E 'Library'"
alias fd-empty="fd --type empty"

alias src="cd ~/src && l"
alias workspaces="cd ~/workspaces && l"
alias org='cd ~/org && e index.md'
alias wiki="cd ~/wiki && e index.md"
alias w=wiki
alias snippets="cd ~/config/config/neovim/snippets/ && ll"

# ============================================================================
# Editor & Tools
# ============================================================================

alias vi=hx
alias vim=hx
alias edit=$EDITOR
alias n="fzf-edit"
alias eb="edit-bin"
alias es="edit-script"

alias f="noglob fetch"
alias sb="fzf-scrollback"
alias prev="fzf --layout=reverse --preview 'bat --style=numbers --color=always --line-range :500 {}'"

alias .tab=fzf-tablature
alias t=fzf-tablature

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

# ============================================================================
# Music Download
# ============================================================================

alias .apple-music=apple-music-dl
alias .beatport="beatportdl-darwin-arm64"
alias .tidal="noglob tidal-dl-ng dl"
alias .faphouse="download-video-faphouse"

# ============================================================================
# Misc SSH & Remote
# ============================================================================

alias .ssh-m4="kitty kitten ssh nom@m4.local"
alias .ssh-m3="kitty kitten ssh nom@m3.local"

# ============================================================================
# Dotter (dotfile manager)
# ============================================================================

alias dd='dotter-deploy'
alias dd-force='cd ~/config/ && dotter --force --cache-directory ~/.config/dotter/cache --cache-file ~/.config/dotter/cache.toml deploy --global-config dotter/global.toml --local-config dotter/local.toml'
alias dw='cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache --cache-file ~/.config/dotter/cache.toml watch --global-config dotter/global.toml --local-config dotter/local.toml'

# ============================================================================
# Misc Utilities
# ============================================================================

alias monitor="btm"
#alias top="btm"
alias cp-pwd='echo $PWD|pbcopy'
alias ~=grep
alias ls-colors='for x in {0..8}; do for i in {30..37}; do for a in {40..47}; do echo -ne "\e[$x;$i;$a""m\\\e[$x;$i;$a""m\e[0;37;40m "; done; echo; done; done; echo ""'

pandoc-yfm() {
  pandoc "$1" -s -f epub -t markdown-markdown_in_html_blocks --extract-media=./ -o book.md --standalone
}

suckit-sub() {
  suckit -v -j 1 --delay 1 --include-visit "${1}(.*)$" --include-download "${1}(.*)$" "$1"
}

extract() {
  atool --extract --explain "$1"
}
alias x=extract
