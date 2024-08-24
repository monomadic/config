alias @config="cd $ZSH_DOTFILES_DIR && nvim zshrc.zsh"
alias @config-zsh=@config
alias @config-env="cd $ZSH_DOTFILES_DIR && nvim scripts/autoload/alias.zsh"
alias @config-aliases=@config-env
alias @config-bin="cd $DOTFILES_DIR/bin && nvim ."
alias @network-detect-captive-portal=detect-captive-portal-color.zsh
alias @network-status=ns
alias @ns=ns
alias @open-captive-portal=detect-captive-portal-color.zsh
alias @status-network=ns
alias @tab="cd $TABLATURE_DIR && fd . --extension pdf | fzf --bind 'enter:execute(open {})'"
alias @zsh-config-edit="e-zsh"
alias @rename="fd-rename-all.zsh"
alias @uptime="uptime-pretty.zsh"

alias b="git branch "$@" --sort=-committerdate --sort=-HEAD --format=$'%(HEAD) %(color:yellow)%(refname:short) %(color:green)(%(committerdate:relative))\t%(color:blue)%(subject)%(color:reset)' --color=always | column -ts$'\t'"
alias battery='pmset -g batt'
alias branch="b"
alias cb="cargo build"
alias cc="cargo check"
alias cd-blue="cd /Volumes/BabyBlue2TB/not-porn"
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
alias dd='WD=${PWD} && cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml deploy --global-config global.toml --local-config local.toml && cd $WD && clear && zsh-reload'
alias doc="cargo doc --open"
alias dw='cd ~/config/ && dotter --cache-directory ~/.config/dotter/cache/ --cache-file ~/.config/dotter/cache.toml watch --global-config global.toml --local-config local.toml'
alias e-brewfile="edit ~/config/Brewfile"
alias e-config="cd ~/config/ && edit ."
alias e-fzf="fzf_edit"
alias e-joshuto="cd ~/config/joshuto/ && edit joshuto.toml"
alias e-kitty="cd ~/config/kitty/ && edit kitty.conf"
alias e-neovim="cd ~/config/neovim/ && edit init.lua"
alias e-snippets="cd ~/config/neovim/snippets/ && edit . +'Telescope find_files'"
alias e-wiki=wiki
alias e-zellij="cd ~/config/zellij/ && edit config.kdl"
alias e-zsh="cd ~/config/zsh/ && edit zshrc.zsh"
alias e="nvim"
alias edit=nvim
alias exa=eza
alias f-all="fzf-cd"
alias fd-video="fd -i -e mp4 -e avi -e mkv -e mov -e wmv -e flv -e webm --color=always"
alias ga="git add . && git commit --amend"
alias gb="b"
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
alias l="echo && exa --icons --group-directories-first && echo"
alias la="exa --icons --group-directories-first --all"
alias lg=lazygit
alias lh="exa --icons --group-directories-first --all"
alias ll-fzf="exa --icons --color=always --group-directories-first --no-permissions --no-user -l --ignore-glob '.DS_Store' | fzf --ansi"
alias ll="echo && exa --icons --group-directories-first --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lla="echo && exa --icons --group-directories-first --all --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lll="exa --tree --icons --level 2"
alias lln="exa --icons --all -l --sort=date"
alias loc=tokei
alias ls-colors='for x in {0..8}; do for i in {30..37}; do for a in {40..47}; do echo -ne "\e[$x;$i;$a""m\\\e[$x;$i;$a""m\e[0;37;40m "; done; echo; done; done; echo ""'
alias monitor="btm"
alias mpv-fs="mpv --macos-fs-animation-duration=0 --no-native-fs --fs"
alias mpv-loop-mode="mpv --loop-file=1 --length=10 --macos-fs-animation-duration=0 --no-native-fs --fs "
alias ns="network-status.zsh"
alias o=open
alias org='cd ~/org && e index.md'
alias p8="ping 8.8.8.8"
alias p="ping cloudflare.com"
alias pandoc-yfm="pandoc "{$1}" -s -f epub -t markdown-markdown_in_html_blocks --extract-media=./ -o book.md --standalone"
alias pg="ping google.com"
alias prev="fzf --layout=reverse --preview 'bat --style=numbers --color=always --line-range :500 {}'"
alias price="price.py"
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
