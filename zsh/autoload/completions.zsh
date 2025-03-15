# COMPLETIONS
#
# useful links
#		- https://github.com/zsh-users/zsh-completions
#

# 1password
eval "$(op completion zsh)"

# fd
eval "$(fd --gen-completions zsh)"

# dotter
eval "$(dotter gen-completions --shell zsh)"
