export FZF_DEFAULT_OPTS="--layout=reverse --inline-info --color=bg+:-1,fg:4,info:15,fg+:5,header:7,hl:5,hl+:5"
export SKIM_DEFAULT_OPTIONS=$FZF_DEFAULT_OPTS
export SKIM_DEFAULT_COMMAND="fd . --max-depth=3"

# function fzf-vlc {
#   local search_term="$1"
# 	fd --fixed-strings "$search_term" -0 |xargs -0 vlc
# }

function fzf-brave-bookmarks() {
  # bookmarks_path=~/Library/Application\ Support/Google/Chrome/Default/Bookmarks
  bookmarks_path=~/Library/Application\ Support/BraveSoftware/Brave-Browser/Default/Bookmarks

  jq_script='
        def ancestors: while(. | length >= 2; del(.[-1,-2]));
        . as $in | paths(.url?) as $key | $in | getpath($key) | {name,url, path: [$key[0:-2] | ancestors as $a | $in | getpath($a) | .name?] | reverse | join("/") } | .path + "/" + .name + "\t" + .url'

  jq -r "$jq_script" <"$bookmarks_path" |
    sed -E $'s/(.*)\t(.*)/\\1\t\x1b[36m\\2\x1b[m/g' |
    fzf --ansi |
    cut -d$'\t' -f2 |
    xargs open
}

function fzf-brave-history() {
  local cols sep google_history open
  cols=$((COLUMNS / 3))
  sep='{::}'

  if [ "$(uname)" = "Darwin" ]; then
    google_history="$HOME/Library/Application Support/BraveSoftware/Brave-Browser/Default/History"
    open=open
  else
    google_history="$HOME/.config/google-chrome/Default/History"
    open=xdg-open
  fi
  cp -f "$google_history" /tmp/h
  sqlite3 -separator $sep /tmp/h \
    "select substr(title, 1, $cols), url
     from urls order by last_visit_time desc" |
    awk -F $sep '{printf "%-'$cols's  \x1b[36m%s\x1b[m\n", $1, $2}' |
    fzf --ansi --multi \
      --prompt ' ' |
    sed 's#.*\(https*://\)#\1#' | xargs $open >/dev/null 2>/dev/null
}
