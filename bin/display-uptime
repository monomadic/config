#!/bin/zsh

# Fetch uptime data
uptime_info=$(uptime | sed 's/.*up *//; s/,[^,]*user.*//; s/ *$//' | sed 's/^ *//; s/ *$//')

# Parse days and hours
if [[ $uptime_info == *day* ]]; then
  days=$(echo $uptime_info | awk '{print $1}')
  hours=$(echo $uptime_info | awk '{print $3}')
  hours=${hours%%:*} # Remove minutes if present
  output="$days days, $hours hours"
elif [[ $uptime_info == *:* ]]; then
  hours=$(echo $uptime_info | awk '{print $1}' | cut -d':' -f1)
  output="$hours hours"
else
  output="$uptime_info"
fi

# Define NerdFont icon and colors
clock_icon=$'\uf017' # Clock icon from NerdFonts
blue='%F{blue}'
green='%F{green}'
reset='%f'

# Print the formatted output
print -P "%F{yellow}${clock_icon}${green}  ${output}${reset}"
