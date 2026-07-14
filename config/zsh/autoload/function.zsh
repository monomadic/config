_zsh_source_sibling() {
  local name="$1" file
  for file in "${${(%):-%x}:A:h}/$name" "$ZSH_DOTFILES_DIR/autoload/$name"; do
    [[ -r "$file" ]] || continue
    source "$file"
    return
  done
  return 1
}

_zsh_source_sibling functions/system.zsh
_zsh_source_sibling functions/media-playback.zsh
_zsh_source_sibling functions/files.zsh
_zsh_source_sibling functions/yt-media.zsh
_zsh_source_sibling functions/misc.zsh

unfunction _zsh_source_sibling
