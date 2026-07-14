# ============================================================================
# File Operations
# ============================================================================

alias rm="rm -i"
alias df="df -h"


alias tag=rename-media
alias .tag=tag
alias rn="batch-rename"
alias ren="batch-rename"
alias .rename="fd-rename-all.zsh"

alias trash-undo="rip --unbury"
alias trash-view="rip --seance"

alias .dupes-check="fdupes --recurse --cache --nohidden --size ."
alias .dupes-delete="fdupes --recurse --cache --nohidden --size --delete ."
alias .dupes-delete-interactive="fdupes --recurse --deferconfirmation --cache --nohidden --size --plain ."
alias .list-moved-files="fclones group --cache --hash-fn metro --isolate --dry-run"

# ============================================================================
# Python/Development
# ============================================================================

alias .python-venv-create="python3 -m venv .venv && source .venv/bin/activate"
alias .python-venv-activate="source .venv/bin/activate"
alias .python-pip-install-requirements="pip install -r requirements.txt"


# ============================================================================
# macOS Specific
# ============================================================================

alias .restart-window-server="sudo killall -HUP WindowServer"
alias .macos-keybindings="source $DOTFILES_DIR/setup/macos/keybindings.sh"
alias .gatekeeper-whitelist="xattr -rd com.apple.quarantine"
alias .self-sign="codesign --sign - --force --deep"
get-app-id() { osascript -e "id of app \"$1\""; }
alias .screen-sharing-kick-users="sudo /System/Library/CoreServices/RemoteManagement/ARDAgent.app/Contents/Resources/kickstart -deactivate -restart -users current"
alias passwordless-reboot="sudo fdesetup authrestart"
alias .clear-notifications="killall NotificationCenter"
alias battery='pmset -g batt'

topaz-video() {
  local binary
  binary="$(topaz_resolve_gui_binary)" || return
  [[ -x "$binary" ]] || {
    print -u2 "Topaz GUI binary not found or not executable: $binary"
    topaz_app_help >&2
    return 1
  }
  env LC_ALL=C LC_NUMERIC=C LANG=C "$binary" "$@"
}
alias .topaz-video=topaz-video
alias .brave-mp4-support="/Applications/Brave\ Browser.app/Contents/MacOS/Brave\ Browser --disable-features=MediaSource,UseModernMediaControls"

alias xdg-open=open
alias o=open
alias tab="open"

# ============================================================================
# Network & System
# ============================================================================

alias .network-detect-captive-portal=detect-captive-portal
alias .network-status=ns
alias .portal=detect-captive-portal
alias .detect-captive-portal=detect-captive-portal
alias ns="network-status.zsh"
alias .uptime="display-uptime"

alias ls-network-interfaces='for i in $(ifconfig -l); do ip=$(ipconfig getifaddr $i); [ -n "$ip" ] && echo "$i -> $ip"; done'

alias p8="ping 8.8.8.8"
alias pc="ping cloudflare.com"
alias pg="ping google.com"

alias ls-usb="system_profiler SPUSBDataType"
alias ls-usb-ioreg="ioreg -p IOUSB -w0"
alias ls-disks="diskutil list"
