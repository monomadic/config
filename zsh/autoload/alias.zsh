# media functions

# media play
alias .4k="media play #4k --shuffle"
alias .perf="media play #60fps #4k --shuffle"
alias .60fps="media play #60fps --shuffle"
alias .clips="media play clips --shuffle"
alias .cumshot="media play #cumshot --shuffle"
alias .edits="media play edits --shuffle"
alias .latest="media play latest"
alias .loops="media play loops --shuffle"
alias .pwd="fd-video | mpv-play"
alias .suki-latest="media play latest #suki"
alias .suki="media play #suki --shuffle"
alias .top-cumshot="media play #top #cumshot --shuffle"
alias .top="media play #top --shuffle"
alias .upscaled="media play #upscaled --shuffle"
alias .lib="media play library --shuffle"
alias .remaster="media play #remaster --shuffle"

# media search
alias @="media search"
alias @@="media search edits"
alias @clips="media search clips"
alias @cumshot="media search #cumshot"
alias @edits="media search edits"
alias @latest="media search latest"
alias @loops="media search loops"
alias @play-all="media-play"
alias @pwd="fd-video | fzf-play"
alias @volumes="ls-media --match-string '/Volumes/' | fzf-play"
alias @safe="media-search-safe"
alias @stats="media-stats"
alias @suki="media search #suki"
alias @top="media search #top"
alias @lib="media search library"
alias @tutorials="fd-video . $TUTORIALS_PATH | fzf-play"
alias @perf="media search #60fps #4k"
alias @remaster="media search #remaster"

alias .kitty-reload="kitty @ set-colors --all ~/.config/kitty/kitty.conf"
alias .kitty-configure="cfg-kitty"

alias .demux="ffmpeg-demux"
alias .demux-audio="ffmpeg-demux --skip-video"
alias .demux-video="ffmpeg-demux --skip-audio"

alias url="download-video-url"

alias media-backup-src="rsync-backup SRC"
alias media-cache="rsync-backup --dry-run $HOME/Movies/Cache \/clipped\/ "

alias tag=media-autotag
alias .tag=tag

alias remove-gatekeeper="xattr -rd com.apple.quarantine "
alias .gatekeeper-remove=remove-gatekeeper
alias gatekeeper-remove="xattr -rd com.apple.quarantine "

alias c=cfg-zsh
alias C=e-config
alias g=git
alias p="media play --shuffle"
alias s="media search --hide-path"
alias ms="media-stats"

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

alias .tab=fzf-tablature

alias .macos-keybindings="source $DOTFILES_DIR/scripts/macos-keybindings.sh"

alias .rename="fd-rename-all.zsh"
alias .config-aliases=.config-env
alias .config-bin="cd $DOTFILES_DIR/bin && nvim ."
alias .config-env="cd $ZSH_DOTFILES_DIR && nvim scripts/autoload/alias.zsh"
alias .config-zsh=e-zsh
alias .config="cd $ZSH_DOTFILES_DIR && nvim zshrc.zsh"
alias .apple-music-dl=apple-music-dl
alias .network-detect-captive-portal=detect-captive-portal
alias .network-status=ns
alias .uptime="display-uptime"
alias .clear-notifications="killall NotificationCenter"

alias faphouse="download-video-faphouse"
alias .faphouse="download-video-faphouse"

alias .brave-mp4-support="/Applications/Brave\ Browser.app/Contents/MacOS/Brave\ Browser --disable-features=MediaSource,UseModernMediaControls"

alias .brewfile="cd $DOTFILES_DIR && e Brewfile"
alias .portal=detect-captive-portal
alias .detect-captive-portal=detect-captive-portal
alias .config=e-zsh
alias .fonts="kitty list-fonts"

alias mpv-auto-safe="mpv --hwdec=auto-safe --vo=libmpv "

#alias ll="echo && eza --icons --group-directories-first --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias amdl=gamdl
alias battery='pmset -g batt'
alias branch="b"
alias cb="cargo build"
alias cc="cargo check"
alias cfg-homebrew="cd $DOTFILES_DIR && nvim Brewfile"
alias cfg-kitty="cd $DOTFILES_DIR/kitty && nvim kitty.conf"
alias cfg-neovim="cd $DOTFILES_DIR/neovim && edit init.lua"
alias cfg-open="cd $DOTFILES_DIR && nvim README.md"
alias cfg-yazi="cd $DOTFILES_DIR/apps/yazi && nvim yazi.toml"
alias .yazi-config="cd $DOTFILES_DIR/apps/yazi && nvim yazi.toml"
alias cfg-zellij="cd $DOTFILES_DIR/zellij && edit config.kdl"
alias cfg-zsh-keybindings="cd $DOTFILES_DIR/zsh && nvim scripts/autoload/keybindings.zsh"
alias cfg-zsh="cd $DOTFILES_DIR/zsh && edit zshrc.zsh"
alias cfg-mpv="cd $DOTFILES_DIR/mpv && edit mpv.conf"
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
alias e-bin="edit-bin"
alias e="nvim"
alias edit=nvim
alias fd-empty="fd --type empty"
alias ga="git add . && git commit --amend"
alias gb="git branch "$@" --sort=-committerdate --sort=-HEAD --format=$'%(HEAD) %(color:yellow)%(refname:short) %(color:green)(%(committerdate:relative))\t%(color:blue)%(subject)%(color:reset)' --color=always | column -ts$'\t'"
alias gc-update="gc update:"
alias gca="ga"
alias gd="git diff"
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