#!/bin/zsh

# Declare and define the list of menu options
typeset -a menu_options
menu_options=(
  "1) Go to directory 'project1' and open 'main.rs' in Neovim"
  "2) Go to directory 'project2' and open 'index.html' in Neovim"
  "3) Go to directory 'project3' and open 'README.md' in Neovim"
  "4) Go to directory 'project4' and list files"
  "5) Print current date and time"
)

# Define the actions for each command
actions() {
  case "$1" in
  1) cd ~/projects/project1 && nvim main.rs ;;
  2) cd ~/projects/project2 && nvim index.html ;;
  3) cd ~/projects/project3 && nvim README.md ;;
  4) cd ~/projects/project4 && ls ;;
  5) date ;;
  *) echo "Invalid choice" ;;
  esac
}

# Display the menu and get the user's selection
selected=$(printf '%s\n' "${menu_options[@]}" | fzf --prompt="Select an action: ")

# Extract the selected command number
choice=$(echo "$selected" | awk '{print $1}' | tr -d ')')

# Execute the corresponding action
actions "$choice"
