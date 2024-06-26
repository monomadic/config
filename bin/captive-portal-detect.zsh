#!/bin/zsh

# Function to get the captive portal login URL
get_captive_portal_url() {
  local url=$(curl -s -I -L -o /dev/null -w '%{url_effective}' http://captive.apple.com/hotspot-detect.html)
  echo $url
}

# Function to check if a captive portal is present and get its URL
check_captive_portal() {
  local portal_url=$(get_captive_portal_url)

  if [[ $portal_url != "http://captive.apple.com/hotspot-detect.html" ]]; then
    echo $portal_url
    return 0 # Captive portal detected
  else
    return 1 # No captive portal
  fi
}

# Function to open the captive portal login page in the default browser
open_captive_portal_login() {
  local login_url=$1
  echo "Captive portal detected. Opening login page in default browser..."
  open "$login_url"
}

# Main script
echo "Checking for captive portal..."

portal_url=$(check_captive_portal)

if [[ $? -eq 0 ]]; then
  open_captive_portal_login "$portal_url"
else
  echo "No captive portal detected."
fi
