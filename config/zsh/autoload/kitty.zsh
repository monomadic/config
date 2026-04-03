#!/bin/zsh

tab_exists() {
  local name="$1"
  kitty @ ls | jq -e --arg name "$name" \
    '.tabs[] | select(.title == $name)' >/dev/null 2>&1
}

edit-kitty-config() {
  if tab_exists "  kitty.conf"; then
    echo "Tab already exists — skipping"
  else
    echo "Launching new tab..."
    kitty @ launch --type=tab --tab-title "MyTab" zsh
  fi
}

switch_or_launch_tab() {
  local name="$1"
  local tab_id

  # Find tab ID by partial title match
  tab_id=$(kitty @ ls |
    jq -r --arg name "$name" '
      .[] | .tabs[] | select(.title | contains($name)) | .id' | head -n1)

  if [[ -n $tab_id ]]; then
    kitty @ focus-tab --match id:"$tab_id"
  else
    kitty @ launch --type=tab --tab-title "$name" zsh
  fi
}

switch_or_launch_tab_exact() {
  local name="$1"
  local tab_id

  # Find tab ID by name
  tab_id=$(kitty @ ls |
    jq -r --arg name "$name" '
      .[] | .tabs[] | select(.title == $name) | .id' | head -n1)

  if [[ -n $tab_id ]]; then
    # Focus the existing tab
    kitty @ focus-tab --match id:"$tab_id"
  else
    # Launch a new one
    kitty @ launch --type=tab --tab-title "$name" zsh
  fi
}
