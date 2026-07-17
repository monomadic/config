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
alias .configure-fzf-cd="cd $DOTFILES_DIR/config/zsh && $EDITOR ./fzf-cd-sources.zsh"

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
