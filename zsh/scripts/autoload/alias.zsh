# media functions
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
alias @="media search"
alias @clips="media search clips"
alias @cumshot="media search #cumshot"
alias @edits="media search edits"
alias @latest="media search latest"
alias @loops="media search loops"
alias @play-all="media-play"
alias @pwd="fd-video | fzf-play"
alias @search-all="media-search"
alias @search-latest="media-search-latest"
alias @search-volumes="ls-media --match-string '/Volumes/' | fzf-play"
alias @search="media-search-safe"
alias @stats="media-stats"
alias @suki="media search #suki"
alias @top="media search #top"

alias autotag=media-autotag

alias c=e-zsh
alias C=e-config
alias g=git
alias p="media play --shuffle"
alias s="media search --hide-path"
alias ms="media-stats"

alias d=download-video
alias dp="download-video porn "
alias dm="download-video music-video "
alias dy="download-video youtube "
alias durl="download-video-url "

alias .dupes-check="fdupes --recurse --cache --nohidden --size --summarize ."
alias .dupes-delete="fdupes --recurse --cache --nohidden --size --delete ."
alias .dupes-delete-interactive="fdupes --recurse --deferconfirmation --cache --nohidden --size --plain ."

alias .tab=fzf-tablature

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

alias faphouse="download-video-faphouse"
alias .faphouse="download-video-faphouse"

alias .brave-mp4-support="/Applications/Brave\ Browser.app/Contents/MacOS/Brave\ Browser --disable-features=MediaSource,UseModernMediaControls"

alias .brewfile="cd $DOTFILES_DIR && e Brewfile"
alias .portal=detect-captive-portal
alias .detect-captive-portal=detect-captive-portal
alias .config=e-zsh

alias amdl=gamdl
alias fd-empty="fd --type empty"
alias gb="git branch "$@" --sort=-committerdate --sort=-HEAD --format=$'%(HEAD) %(color:yellow)%(refname:short) %(color:green)(%(committerdate:relative))\t%(color:blue)%(subject)%(color:reset)' --color=always | column -ts$'\t'"
alias battery='pmset -g batt'
alias branch="b"
alias cb="cargo build"
alias cc="cargo check"
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
alias dd='WD=${PWD} && cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml deploy --global-config global.toml --local-config local.toml && cd $WD && echo "\nDone."'
alias doc="cargo doc --open"
alias dw='cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml watch --global-config global.toml --local-config local.toml'
alias e-brewfile="edit ~/config/Brewfile"
alias e-bin="edit-bin"
alias e-config="nvim $DOTFILES_DIR/README.md"
alias e-joshuto="nvim $DOTFILES_DIR/joshuto/joshuto.toml"
alias e-kitty="nvim $DOTFILES_DIR/kitty/kitty.conf"
alias e-neovim="nvim $DOTFILES_DIR/neovim/init.lua"
alias e-fzf="fzf_edit"
alias e-snippets="cd ~/config/neovim/snippets/ && edit . +'Telescope find_files'"
alias e-wiki=wiki
alias e-zellij="cd ~/config/zellij/ && edit config.kdl"
alias e-zsh="cd ~/config/zsh/ && edit zshrc.zsh"
alias e="nvim"
alias n="fzf-neovim"
alias edit=nvim
alias f-all="fzf-cd"
alias ga="git add . && git commit --amend"
alias gc-update="gc update:"
alias gca="ga"
alias gd="git diff"
alias gen-yew-web3="cargo generate --git https://github.com/monomadic/yew-web3-template"
alias gp="git push"
alias gr="cd /"
alias gs="git status"
alias gss="git status --short --untracked-files=all"
alias iina-shuffle="iina --mpv-shuffle --mpv-loop-playlist"
alias img="chafa --format=symbols "
alias j=joshuto
alias kitty-joshuto="kitty --override background=#000 --working-directory=$HOME/workspaces --single-instance joshuto"
alias l="echo && eza --icons --group-directories-first && echo"
alias la="eza --icons --group-directories-first --all"
alias lg=lazygit
alias lh="eza --icons --group-directories-first --all"
alias ll-fzf="eza --icons --color=always --group-directories-first --no-permissions --no-user -l --ignore-glob '.DS_Store' | fzf --ansi"
alias ll="echo && eza --icons --group-directories-first --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lla="echo && eza --icons --group-directories-first --all --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lll="eza --tree --icons --level 2"
alias lln="eza --icons --all -l --sort=date"
alias loc=tokei
alias ls-colors='for x in {0..8}; do for i in {30..37}; do for a in {40..47}; do echo -ne "\e[$x;$i;$a""m\\\e[$x;$i;$a""m\e[0;37;40m "; done; echo; done; done; echo ""'
alias monitor="btm"
alias mpv-fs="mpv --macos-fs-animation-duration=0 --no-native-fs --fs "
alias mpv-loop-mode="mpv --loop-file=1 --length=10 --macos-fs-animation-duration=0 --no-native-fs --fs "
alias ns="network-status.zsh"
alias o=open
alias org='cd ~/org && e index.md'
alias p8="ping 8.8.8.8"
alias pc="ping cloudflare.com"
alias pandoc-yfm="pandoc "{$1}" -s -f epub -t markdown-markdown_in_html_blocks --extract-media=./ -o book.md --standalone"
alias pg="ping google.com"
alias prev="fzf --layout=reverse --preview 'bat --style=numbers --color=always --line-range :500 {}'"
alias pull="git pull"
alias pull="git pull"
alias push="git push"
alias push="git push"
alias q=exit
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
