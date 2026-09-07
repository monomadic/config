_zsh_source_sibling() {
  local name="$1" file
  for file in "${${(%):-%x}:A:h}/$name" "$ZSH_DOTFILES_DIR/autoload/$name"; do
    [[ -r "$file" ]] || continue
    source "$file"
    return
  done
  return 1
}

alias kitty-center="kitty +kitten panel --edge=center-sized --lines=600px --columns=800px --focus-policy=exclusive --layer=overlay -o background_opacity=0.95 -o hide_window_decorations=yes -o window_padding_width=10 fzf"

_zsh_source_sibling aliases/core.zsh
_zsh_source_sibling aliases/downloads.zsh
_zsh_source_sibling aliases/system.zsh
_zsh_source_sibling aliases/dev.zsh
_zsh_source_sibling aliases/navigation.zsh
_zsh_source_sibling aliases/misc.zsh

unfunction _zsh_source_sibling
