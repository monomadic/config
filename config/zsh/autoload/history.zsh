HISTFILE="$HOME/.zsh_history"

# Set the maximum number of history entries
HISTSIZE=100000
SAVEHIST=10000

# Use extended history format
setopt EXTENDED_HISTORY

# Append history entries rather than overwriting the file
setopt APPEND_HISTORY

# Share command history between all sessions
setopt SHARE_HISTORY

# Do not store duplicate entries consecutively
setopt HIST_IGNORE_DUPS

# Do not store entries starting with a space
setopt HIST_IGNORE_SPACE

# Remove command lines from the history list when the first word is the same as the previous command
setopt HIST_IGNORE_ALL_DUPS

# Stop commands that start with a space from going into history
setopt HIST_NO_STORE

# Record each line as it gets issued
setopt INC_APPEND_HISTORY

# Load history from the file after each command
setopt INC_APPEND_HISTORY_TIME

# Expire duplicate entries first when trimming history
setopt HIST_EXPIRE_DUPS_FIRST

# Immediately write the history file after each command.
setopt INC_APPEND_HISTORY

# This allows you to search through your history with the up and down arrows using text typed at the prompt as a search prefix
bindkey '^[[A' history-beginning-search-backward
bindkey '^[[B' history-beginning-search-forward
