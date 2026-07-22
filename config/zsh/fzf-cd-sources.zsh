# Sources for the shared fzf directory picker.
# `static` entries are exact directory paths that should appear immediately.
# `commands` are streamed in the background and are the right place for
# recursive, remote, or otherwise slow searches.

typeset -ga _cd_fzf_global_static=(
  '$HOME'
  '$DOTFILES_DIR'
  '$HOME/Library/Application\ Support/VirtualDJ'
  '$HOME/Movies/Private'
  '$HOME/Movies/Porn'
  '$ICLOUD_HOME/Music'
  '$ICLOUD_HOME/Music/Tablature'
  '$ICLOUD_HOME/Music/Tablature/ChordPro'
  '$ICLOUD_HOME/Movies/Visuals'
  '$ICLOUD_HOME/Movies/Visuals/Downloads'
)

fd-dirs() {
  fd --type d --absolute-path --max-depth $2 . "$1" 2>/dev/null
}

typeset -ga _cd_fzf_global_commands=(
  'fd-dirs "$HOME" 1'
  'fd-dirs "$HOME/src" 1'
  'fd-dirs "$HOME/Music" 1'
  'fd --type d --absolute-path --max-depth 2 . "$HOME/Movies/Porn" 2>/dev/null'
  'fd --type d --absolute-path --max-depth 1 . "$HOME/Library/Application Support/VirtualDJ" 2>/dev/null'
  # 'fd --type d --absolute-path --max-depth 1 . "$ICLOUD_HOME" 2>/dev/null'
  # 'fd --type d --absolute-path --max-depth 1 . "$ICLOUD_HOME/Movies" 2>/dev/null'
  'fd-dirs "$ICLOUD_HOME/Music" 1'
  'fd --type d --absolute-path --max-depth 2 . "$DOTFILES_DIR/config" 2>/dev/null'
  'for vol in /Volumes/*(N/); do fd --type d --absolute-path --max-depth 1 . "$vol" 2>/dev/null; done'
  # 'for media_dir in /Volumes/*/Movies/Porn(N/); do fd --type d --absolute-path --max-depth 1 . "$media_dir" 2>/dev/null; done'

  'for vol in /Volumes/*/Movies/Porn/(N/); do fd --type d --absolute-path --max-depth 1 . "$vol" 2>/dev/null; done'
)

typeset -gi _cd_fzf_local_max_depth=5
