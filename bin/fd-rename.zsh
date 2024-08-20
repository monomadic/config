#!/bin/zsh

# Define color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if the correct number of arguments is provided
if [ "$#" -ne 3 ]; then
  echo "${YELLOW}Usage: $0 <directory> <oldstring> <newstring>${NC}"
  exit 1
fi

directory=$1
oldstring=$2
newstring=$3

# Change to the specified directory
cd "$directory" || exit 1

# Use fd with --fixed-strings and -0 options
fd --fixed-strings "$oldstring" --type f -0 | while IFS= read -r -d '' file; do
  # Use parameter expansion for string replacement
  newfile=${file//$oldstring/$newstring}

  # Prompt user for confirmation before renaming, using /dev/tty to ensure input works in loops
  echo "\n${BLUE}Rename:${NC}"
  echo "${YELLOW}${file:t}${NC}"
  echo "${GREEN}${newfile:t}${NC}"
  echo "${BLUE}Proceed? [y/n]${NC}"

  read -r reply </dev/tty

  if [[ $reply == [yY] ]]; then
    if [ ! -e "$newfile" ]; then
      mv -n -- "$file" "$newfile"
      echo "${GREEN}Renamed '${file:t}' to '${newfile:t}'${NC}"
    else
      echo "${RED}Skipped '${file:t}' because '${newfile:t}' already exists${NC}"
    fi
  else
    echo "${YELLOW}Skipped '${file:t}'${NC}"
  fi
done
