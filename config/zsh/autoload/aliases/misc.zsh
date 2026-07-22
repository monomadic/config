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
alias dd-force='cd $DOTFILES_DIR && dotter --force --cache-directory ~/.config/dotter/cache --cache-file ~/.config/dotter/cache.toml deploy --global-config dotter/global.toml --local-config dotter/local.toml'
alias dw='cd $DOTFILES_DIR && dotter --cache-directory ~/.config/dotter/cache --cache-file ~/.config/dotter/cache.toml watch --global-config dotter/global.toml --local-config dotter/local.toml'
alias dotter-packages='$DOTFILES_DIR/setup/macos/packages.sh'

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
