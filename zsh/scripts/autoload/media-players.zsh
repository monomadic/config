alias fd-video="fd -E '.*\.(mp4|webp|webm|mkv|mov)$' --type=file"

function elmedia-open {
  xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
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

# fzf search and open in vlc
fzf-play() {
  fzf \
    --ansi \
    --multi \
    --exact \
    --bind 'enter:select-all+execute-silent(vlc {+})' \
    --bind 'alt-o:execute-silent(vlc {})' \
    --bind 'alt-i:select-all+execute-silent(iina {+})' \
    --bind 'alt-a:select-all,alt-d:deselect-all,ctrl-/:toggle-preview' \
    --query "$*"
}
