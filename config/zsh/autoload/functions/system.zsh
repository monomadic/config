macos-disable-wifi-power-management() {
  # Check current state
  sudo pmset -g | grep proximitywake

  # Disable WiFi power management (persists across reboots)
  sudo pmset -a proximitywake 0

  # Also worth trying:
  sudo pmset -a tcpkeepalive 1
}

disk-speed-test() {
  dd if="$tmp" of=/dev/null bs=1024k
  rm -f "$tmp"
}

# Show negotiated Ethernet link speed
get-eth-speed() {
  local iface

  iface=$(networksetup -listallhardwareports |
    awk '/Hardware Port: Ethernet|Hardware Port: USB 10\/100\/1000 LAN|Hardware Port: Thunderbolt Ethernet/{getline; print $2; exit}')

  [[ -z $iface ]] && {
    echo "No Ethernet interface found"
    return 1
  }

  ifconfig "$iface" | awk '
    /media:/ {
      match($0, /[0-9]+(baseT|GbaseT)/)
      print substr($0, RSTART, RLENGTH)
    }'
}

diff-filenames() {
  emulate -L zsh
  setopt pipefail

  if [[ $# -ne 2 ]]; then
    print "Usage: diffnames <dir1> <dir2>" >&2
    return 1
  fi

  local left=$1 right=$2
  local red=$'%F{red}'
  local green=$'%F{green}'
  local reset=$'%f'

  comm -3 \
    <(cd "$left"  && fd -t f --strip-cwd-prefix | sort) \
    <(cd "$right" && fd -t f --strip-cwd-prefix | sort) \
  | while IFS= read -r line; do
      if [[ $line == *$'\t'* ]]; then
        # right side (missing in left)
        print -P "${green}${line//$'\t'/}${reset}"
      else
        # left side (missing in right)
        print -P "${red}${line}${reset}"
      fi
    done
}

columns() {
  emulate -L zsh
  local cols=2
  if [[ $# -ge 1 && $1 = <-> ]]; then
    cols=$1
    shift
  fi

  if [[ $# -eq 0 && -t 0 ]]; then
    print "Usage: columns [cols] <file...> (or pipe input)"
    return 1
  fi

  if [[ $# -gt 0 ]]; then
    pr -t -"$cols" -w "$(tput cols)" "$@"
  else
    pr -t -"$cols" -w "$(tput cols)"
  fi
}
