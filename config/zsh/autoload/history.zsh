HISTFILE="$HOME/.zsh_history"

# Keep in-memory and on-disk history the same size
HISTSIZE=100000
SAVEHIST=100000

# Use extended history format
setopt EXTENDED_HISTORY

# Share command history between all sessions.
# (Implies incremental appending — do not combine with INC_APPEND_HISTORY
# or INC_APPEND_HISTORY_TIME; zsh treats setting more than one as an error.)
setopt SHARE_HISTORY

# Do not store duplicate entries consecutively
setopt HIST_IGNORE_DUPS

# Do not store entries starting with a space
setopt HIST_IGNORE_SPACE

# Remove command lines from the history list when the first word is the same as the previous command
setopt HIST_IGNORE_ALL_DUPS

# Stop commands that start with a space from going into history
setopt HIST_NO_STORE

# Expire duplicate entries first when trimming history
setopt HIST_EXPIRE_DUPS_FIRST

# This allows you to search through your history with the up and down arrows using text typed at the prompt as a search prefix
bindkey '^[[A' history-beginning-search-backward
bindkey '^[[B' history-beginning-search-forward
