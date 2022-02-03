echo "loaded .profile"
source "$HOME/.cargo/env"
export PATH="/Users/nom/.deno/bin:$PATH"
export PATH="/Users/nom/.local/share/solana/install/active_release/bin:$PATH"
export PATH="/opt/homebrew/opt/openssl@1.1/bin:$PATH"
export PATH="/Users/nom/.local/share/solana/bin:$PATH"

export EDITOR="nvim"

export FZF_DIR_JUMP='fd --hidden --max-depth 4 --type d'
export FZF_DEFAULT_COMMAND='fd --hidden --max-depth 4 --type f'

alias lite="/Applications/lite-xl.app/Contents/MacOS/lite-xl"
alias enc="gpg --symmetric --cipher-algo AES256 "
alias code=code-insiders
alias g=git
alias gp=git pull
alias gs="git status"
alias r=ranger
alias f=fzf
alias v=nvim
alias vim=nvim
alias code=code-insiders
