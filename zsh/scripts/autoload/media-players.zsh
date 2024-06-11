alias fd-video="fd -E '.*\.(mp4|webp|webm|mkv|mov)$' --type=file"

function elmedia-open {
  fd-video | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
}

# function airflow-open {
#   fd-video | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Airflow.app/Contents/MacOS/Airflow
# }

function fzf-elmedia {
  local search_term="$1"
  local files

  echo "A"
  #files=$(fd -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$')
  files=$(fd -E '.*\.(mp4|webp|webm|mkv|mov)$' --type=file)
  echo "b"

  if [[ -n "$files" ]]; then
    echo "$files" | xargs -I {} open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player "{}"
  else
    echo "No matching files found."
  fi
  #echo $files | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
}
