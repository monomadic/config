alias fd-dirs-all="fd --type d --hidden --exclude .git --exclude Library --exclude target --exclude go --exclude .cargo --exclude .local --exclude .rustup --exclude build --color always"
alias fd-dirs="fd --type d --exclude .git --exclude Library --exclude target --exclude go --exclude .cargo --exclude .local --exclude .rustup --exclude build --color always"

function elmedia-open {
  # xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
  xargs -I {} open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player "$*"
}

fzf-fd-cd() {
  cd $(fd-dirs-all | fzf --ansi --exact)
}

# function elmedia-open {
#   fd-video | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
# }

# function fzf-elmedia {
#   local files
#   files=$(fd-video)
#
#   if [[ -n "$files" ]]; then
#     echo "$files" | xargs -I {} open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player "{}"
#   else
#     echo "No matching files found."
#   fi
#   # echo $files | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
# }
