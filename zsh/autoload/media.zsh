# ============================================================================
# Media Selection & Playback
# ============================================================================

typeset -ga DJ_VISUALS_PATHS
DJ_VISUALS_PATHS=("$ICLOUD_HOME/Movies/Visuals")

alias @play="mpv-play"
alias @select="fzf-select | mpv-play"

#
# ALIASES
# 
alias .play="ls-media | mpv-play"
alias .play-new="ls-media --sort-created | mpv-play"
alias .pwd-play="ls-media --path . | mpv-play"
alias .pwd-select="ls-media --path . | fzf-select | mpv-play"
alias .pwd-select-latest="ls-media --path . --sort-created | fzf-select| mpv-play"
alias .select="ls-media | fzf-select | mpv-play"
alias .select-latest="ls-media --sort-created | fzf-select| mpv-play"
alias .play-pwd-new="fd-video-sort | mpv-play"
alias .select-pwd-new="fd-video-sort | fzf-select | mpv-play"
alias .play-local="fd-video . $LOCAL_MEDIA_PATHS | mpv-play"
alias .select-local="fd-video . $LOCAL_MEDIA_PATHS | fzf-select | mpv-play"
alias .play-downloads="fd-video . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | mpv-play"
alias .select-downloads="fd-video . /Volumes/*/Movies/Porn/Downloads(N) $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .play-downloads-latest="fd-video-sort . $HOME/Movies/Porn/Downloads $HOME/Movies/Porn/Downloads | mpv-play"
alias .select-downloads-latest="fd-video-sort . $HOME/Movies/Porn/Downloads $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .play-local-downloads="fd-video . $HOME/Movies/Porn/Downloads"
alias .select-local-downloads="fd-video . $HOME/Movies/Porn/Downloads | fzf-select | mpv-play"
alias .play-local-downloads-incomplete="mpv $HOME/Movies/Porn/Downloads/**/*.part"

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

alias .play-clips="ls-media --match-string /Clips/ | mpv-play"
alias .select-clips="fd-clips | strip-slash | fzf-select | mpv-play"

alias .play-visuals='fd-video . "${DJ_VISUALS_PATHS[@]}" | mpv-play'
alias .select-visuals="fd-video . "${DJ_VISUALS_PATHS[@]}" | fzf-select | mpv-play"
alias .select-visuals="fd-visuals | fzf-select | mpv-play"

alias .play-visuals-bg-black='fd-video --regex "#bg-black" . "${DJ_VISUALS_PATHS[@]}" | fzf-select | mpv-play'

alias .select-external="fd-video . /Volumes/*/Movies/Porn | fzf-select | mpv-play"
alias .select-masters="fd-video . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | fzf-select | mpv-play"
alias .play-masters="fd-video . /Volumes/*/Movies/Porn/Masters(N) $HOME/Movies/Porn/Masters(N) | mpv-play"
alias .play-tower-masters="fd-video . /Volumes/Tower/Movies/Porn/Masters | mpv-play"
alias .play-tower-masters-new="fd-video-sort . /Volumes/Tower/Movies/Porn/Masters | mpv-play"
alias .play-tower-downloads="fd-video . /Volumes/Tower/Movies/Porn/Downloads | mpv-play"
alias .local-sorted="fd-video-sort . $LOCAL_MEDIA_PATHS | fzf-select | mpv-play"
alias .play-local-sorted="fd-video-sort . $LOCAL_MEDIA_PATHS | fzf-select | mpv-play"

# Media search shortcuts
alias @=".play"
alias @@=".play-new"
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
