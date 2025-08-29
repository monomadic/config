ICLOUD_HOME="$HOME/Library/Mobile Documents/com~apple~CloudDocs"
DJ_VISUALS_PATH="$ICLOUD_HOME/Movies/Visuals"

alias yt-dlp-youtube-embedded="yt-dlp --cookies-from-browser brave --continue --progress --verbose --retries infinite --fragment-retries infinite --socket-timeout 15 -f bestvideo+ba/best --embed-metadata --extractor-args 'youtube:player-client=tv_embedded' "

# media functions

alias mpv-loop="mpv --player-operation-mode=pseudo-gui --loop-file=inf --loop-playlist=inf --image-display-duration=5 --force-window=yes --no-config --no-input-default-bindings "
alias .pwd="mpv-play $PWD"
alias mpv-play-porn="setopt local_options null_glob && mpv-play $~MEDIA_GLOBS"
alias mpv-play-volumes="mpv-play /Volumes/*/Movies/**/*.mp4"

# media search
alias @="ls-media | fzf-play --hide-path --tac"
alias @@@="setopt local_options null_glob && printf '%s\0' $~MEDIA_GLOBS | fzf-play --hide-path -0"
alias @@@pwd="fd-video --absolute-path --print0 | fzf-play --hide-path -0"
alias @sort="fselect-porn-sort -0 | fzf-play --hide-path --tac"
alias @loop="fselect-porn -0 | fzf-media-select --hide-path --tac | mpv-with-config -"
alias @pwd-sort="fselect-pwd-sort -0 | fzf-play --hide-path --tac"
alias @@="fd-video | fzf-play --hide-path --tac"
alias @volumes="fd-video --print0 . /Volumes/*/Movies/Porn |fzf-play --hide-path -0"
alias @masters="fd-video --print0 . /Volumes/*/Movies/Porn/Masters/ $HOME/Movies/Porn/Masters/  |fzf-play -0"
alias @tutorials="fd-video . $TUTORIALS_PATH | fzf-play"
alias @external=@volumes
alias @dj-visuals="fd-video . '$DJ_VISUALS_PATH' '$HOME/Movies/Visuals' '/Volumes/*/Movies/Visuals' | fzf-play --hide-path"
alias .dj-visuals="mpv-loop '$DJ_VISUALS_PATH' '$HOME/Movies/Visuals' '/Volumes/*/Movies/Visuals'"
alias .python-venv-create="python3 -m venv .venv && source .venv/bin/activate"
alias .python-venv-activate="source .venv/bin/activate"
alias .python-pip-install-requirements="pip install -r requirements.txt"

fd-video-color() {
  { fd -e mp4 $1 } | sd '\]\[' '] [' | sd '\[([^\]]+)\]' $'\e[32m''$1'$'\e[0m' | sd '\{([^}]*)\}' $'\e[33m''$1'$'\e[0m' | sd '(^|/)\(([^)]*)\)' '${1}'$'\e[36m''$2'$'\e[0m' | rg --passthru --color=always -N -r '$0' -e '#\S+' --colors 'match:fg:magenta'
}

rename-m4v-to-mp4() {
  local f new
  for f in *.m4v; do
    [[ -e "$f" ]] || continue  # skip if no match
    new="${f%.m4v}.mp4"
    if [[ -e "$new" ]]; then
      print -P "%F{yellow}Skipping:%f $f → $new (already exists)"
    else
      mv -- "$f" "$new"
      print -P "%F{green}Renamed:%f  $f → $new"
    fi
  done
}

# note:
# pipx install demucs
# pipx inject demucs soundfile
alias .stem-split="demucs -d mps -n htdemucs --flac -o stems_output"
alias .stem-split-2="demucs -d mps -n htdemucs --flac -o stems_output --two-stems=vocals"
alias .stem-split-4="demucs -d mps -n htdemucs --flac -o stems_output"

.stem-mdx23() {
  cd $HOME/Music/Stems/MVSEP-MDX23-Colab_v2.1 &&
    source .venv/bin/activate &&
    time python inference_2.2_b1.5.1_voc_ft.py --input_audio $1 --output_folder $HOME/Music/Stems --large_gpu --chunk_size 500000
}

