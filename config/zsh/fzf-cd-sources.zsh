# Sources for the shared fzf directory picker.
# `static` entries are exact directory paths that should appear immediately.
# `commands` are streamed in the background and are the right place for
# recursive, remote, or otherwise slow searches.

typeset -ga _cd_fzf_global_static=(
  '$HOME'
  '$HOME/src/virtualdj-skin-graveraver'
  '$HOME/Library/Application\ Support/VirtualDJ'
  '$HOME/Movies/Private'
)

typeset -ga _cd_fzf_global_commands=(
  'fd --type d --absolute-path --max-depth 1 . "$HOME/Downloads" 2>/dev/null'
  'fd --type d --absolute-path --max-depth 2 . "$HOME/Movies/Porn" 2>/dev/null'
  'fd --type d --absolute-path --max-depth 1 . "$HOME/Library/Application Support/VirtualDJ" 2>/dev/null'
  'fd --type d --absolute-path --max-depth 2 . "$ICLOUD_HOME" 2>/dev/null'
  'fd --type d --absolute-path --max-depth 1 . "$ICLOUD_HOME/Movies" 2>/dev/null'
  'fd --type d --absolute-path --max-depth 4 . "$HOME/config" 2>/dev/null'
  'for vol in /Volumes/*(N/); do fd --type d --absolute-path --max-depth 1 . "$vol" 2>/dev/null; done'
  'for media_dir in /Volumes/*/Movies/Porn(N/); do fd --type d --absolute-path --max-depth 2 . "$media_dir" 2>/dev/null; done'
)

typeset -gi _cd_fzf_local_max_depth=5
