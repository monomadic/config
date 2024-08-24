#!/bin/zsh

# Define color codes
local RED='\033[0;31m'
local GREEN='\033[0;32m'
local YELLOW='\033[0;33m'
local BLUE='\033[0;34m'
local NC='\033[0m' # No Color

# Check if the correct number of arguments is provided
if [ "$#" -lt 3 ]; then
	echo "${YELLOW}Usage: batch_rename <files> <oldstring> <newstring>${NC}"
	return 1
fi

local oldstring="${@[-2]}"
local newstring="${@[-1]}"
local files=("${@:1:-2}")

# If no files found, exit
if [ ${#files[@]} -eq 0 ]; then
	echo "${YELLOW}No files specified${NC}"
	return 0
fi

# Display all files that will be renamed
echo "${BLUE}The following files will be renamed:${NC}"
for file in "${files[@]}"; do
	if [[ "$file" == *"$oldstring"* ]]; then
			local newfile=${file//$oldstring/$newstring}
			echo "${YELLOW}${file:t}${NC} -> ${GREEN}${newfile:t}${NC}"
	fi
done

# Prompt user for confirmation
echo "\n${BLUE}Do you want to proceed with renaming these files? [y/n]${NC}"
read -r reply

if [[ $reply == [yY] ]]; then
	for file in "${files[@]}"; do
			if [[ "$file" == *"$oldstring"* ]]; then
					local newfile=${file//$oldstring/$newstring}
					if [ ! -e "$newfile" ]; then
							mv -n -- "$file" "$newfile"
							echo "${GREEN}Renamed '${file:t}' to '${newfile:t}'${NC}"
					else
							echo "${RED}Skipped '${file:t}' because '${newfile:t}' already exists${NC}"
					fi
			fi
	done
	echo "${GREEN}Renaming process completed.${NC}"
else
	echo "${YELLOW}Renaming process cancelled.${NC}"
fi