alias fd-dirs="fd -t d -d 15 -E '.*' -E 'Library'"

alias .kitty-mark-current-tab-orange="kitty @ set-tab-color active_bg=orange active_fg=white inactive_bg=orange inactive_fg=black"
alias .kitty-mark-current-tab-red="kitty @ set-tab-color inactive_bg=red inactive_fg=black"
alias .kitty-set-tab-color-orange="kitty @ set-tab-color --match id:$KITTY_WINDOW_ID active_bg=#FFA500 active_fg=#050F63 inactive_fg=#FFA500 inactive_bg=#030D43"
alias .kitty-set-tab-color-green="kitty @ set-tab-color --match id:$KITTY_WINDOW_ID active_bg=#38F273 active_fg=#050F63 inactive_fg=#38F273 inactive_bg=#030D43"

alias mpv-with-config="mpv --profile=fast --video-sync=display-resample --hwdec=auto-safe --shuffle --no-native-fs --macos-fs-animation-duration=0 --mute"
alias mpv-without-config="mpv --profile=fast --video-sync=display-resample --hwdec=auto-safe --no-config --shuffle --no-native-fs --macos-fs-animation-duration=0 --mute"

alias topaz-video="env LC_ALL=C LC_NUMERIC=C LANG=C /Applications/Topaz\ Video\ AI.app/Contents/MacOS/Topaz\ Video\ AI"
alias .topaz-video=topaz-video

alias .list-moved-files="fclones group --cache --hash-fn metro --isolate --dry-run"
alias config-dotfiles="cd $DOTFILES_DIR && fd --type directory --max-depth=2 | fzf | xargs $EDITOR"

alias cd-relative="cd ${fd--type directory | fzf-cd}"

alias .get-app-id="osascript -e 'id of app $1'"
alias .get-app-id-debug="echo \"osascript -e 'id of app ${1}'\""

alias .screen-sharing-kick-users="sudo /System/Library/CoreServices/RemoteManagement/ARDAgent.app/Contents/Resources/kickstart -deactivate -restart -users current"

alias .kitty-reload="kitty @ set-colors --all ~/.config/kitty/kitty.conf"
alias .kitty-configure="e-kitty"
alias .kitty-kill-all-nvim="kitten @ close-tab --match 'env:PROC=nvim'"
alias .nvim-kill-all=.kitty-kill-all-nvim
alias .kitty-close-idle-tabs="kitty @ close-tab --match 'env:PROC=zsh'"

alias .demux="ffmpeg-demux"
alias .demux-video="ffmpeg-demux --video"
alias .demux-audio="ffmpeg-demux --audio"

alias url="yt-url"

alias rsync-backup-masters="rsync-backup /Volumes/Masters/Movies/Porn/Masters $BACKUP_TARGET/Movies/Porn/Masters"
alias tag=rename-media
alias .tag=tag

alias .gatekeeper-whitelist="xattr -rd com.apple.quarantine "
alias .self-sign="codesign --sign - --force --deep "

alias c=e-zsh
alias C=e-config
alias g=git

alias rn="batch-rename"
alias ren="batch-rename"

alias d=download-video
alias dp="download-video porn "
alias dmv="download-video music-video "
alias dyt="download-video youtube "
alias dlu="download-video-url "

alias .dupes-check="fdupes --recurse --cache --nohidden --size --summarize ."
alias .dupes-delete="fdupes --recurse --cache --nohidden --size --delete ."
alias .dupes-delete-interactive="fdupes --recurse --deferconfirmation --cache --nohidden --size --plain ."

alias ls-usb="system_profiler SPUSBDataType"
alias ls-usb-ioreg="ioreg -p IOUSB -w0"
alias ls-disks="diskutil list"

alias .tab=fzf-tablature
alias t=fzf-tablature

alias .macos-keybindings="source $DOTFILES_DIR/scripts/macos-keybindings.sh"

