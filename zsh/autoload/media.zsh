# ============================================================================
# Media Selection & Playback
# ============================================================================

#
# ALIASES
# 
alias .play="ls-media | mpv-play"
alias .play-pwd="mpv ."
alias .play-pwd-new="fd-video-sort | mpv-play"
alias .select-pwd="fd-video . | fzf-select | mpv-play"
alias .select-pwd-new="fd-video-sort | fzf-select | mpv-play"

alias .play-downloads="mpv $HOME/Movies/Porn/Downloads/**/*.mp4"
alias .play-downloads-incomplete="mpv $HOME/Movies/Porn/Downloads/**/*.part"
alias @suki="ls-media --match-string #suki | mpv-play"

alias ..visuals="fd-visuals | mpv-socket"
alias ..clips="fd-clips | strip-slash | fzf-select | mpv-play"
alias ..volumes="fd-video . /Volumes/*/Movies/Porn | fzf-select | mpv-play"
alias ..masters="fd-video . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | fzf-select | mpv-play"
alias ..downloads="fd --extension=mp4 . $HOME/Downloads | fzf-select | mpv-play"
alias ..downloads-latest="fd-video-sort . $HOME/Movies/Porn/Downloads $HOME/Downloads | fzf-select | mpv-play"
alias ..tower-masters="fd-video . /Volumes/Tower/Movies/Porn/Masters | mpv-play"
alias ..tower-masters-new="fd-video-sort . /Volumes/Tower/Movies/Porn/Masters | mpv-play"
alias ..tower-downloads="fd-video . /Volumes/Tower/Movies/Porn/Downloads | mpv-play"
alias ..local="fd-video . $LOCAL_MEDIA_PATHS | mpv-socket"
alias ..local-sorted="fd-video-sort . $LOCAL_MEDIA_PATHS | mpv-socket"
alias .play-local-sorted="fd-video-sort . $LOCAL_MEDIA_PATHS | mpv-play"

# Media search shortcuts
alias @=".play"
alias @unc="fd-video . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-play"
alias @towerlocal="fd-video . /Volumes/Tower/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-socket"
alias @unique='fd-video . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | awk -F/ '"'"'!seen[$NF]++'"'"' | mpv-socket'
alias @full-path="fd-video . /Volumes/*/Movies/Porn/(N) $HOME/Movies/Porn/(N) | mpv-socket"
alias @@@="setopt local_options null_glob && printf '%s\0' $~MEDIA_GLOBS | fzf-play --hide-path -0"
alias @clips="fd --absolute-path --exact-depth=1 --color=never . /Volumes/*/Movies/Porn/Masters/Clips/*/(N) $HOME/Movies/Porn/Masters/Clips/*/(N) | mpv-socket"
alias @pwd="fd-video | mpv-socket"
alias @@@pwd="ls-media --absolute-path --print0 | mpv-select"
alias @by-created="ls-media --sort-created | mpv-socket"
alias @loop="fselect-porn -0 | fzf-media-select --hide-path --tac | mpv-with-config -"
alias @pwd-sort="fselect-pwd-sort -0 | fzf-play --hide-path --tac"
alias @queue="fd-video --print0 . $HOME/Movies/Porn/Queue/(N) | mpv-select"
alias @tutorials="fd-video . $TUTORIALS_PATH | mpv-select"
alias @external=@volumes

alias mount-tower="open smb://nom@m4.local/Tower"
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
