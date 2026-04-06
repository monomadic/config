# ============================================================================
# Media Selection & Playback
# ============================================================================

typeset -ga DJ_VISUALS_PATHS
DJ_VISUALS_PATHS=("$ICLOUD_HOME/Movies/Visuals")

play-index() {
  
}

.play-tower-with-mount() {
  MEDIA_PATH="/Volumes/Tower/Movies/Porn"

  if [[ ! -d "/Volumes/Tower" ]]; then
     osascript -e 'tell application "Finder" to mount volume "smb://nom@m4.local/Tower"'
  fi

  if [[ ! -d "$MEDIA_PATH/.index" ]]; then
     ls-media --path "$MEDIA_PATH" --sort created > "$MEDIA_PATH/.index"
  fi

  mpv --playlist="$MEDIA_PATH/.index"
}

alias @play="mpv-play"
alias @select="fzf-select | mpv-play"

alias .select-and-play="fzf-select --color | mpv --playlist=-"

#
# ALIASES
#
alias fd-media=ls-media
alias .ls="ls-media"
alias .ls-sorted="ls-media --sort=created"
alias .ls-pwd="ls-media . ."
alias .ls-sorted-pwd=".ls-sorted . ."
alias .ls-local="fd-video . {${ICLOUD_HOME},${HOME},/Volumes/*}/Movies/Porn"

alias .play=".ls | mpv-play"
alias .play-all-sorted=".ls-sorted | mpv-play"
alias .play-pwd=" | mpv-play"
alias .play-pwd-sorted=".ls-pwd | mpv-play"
alias .play-local=".ls-local | mpv-play"

alias cat-indexes="cat $HOME/.indexes/*"
alias cat-index-tower="cat $HOME/.indexes/tower-porn"

alias .index-play="cat-indexes | mpv --playlist=-"
alias .index-play-tower="mpv --playlist=/Volumes/Tower/Movies/Porn/.index"
alias \%play-tower=".index-play-tower"
alias \%select-tower="cat /Volumes/Tower/Movies/Porn/.index | fzf-select | mpv-play --playlist=-"

# alias .index-create-tower="fd . /Volumes/Tower/Movies/Porn/ > $HOME/.indexes/Tower"
alias .index-select-tower="%select-tower"
alias .index-select-tower-masters="cat-index-tower | grep 'Masters' | .select-and-play"
alias .index-select-tower-downloads="cat-index-tower | grep 'Downloads' | .select-and-play"

.create-index() {
  echo "Building index for $1"
  ls-media --path $1 --sort created > $1/.index
}

.create-all-indexes() {
  .create-index /Volumes/Tower/Movies/Porn
  .create-index /Volumes/Tower/Movies/Porn/Downloads
  .create-index /Volumes/Tower/Movies/Porn/Masters
  .create-index /Volumes/Tower/Movies/Porn/Masters/Clips
  .create-index /Volumes/Tower/Movies/Porn/Masters/Clips/Full
  .create-index "$HOME/Tower/Movies/Porn"
  .create-index "$HOME/Tower/Movies/Porn/Downloads"
  .create-index "$HOME/Tower/Movies/Porn/Masters"
  .create-index "$HOME/Library/Mobile Documents/com~apple~CloudDocs/Movies/Visuals"
}
alias .update-all-indexes=.create-all-indexes

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
    ls-media --path "${INDEX_DIR}" --sort created > "${INDEX_DIR}/.index"
    create-index /Volumes/Tower/Movies/Porn/Downloads
  fi

  echo "Using index at $INDEX_DIR/.index"

  mpv --playlist="$INDEX_DIR/.index"
}



alias .select=".ls | fzf-select | mpv-play"
alias .select-all-sorted=".ls-all-sorted | fzf-select | mpv-play"
alias .select-pwd=".ls-pwd | fzf-select | mpv-play"
alias .select-pwd-sorted=".ls-pwd-sorted | fzf-select | mpv-play"

alias \%select="cat $HOME/.indexes/Tower | fzf-select | mpv --playlist=-"

alias .play-downloads="fd-video . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | mpv-play"
alias .play-local-downloads="fd-video . $HOME/Movies/Porn/Downloads | mpv-play"
alias .play-local-downloads-sorted="fd-video-sort . $HOME/Movies/Porn/Downloads(N) | mpv-play"

alias .select-local="fd-video . $LOCAL_MEDIA_PATHS | fzf-select | mpv-play"

alias .select-downloads="fd-video . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .select-local-downloads-sort="fd-video-sort . $HOME/Movies/Porn/Downloads $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .select-downloads-sort="fd-video-sort . /Volumes/*/Movies/Porn/Downloads $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .select-local-downloads="fd-video . $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .play-local-downloads-incomplete="mpv $HOME/Movies/Porn/Downloads/**/*.part"
alias .play-local-downloads-incomplete-sorted="mpv $HOME/Movies/Porn/Downloads/**/*.part"