alias .rename="fd-rename-all.zsh"
alias .config-aliases=.config-env
alias .config-bin="cd $DOTFILES_DIR/bin && $EDITOR ."
alias .config-env="cd $ZSH_DOTFILES_DIR && $EDITOR scripts/autoload/alias.zsh"
alias .config-zsh=e-zsh
alias .config="cd $ZSH_DOTFILES_DIR && $EDITOR zshrc.zsh"
alias .apple-music=apple-music-dl
alias .beatport="beatportdl-darwin-arm64"
alias .tidal="noglob tidal-dl-ng dl"
alias .network-detect-captive-portal=detect-captive-portal
alias .network-status=ns
alias .uptime="display-uptime"
alias .clear-notifications="killall NotificationCenter"
alias f="noglob fetch"

alias dl-beatport=beatportdl-darwin-arm64
alias dl-apple-music=apple-music-dl

alias faphouse="download-video-faphouse"
alias .faphouse="download-video-faphouse"

alias .brave-mp4-support="/Applications/Brave\ Browser.app/Contents/MacOS/Brave\ Browser --disable-features=MediaSource,UseModernMediaControls"

alias .brewfile="cd $DOTFILES_DIR && e Brewfile"
alias .portal=detect-captive-portal
alias .detect-captive-portal=detect-captive-portal
alias .config=e-zsh
alias .fonts="kitty list-fonts"

alias mpv-auto-safe="mpv --hwdec=auto-safe --vo=libmpv "

# Quick image viewer that loops
alias mpv-image-viewer='mpv-stdin --image-display-duration=inf'
alias mpv-image-slideshow='mpv-stdin --image-display-duration=5'

