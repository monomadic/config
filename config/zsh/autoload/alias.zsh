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

topaz-video() {
  local binary
  binary="$(topaz_resolve_gui_binary)" || return
  [[ -x "$binary" ]] || {
    print -u2 "Topaz GUI binary not found or not executable: $binary"
    topaz_app_help >&2
    return 1
  }
  env LC_ALL=C LC_NUMERIC=C LANG=C "$binary" "$@"
}
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
alias .config-env="cd $ZSH_DOTFILES_DIR && $EDITOR autoload/alias.zsh"
alias config-dotfiles="cd $DOTFILES_DIR && fd --type directory --max-depth=2 | fzf | xargs $EDITOR"
alias configure-mpv="cd $DOTFILES_DIR/config/mpv && kitty-exec '   mpv.conf ' '#A442F3' hx ."
alias configure-helix="cd $DOTFILES_DIR/config/helix && $EDITOR ."

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


# ============================================================================
# Git Shortcuts
# ============================================================================

alias g=git
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
cdd() { local d; d="$(fzf-open --local)" && cd -- "$d"; }
alias fd-dirs="fd -t d -d 15 -E '.*' -E 'Library'"
alias fd-empty="fd --type empty"

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
alias sb=switchblade
alias prev="fzf --layout=reverse --preview 'bat --style=numbers --color=always --line-range :500 {}'"

alias .tab=fzf-tablature
alias t=fzf-tablature
alias .chordpro="cd '${TABLATURE_DIR}/ChordPro' && chordpro-tui ."



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

alias deploy='dotter-deploy'
alias dd-force='cd ~/config/ && dotter --force --cache-directory ~/.config/dotter/cache --cache-file ~/.config/dotter/cache.toml deploy --global-config dotter/global.toml --local-config dotter/local.toml'
alias dw='cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache --cache-file ~/.config/dotter/cache.toml watch --global-config dotter/global.toml --local-config dotter/local.toml'

# ============================================================================
# Misc Utilities
# ============================================================================

alias monitor="btm"
#alias top="btm"
alias cp-pwd='echo $PWD|pbcopy'
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
