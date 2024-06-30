export FZF_DEFAULT_OPTS="--layout=reverse --inline-info --color=bg+:-1,fg:4,info:15,fg+:5,header:7,hl:5,hl+:5"
export SKIM_DEFAULT_OPTIONS=$FZF_DEFAULT_OPTS
export SKIM_DEFAULT_COMMAND="fd . --max-depth=3"

# history
# bind '"\C-r": "$(fc -rl 1 | fzf -e)"'

# ssh into known_hosts
function fzf-ssh() {
  ssh $(grep -oP 'Host \K.*' ~/.ssh/config | fzf)
}

# kill processes
function fzf-kill() {
  kill -9 $(ps -ef | fzf | awk '{print $2}')
}

# set environment variables
function fzf-env() {
  export $(printenv | fzf | cut -d= -f1)
}

# show brew info for installed packages
function fzf-brew-installed() {
  brew list -1 | fzf \
    --preview 'brew info {}' \
    --bind 'enter:execute(open $(brew info --json=v1 {} | jq -r ".[0].homepage"))'
}

# alias mark='echo $PWD >> ~/.marks'
# alias jump='cd $(cat ~/.marks | fzf)'

# install with homebrew
# function fzf-brew-install() {
#   local formulae
#   formulae=$(brew search | sort)
#   local selected
#   selected=$(echo "$formulae" | fzf -m --preview 'brew info {}')
#   if [[ -n "$selected" ]]; then
#     echo "$selected" | tr ' ' '\n' | xargs -I {} brew install {}
#   fi
# }

# copy lines from the scrollback buffer
function fzf-scrollback() {
  local scrollback
  scrollback=$(kitty @ get-text)

  if [[ -n "$scrollback" ]]; then
    selected=$(echo "$scrollback" | fzf --no-sort)
    if [[ -n "$selected" ]]; then
      echo "$selected" | kitten clipboard
      echo "Copied to clipboard: $selected"
    fi
  else
    echo "No scrollback buffer available."
  fi
}
alias @sb="fzf-scrollback"
alias sb="fzf-scrollback"

# search emojis
function fzf-emoji() {
  emojis=$(cat ~/.zsh/autoload/emoji.json | jq -r '.[] | "\(.emoji) \(.description)"')
  selected=$(echo "$emojis" | fzf --preview 'echo {1}' --preview-window up:1)
  echo -n "${selected%% *}" | pbcopy
  echo "Copied ${selected%% *} to clipboard!"
}

function fzf-git-switch-branch() {
  local branches branch
  branches=$(git branch --all | grep -v HEAD) &&
    branch=$(echo "$branches" | fzf -d $((2 + $(wc -l <<<"$branches"))) +m) &&
    git checkout $(echo "$branch" | sed "s/.* //" | sed "s#remotes/[^/]*/##")
}

# fzf search and open in vlc
function fzf-play() {
  fzf \
    --ansi \
    --multi \
    --exact \
    --bind 'enter:select-all+execute-silent(vlc {+})' \
    --bind 'alt-enter:execute-silent(kitty @ launch --cwd $(dirname {}))' \
    --bind 'alt-A:select-all+execute(airflow-open {})' \
    --bind 'alt-a:select-all,alt-d:deselect-all,ctrl-/:toggle-preview' \
    --bind 'alt-e:select-all+execute(elmedia-open {})' \
    --bind 'alt-i:select-all+execute(iina-open {})' \
    --bind 'alt-l:execute(kitty @ launch --cwd $(dirname {}) lf)' \
    --bind 'alt-o:execute(open --reveal {})' \
    --bind 'alt-m:select-all+execute-silent(mpv --macos-fs-animation-duration=0 {+})' \
    --bind 'alt-p:select-all+execute-silent(printf "%s\n" {+} > playlist.m3u)+abort' \
    --bind 'alt-s:select-all+accept' \
    --bind 'alt-v:execute-silent(vlc {})' \
    --bind 'alt-t:execute-silent(kitty @ launch --cwd $(dirname {}))' \
    --header 'enter:play-all | alt-m:mpv | alt-o:play-one | alt-i:iina | alt-e:elmedia | alt-f:finder | alt-A:airflow | alt-a:select-all | alt-d:deselect-all | alt-t:new-tab | alt-l:lf | alt-s: stdout | alt-p:playlist' \
    --color header:italic:47 \
    --query "$*" # $* treat all arguments as a single string
}

function git-log-fzf {
  git log --oneline --decorate --color | fzf --ansi --preview 'git show --color $(echo {} | cut -d" " -f1)'
}
alias gl="git-log-fzf"

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

# Function to open a new centered kitty window and run 'index-play-checked'
function kitty-popup-centered() {
  # Center coordinates for a 1920x1080 screen, adjust as needed
  SCREEN_WIDTH=1920
  SCREEN_HEIGHT=1080
  WINDOW_WIDTH=800
  WINDOW_HEIGHT=600

  CENTER_X=$(((SCREEN_WIDTH - WINDOW_WIDTH) / 2))
  CENTER_Y=$(((SCREEN_HEIGHT - WINDOW_HEIGHT) / 2))

  # Create a temporary command to run in the new kitty window
  TEMP_COMMAND="index-play-checked"

  # Open a new kitty window with specified geometry and run the command
  kitty @ new-window --cwd="$HOME" --title="Centered Window" \
    --override "initial_window_width=${WINDOW_WIDTH}px" \
    --override "initial_window_height=${WINDOW_HEIGHT}px" \
    --override "window_margin_width=${CENTER_X}px" \
    --override "window_margin_height=${CENTER_Y}px" \
    zsh -c "$TEMP_COMMAND"
}
