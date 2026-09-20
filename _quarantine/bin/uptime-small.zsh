#!/bin/zsh

local uptime=$(uptime | awk '{print $3,$4,$5}' | sed 's/,//g')
local icon="\uf8c4"          # Nerd Font icon for clock
local color="\033[38;5;039m" # Light blue color
local reset="\033[0m"

echo "${color}${icon} Uptime: ${uptime}${reset}"
