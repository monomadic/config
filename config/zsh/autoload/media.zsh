# ============================================================================
# Media Selection & Playback
# ============================================================================

mpv-compare() {
  if [[ $# -ne 2 || $1 == (-h|--help) ]]; then
    echo "Usage: mpv-ab <file1> <file2>"
    echo ""
    echo "Play file1 and file2 in A/B test mode, comparing the two against each other."
    echo ""
    echo "Arguments:"
    echo "  file1  First file (usually original)"
    echo "  file2  Second file (usually new master)"
    return ${${(M)$#:#0}:+1}  # return 1 if wrong arg count, 0 if --help
  fi

  mpv --profile=ab --external-file="$1" "$2"
}

typeset -ga DJ_VISUALS_PATHS
DJ_VISUALS_PATHS=("$ICLOUD_HOME/Movies/Visuals")

.play-tower-with-mount() {
  MEDIA_PATH="/Volumes/Tower/Movies/Porn"

  if [[ ! -d "/Volumes/Tower" ]]; then
     osascript -e 'tell application "Finder" to mount volume "smb://nom@m4.local/Tower"'
  fi

  if [[ ! -d "$MEDIA_PATH/.index" ]]; then
     fd-media --path "$MEDIA_PATH" --sort created > "$MEDIA_PATH/.index"
  fi

  mpv-send play < "$MEDIA_PATH/.index"
}

alias @play="mpv-send play"
alias @select="fzf-select | mpv-send play"
alias .select-and-play="fzf-select --color | mpv-send play"
alias .keyframe-cut="ffmpeg-lossless-cut-by-fzf-keyframe-select --force"
alias .batch="yt-dlp-porn-batch-file /Users/nom/Library/Mobile\ Documents/com\~apple\~CloudDocs/Sync/links.txt"
#
# ALIASES
#
alias .trim="ffmpeg-lossless-cut-by-fzf-keyframe-select"
alias .trim-end="ffmpeg-lossless-cut-by-fzf-keyframe-select --reverse"

alias .ls="fd-media"
alias .ls-sorted="fd-media --sort=created"
alias .ls-pwd="fd-media . ."
alias .ls-sorted-pwd=".ls-sorted . ."
alias .ls-local="fd-media . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Porn(N)"

# alias .play="fd-media --print0 . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Porn(N) | mpv-send play -0"
alias .play="fd-media --print0 . {${HOME},/Volumes/*}/Movies/Porn(N) | mpv-send play -0"
alias play="fd-media --print0 . . | mpv-send play -0"
alias .play-new="fd-media --sort created --print0 . . | mpv-send play -0"
alias .play-new-local="fd-media --sort created --print0 . $HOME/Movies/Porn | mpv-send play -0"

# alias .select="fd-media --print0 . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Porn(N) | fzf-select -0 --print0 --stream | mpv-send play -0"
alias .select="fd-media --print0 . {${HOME},/Volumes/*}/Movies/Porn(N) | fzf-select -0 --print0 --stream | mpv-send play -0"

alias .play-all-sorted=".ls-sorted | mpv-send play"
alias .play-pwd="fd-media --print0 . . | mpv-send play -0 && mpv-send sort && mpv-send goto 1"
alias .pwd=.play-pwd
alias .play-local=".ls-local | mpv-send play"
alias .play-tower="fd-media --print0 . /Volumes/Tower/Movies/Porn(N) | mpv-send play -0"
alias .play-tower-downloads="fd-media --print0 . /Volumes/Tower/Movies/Porn/Downloads(N) | mpv-send play -0"
alias .play-tower-masters="fd-media --print0 . /Volumes/Tower/Movies/Porn/Masters(N) | mpv-send play -0"
alias .play-tower-clips="fd-media --print0 . /Volumes/Tower/Movies/Porn/Clips(N) | mpv-send play -0"
alias .play-tower-clips-landscape="fd-media --print0 . /Volumes/Tower/Movies/Porn/Clips/Landscape(N) | mpv-send play -0"

.play-tower-downloads-indexed() {
  MOUNT_PATH="/Volumes/Tower"
  MEDIA_PATH="/Movies/Porn/Downloads"
  INDEX_DIR="${MOUNT_PATH}${MEDIA_PATH}"
  SMB_URL="smb://nom@m4.local/Tower"

  if [[ ! -d "$MOUNT_PATH" ]]; then
    echo "Mounting $SMB_URL..."
    osascript -e 'tell application "Finder" to mount volume "$SMB_URL"'
  fi

  echo "$SMB_URL mounted."

  if [[ ! -d "$INDEX_DIR/.index" ]]; then
    echo "Index not found at ${INDEX_DIR}/.index"
    echo "Creating new index..."
    fd-media --path "${INDEX_DIR}" --sort created > "${INDEX_DIR}/.index"
    create-index /Volumes/Tower/Movies/Porn/Downloads
  fi

  echo "Using index at $INDEX_DIR/.index"

  mpv-send play < "$INDEX_DIR/.index"
}

filter-icloud-files() {  
  while IFS= read -r -d '' f; do
    status="$(mdls -raw -name kMDItemDownloadedDate "$f" 2>/dev/null || true)"
    [[ "$status" != "(null)" && -n "$status" ]] && print -r -- "$f"
  done
}

alias .select="fd-media --print0 . {${HOME},/Volumes/*}/Movies/Porn(N) | fzf-select -0 --print0 --stream | mpv-send play -0"
alias .select-all-sorted=".ls-sorted | fzf-select | mpv-send play"
alias .select-pwd=".ls-pwd | fzf-select | mpv-send play"
alias .select-pwd-sorted=".ls-pwd-sorted | fzf-select | mpv-send play"

alias \%select="cat $HOME/.indexes/Tower | fzf-select | mpv-send play"

alias .play-downloads="fd-media --print0 . {/Users/nom/Library/Mobile\ Documents/com~apple~CloudDocs,/Users/nom,/Volumes/*}/Movies/Porn/Downloads(N) | mpv-send play -0"
alias .play-local-downloads="fd-media --print0 . ~/Movies/Porn/Downloads | mpv-send play -0"
alias .play-newest-local-downloads="setopt extendedglob && printf '%s\n' ~/Movies/Porn/Downloads/**/*.part~*Frag*(N.om) | mpv-send play"
alias .play-local-incomplete-downloads="fd -t f -g '*.part' -E '*Frag*.part' . ~/Movies/Porn/Downloads | mpv-send play"

alias .select-local="fd-media . $LOCAL_MEDIA_PATHS | fzf-select | mpv-send play"

alias .select-downloads="fd-media . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"
alias .select-local-downloads-sort="fd-media --sort created . $HOME/Movies/Porn/Downloads $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"
alias .select-downloads-sort="fd-media --sort created . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"
alias .select-local-downloads="fd-media . $HOME/Movies/Porn/Downloads | fzf-select | mpv-send play"

alias .play-suki="fd-media --match-string #suki | mpv-send play"
alias \#suki=.play-suki
alias .select-suki="fd-media --match-string #suki | fzf-select | mpv-send play"
alias .play-60fps="fd-media --match-string 60fps | mpv-send play"
alias \#60fps=.play-60fps
alias .play-4k60fps="fd-media --match-string 60fps --match-string 2160p | mpv-send play"
alias .play-4k60fps-top="fd-media --match-string 60fps --match-string 2160p --match-string ★★★ | mpv-send play"
alias .select-4k60fps-top="fd-media --match-string 60fps --match-string 2160p --match-string ★★★ | fzf-select | mpv-send play"
alias .play-best="fd-media --match-string ★★★ | mpv-send play"
alias \#★★★="fd-media --match-string ★★★ | mpv-send play"
alias \#★★★★★="fd-media --match-string ★★★★★ | mpv-send play"
alias .select-best="fd-media --match-string ★★★ | fzf-select | mpv-send play"

alias \$=.select
alias \$.=.select-pwd

alias .play-clips="fd-media --match-string /Clips/ | mpv-send play"
alias .select-clips="fd-clips | strip-slash | fzf-select | mpv-send play"

alias .select-visuals="fd-media . {${ICLOUD_HOME},${HOME},/Volumes/*}/Movies/Visuals(N) | fzf-select | mpv-send play"

alias .play-visuals="fd-media --print0 . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Visuals(N) | mpv-vj play -0"
alias .play-visuals-bg-black="fd-media . {${HOME}/Library/Mobile\ Documents/com~apple~CloudDocs,${HOME},/Volumes/*}/Movies/Visuals(N) | grep '#bg-black' | mpv-vj play"
alias \#bg-black=".play-visuals-bg-black"

alias .select-external="fd-media . /Volumes/*/Movies/Porn(N) | fzf-select | mpv-send play"
alias .select-masters="fd-media . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | fzf-select | mpv-send play"
alias .play-masters="fd-media . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | mpv-send play"

alias .select-tower-downloads-queue="fd-media --path /Volumes/Tower/Movies/Porn/Downloads/_queue | fzf-select | mpv-send play"
alias .play-tower-downloads-60fps="fd-media --match-string 60fps --path /Volumes/Tower/Movies/Porn/Downloads | mpv-send play"
alias .play-tower-downloads-2160p="fd-media --match-string 2160p --path /Volumes/Tower/Movies/Porn/Downloads | mpv-send play"
alias .play-tower-masters="fd-media --path=/Volumes/Tower/Movies/Porn/Masters | mpv-send play"
alias .play-tower-masters-sorted="fd-media --path=/Volumes/Tower/Movies/Porn/Masters --sort created | mpv-send play"
alias .play-tower-downloads="fd-media . /Volumes/Tower/Movies/Porn/Downloads | mpv-send play"
alias .local-sorted="fd-media --sort created . $LOCAL_MEDIA_PATHS | fzf-select | mpv-send play"
alias .play-local-sorted="fd-media --sort created . $LOCAL_MEDIA_PATHS | fzf-select | mpv-send play"

# Media search shortcuts
alias @=".play"
alias @@=".play-sort"
alias @@@="setopt local_options null_glob && printf '%s\0' $~MEDIA_GLOBS | fzf-select -0 --print0 --color | mpv-send play -0"
alias @unc="fd-media . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-send play"
alias @towerlocal="fd-media . /Volumes/Tower/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-send play"
alias @unique='fd-media . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | awk -F/ '"'"'!seen[$NF]++'"'"' | mpv-send play'
alias @full-path="fd-media . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-send play"
alias @clips="fd --absolute-path --exact-depth=1 --color=never . /Volumes/*/Movies/Porn/Masters/Clips/*/(N) $HOME/Movies/Porn/Masters/Clips/*/(N) | mpv-send play"
alias @pwd="fd-media . . | mpv-send play"
alias @@@pwd="fd-media --absolute-path --print0 | fzf-select -0 --print0 | mpv-send play -0"
alias @loop="fselect-porn -0 | fzf-select -0 --print0 --tac | mpv-vj play -0 --shuffle"
alias @pwd-sort="fselect-pwd-sort -0 | fzf-play --hide-path --tac"
alias @queue="fd-media --print0 . $HOME/Movies/Porn/Queue/(N) | fzf-select -0 --print0 | mpv-send play -0"
alias @tutorials="fd-media . $TUTORIALS_PATH | fzf-select | mpv-send play"
alias @external=@volumes

alias mount-tower="open smb://nom@m4.local/Tower"
alias .mount-tower=mount-tower
alias unmount-tower="diskutil unmount /Volumes/Tower"

# list all unique tags found in files under the present directory
fd-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

fd-creators() {
  setopt +o nomatch
  fd -td -d1 . {/Volumes/*/Movies/Porn/*/creators,$HOME/Movies/Porn/*/creators} 2>/dev/null
}

fzf-creators() {
  fd-creators | while read -r path; do
    name=${path%/}   # Remove trailing slash
    name=${name##*/} # Get last component
    echo "$name	$path"
  done | fzf --exit-0 --with-nth=1 --delimiter="\t" --preview="ls '{2}'" | cut -f2
}

cd-creators() {
  cd $(fzf-creators)
}

media-cache-clear() {
  cd $LOCAL_CACHE_PATH && rm -rf **/*
}