alias battery='pmset -g batt'
alias branch="b"
alias cb="cargo build"
alias cc="cargo check"
alias e-homebrew="cd $DOTFILES_DIR && $EDITOR Brewfile"
alias e-kitty="cd $DOTFILES_DIR/kitty && $EDITOR kitty.conf"
alias e-neovim="cd $DOTFILES_DIR/neovim && $EDITOR init.lua"
alias e-open="cd $DOTFILES_DIR && $EDITOR README.md"
alias e-yazi="cd $DOTFILES_DIR/apps/yazi && $EDITOR yazi.toml"
alias e-zellij="cd $DOTFILES_DIR/zellij && $EDITOR config.kdl"
alias e-zsh-keybindings="cd $DOTFILES_DIR/zsh && $EDITOR scripts/autoload/keybindings.zsh"
alias e-zsh="cd $DOTFILES_DIR/zsh && $EDITOR zshrc.zsh"
alias e-mpv="cd $DOTFILES_DIR/mpv && $EDITOR mpv.conf"
alias .yazi-config="cd $DOTFILES_DIR/apps/yazi && $EDITOR yazi.toml"
alias ci="cargo install --path ."
alias cp-pwd="echo $PWD|pbcopy" # mac only
alias cr="cargo run"
alias crr="cargo run --release"
alias ct="cargo test"
alias d-bevy-cheat="open https://bevy-cheatbook.github.io/"
alias d-rs-yew="open https://docs.rs/yew/latest/yew/"
alias d-rustdoc="open https://doc.rust-lang.org/rustdoc/"
alias d-rustup-cargo="rustup doc --cargo"
alias d-rustup-core="rustup doc --core"
alias d-wasmtime="open https://docs.wasmtime.dev/"
alias d-yew="open https://yew.rs/docs/next/"
alias dd-force='WD=${PWD} && cd ~/config/ && dotter --force --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml deploy --global-config global.toml --local-config local.toml && cd $WD && echo "\nDone."'
alias dd='WD=${PWD} && cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml deploy --global-config global.toml --local-config local.toml && cd $WD && echo "\nDone."'
alias doc="cargo doc --open"
alias dw='cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml watch --global-config global.toml --local-config local.toml'
alias e=$EDITOR
alias eb="edit-bin"
alias edit=$EDITOR
alias fd-empty="fd --type empty"
alias ga="git add . && git commit --amend"
alias gb="git branch "$@" --sort=-committerdate --sort=-HEAD --format=$'%(HEAD) %(color:yellow)%(refname:short) %(color:green)(%(committerdate:relative))\t%(color:blue)%(subject)%(color:reset)' --color=always | column -ts$'\t'"
alias gc-update="gc update:"
alias gca="ga"
alias gd="git diff"
alias git-stage-last-commit="git reset --soft HEAD~"
alias gen-yew-web3="cargo generate --git https://github.com/monomadic/yew-web3-template"
alias gl="fzf-git-log"
alias gp="git push"
alias gr="cd /"
alias gs="git status"
alias gss="git status --short --untracked-files=all"
alias iina-shuffle="iina --mpv-shuffle --mpv-loop-playlist"
alias img="chafa --format=symbols "
alias kitty-joshuto="kitty --override background=#000 --working-directory=$HOME/workspaces --single-instance joshuto"
alias l="echo && eza --icons --group-directories-first && echo"
alias la="eza --icons --group-directories-first --all"
alias lg=lazygit
alias lh="eza --icons --group-directories-first --all"
alias ll-fzf="eza --icons --color=always --group-directories-first --no-permissions --no-user -l --ignore-glob '.DS_Store' | fzf --ansi"
alias ll="echo && lsd --icon always --long --depth 1 --ignore-config --blocks name --group-directories-first --color always && echo"
alias lla="echo && eza --icons --group-directories-first --all --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lll="lsd --icon always --long --depth 1 --ignore-config --group-directories-first --color always"
alias lln="eza --icons --all -l --sort=date"
alias loc=tokei
alias ls-colors='for x in {0..8}; do for i in {30..37}; do for a in {40..47}; do echo -ne "\e[$x;$i;$a""m\\\e[$x;$i;$a""m\e[0;37;40m "; done; echo; done; done; echo ""'
alias monitor="btm"
alias mpv-fs="mpv --macos-fs-animation-duration=0 --no-native-fs --fs "
alias mpv-loop-mode="mpv --loop-file=1 --length=10 --macos-fs-animation-duration=0 --no-native-fs --fs "
alias n="fzf-neovim"
alias ns="network-status.zsh"
alias o=open
alias org='cd ~/org && e index.md'
alias p8="ping 8.8.8.8"
alias pandoc-yfm="pandoc "{$1}" -s -f epub -t markdown-markdown_in_html_blocks --extract-media=./ -o book.md --standalone"
alias pc="ping cloudflare.com"
alias pg="ping google.com"
alias prev="fzf --layout=reverse --preview 'bat --style=numbers --color=always --line-range :500 {}'"
alias pull="git pull"
alias pull="git pull"
alias push="git push"
alias push="git push"
alias q=exit
alias sb="fzf-scrollback"
alias sips-to-webp-lossy='sips -s format webp -s formatOptions 75'
alias sips-to-webp-lossy='sips -s format webp -s'
alias sixel-kitty="chafa --clear --format=kitty --center=on --scale=max "
alias sixel-sixel="chafa --clear --format=sixel --center=on --scale=max "
alias sixel="chafa --clear --format=symbol --center=on --scale=max "
alias snippets="cd ~/config/neovim/snippets/ && ll"
alias src="cd ~/src && l"
alias suckit-sub="suckit -v -j 1 --delay 1 --include-visit '${1}(.*)$' --include-download '${1}(.*)$' ${1}"
alias top="btm" # I always forget which monitor I have installed
alias trash-undo="rip --unbury "
alias trash-view="rip --seance"
alias unzip 'atool --extract --explain $1'
alias up="cd .."
alias v="viu --height 20"
alias vi="nvim"
alias vi=nvim
alias vim="nvim"
alias vim=nvim
alias w=wiki
alias wiki="cd ~/wiki && e index.md"
alias wiki='cd ~/wiki && e ~/wiki/index.md'
alias workspaces="cd ~/workspaces && l"
alias xdg-open=open
alias zsh-config="cd $DOTFILES_DIR/zsh/ && nvim zshrc.zsh"
alias zsh-reload="source ~/.zshrc"
alias ~=grep
