# Restore Ctrl+P and Ctrl+N for history navigation
bindkey '^P' up-line-or-history   # Ctrl+P for previous command
bindkey '^N' down-line-or-history # Ctrl+N for next command

# Optionally, restore more common keybindings if needed:
bindkey '^R' history-incremental-search-backward # Ctrl+R for reverse search
bindkey '^A' beginning-of-line                   # Ctrl+A to go to the start of the line
bindkey '^E' end-of-line                         # Ctrl+E to go to the end of the line
bindkey '^U' kill-whole-line                     # Ctrl+U to delete the whole line
bindkey '^K' kill-line                           # Ctrl+K to delete the rest of the line
