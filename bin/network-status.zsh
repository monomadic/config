#!/bin/zsh

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# NerdIcons
ICON_NETWORK_STATUS="󰀂" # Example: Satellite dish
ICON_WIFI="\uf1eb"      # WiFi icon
ICON_IP="\uf124"        # Network IP icon
ICON_CAPTIVE="\uf2d1"   # Portal icon

# Check if the network is up
if ping -q -c 1 -W 1 8.8.8.8 &>/dev/null; then
  network_status="${GREEN}UP${NC}"
else
  network_status="${RED}DOWN${NC}"
fi

# Get WiFi SSID
ssid=$(networksetup -getairportnetwork en0 | awk -F': ' '{print $2}')
if [[ -z "$ssid" || "$ssid" == "You are not associated with an AirPort network." ]]; then
  ssid="${RED}Not connected to WiFi${NC}"
else
  ssid="${GREEN}${ssid}${NC}"
fi

# Get current IP address
ip=$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null)
if [[ -z "$ip" ]]; then
  ip="${RED}No IP address${NC}"
else
  ip="${GREEN}${ip}${NC}"
fi

echo

# Check for a captive portal if the network is down but a WiFi SSID is present
if [[ "$network_status" == "${RED}DOWN${NC}" && "$ssid" != "${RED}Not connected to WiFi${NC}" ]]; then
  captive_portal_url=$(curl -I http://captive.apple.com 2>/dev/null | grep -i 'location' | awk '{print $2}' | tr -d '\r')
  if [[ -n "$captive_portal_url" ]]; then
    captive_portal_status="${YELLOW}Captive portal detected: ${BLUE}${captive_portal_url}${NC}"
  else
    captive_portal_status="${GREEN}No captive portal detected${NC}"
  fi
else
  captive_portal_status=""
fi

# Output status
echo -e "${CYAN}${ICON_NETWORK_STATUS} Network ${NC}\t${network_status}"
echo -e "${CYAN}${ICON_WIFI} SSID ${NC}\t\t${ssid}"
echo -e "${CYAN}${ICON_IP} IPv4 ${NC}\t\t${ip}"
[[ -n "$captive_portal_status" ]] && echo -e "\n${CYAN}${ICON_CAPTIVE} ${captive_portal_status}"