alias .play-suki="ls-media --match-string #suki | mpv-play"
alias \#suki=.play-suki
alias .select-suki="ls-media --match-string #suki | fzf-select | mpv-play"
alias .play-60fps="ls-media --match-string 60fps | mpv-play"
alias \#60fps=.play-60fps
alias .play-4k60fps="ls-media --match-string 60fps --match-string 2160p | mpv-play"
alias .play-4k60fps-top="ls-media --match-string 60fps --match-string 2160p --match-string ★★★ | mpv-play"
alias .select-4k60fps-top="ls-media --match-string 60fps --match-string 2160p --match-string ★★★ | fzf-select | mpv-play"
alias .play-best="ls-media --match-string ★★★ | mpv-play"
alias \#★★★="ls-media --match-string ★★★ | mpv-play"
alias \#★★★★★="ls-media --match-string ★★★★★ | mpv-play"
alias .select-best="ls-media --match-string ★★★ | fzf-select | mpv-play"

alias \$=.select
alias \$.=.select-pwd

alias .play-clips="ls-media --match-string /Clips/ | mpv-play"
alias .select-clips="fd-clips | strip-slash | fzf-select | mpv-play"

alias .select-visuals="fd-video . {${ICLOUD_HOME},${HOME},/Volumes/*}/Movies/Visuals | fzf-select | mpv --playlist=-"
alias .play-visuals-bg-black="fd-video --regex "#bg-black" . {${ICLOUD_HOME},${HOME},/Volumes/*}/Movies/Visuals | mpv --playlist=-"
alias \#bg-black=".play-visuals-bg-black"

alias .select-external="fd-video . /Volumes/*/Movies/Porn | fzf-select | mpv-play"
alias .select-masters="fd-video . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | fzf-select | mpv-play"
alias .play-masters="fd-video . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | mpv-play"

alias .select-tower-downloads-queue="ls-media --path /Volumes/Tower/Movies/Porn/Downloads/_queue | mpv-select | mpv-play"
alias .play-tower-downloads-60fps="ls-media --match-string 60fps --path /Volumes/Tower/Movies/Porn/Downloads | mpv-play"
alias .play-tower-downloads-2160p="ls-media --match-string 2160p --path /Volumes/Tower/Movies/Porn/Downloads | mpv-play"
alias .play-tower-masters="ls-media --path=/Volumes/Tower/Movies/Porn/Masters | mpv-play"
alias .play-tower-masters-sorted="ls-media --path=/Volumes/Tower/Movies/Porn/Masters --sort created | mpv-play"
alias .play-tower-downloads="fd-video . /Volumes/Tower/Movies/Porn/Downloads | mpv-play"
alias .local-sorted="fd-video-sort . $LOCAL_MEDIA_PATHS | fzf-select | mpv-play"
alias .play-local-sorted="fd-video-sort . $LOCAL_MEDIA_PATHS | fzf-select | mpv-play"

# Media search shortcuts
alias @=".play"
alias @@=".play-sort"
alias @@@="setopt local_options null_glob && printf '%s\0' $~MEDIA_GLOBS | fzf-play --hide-path -0"
alias @unc="fd-video . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-play"
alias @towerlocal="fd-video . /Volumes/Tower/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-socket"
alias @unique='fd-video . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | awk -F/ '"'"'!seen[$NF]++'"'"' | mpv-socket'
alias @full-path="fd-video . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-socket"
alias @clips="fd --absolute-path --exact-depth=1 --color=never . /Volumes/*/Movies/Porn/Masters/Clips/*/(N) $HOME/Movies/Porn/Masters/Clips/*/(N) | mpv-socket"
alias @pwd="fd-video | mpv-socket"
alias @@@pwd="ls-media --absolute-path --print0 | mpv-select"
alias @loop="fselect-porn -0 | fzf-media-select --hide-path --tac | mpv-with-config -"
alias @pwd-sort="fselect-pwd-sort -0 | fzf-play --hide-path --tac"
alias @queue="fd-video --print0 . $HOME/Movies/Porn/Queue/(N) | mpv-select"
alias @tutorials="fd-video . $TUTORIALS_PATH | mpv-select"
alias @external=@volumes

alias mount-tower="open smb://nom@m4.local/Tower"
alias .mount-tower=mount-tower
alias unmount-tower="diskutil unmount /Volumes/Tower"

alias @masters-full="fd-video --print0 . /Volumes/*/Movies/Porn/Masters/Full(N) $HOME/Movies/Porn/Masters/Full(N) | fzf-play --hide-path -0"
alias @masters-clips="fd-video --print0 . /Volumes/*/Movies/Porn/Masters/Clips(N) $HOME/Movies/Porn/Masters/Clips(N) | fzf-play --hide-path -0"

alias fd-clips="fd --absolute-path --exact-depth=1 --color=never . /Volumes/*/Movies/Porn/Masters/Clips/*/(N) $HOME/Movies/Porn/Masters/Clips/*/(N)"


# detect available media paths
ls-media-paths-checked() {
  ls-media-paths | while IFS= read -r media_path; do
    [[ $media_path == *Backup* ]] && continue
    echo "$media_path"
  done
}

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
