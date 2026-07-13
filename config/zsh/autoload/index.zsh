# Indexing and offline searching for non-persistent volumes

mpv-stdin() {
  mpv-send play "$@"
}

mpv-play-local() {
  expand-paths $LOCAL_MEDIA_PATHS | mpv-send play
}
alias .local=mpv-play-local

grep-top() {
  grep -E '(?i)\[TOP\]|🎖️|\[\*\]|\#top'
}
alias fd-top="fd-video |grep-top"

grep-safe() {
  grep -v -E '#g(\.| |/)|#bi(\.| |/)|#unsafe(\.| |/)|#ts(\.| |/)'
}

grep-unsafe() {
  grep -E '#g(\.| |/)|#bi(\.| |/)|#unsafe(\.| |/)|#ts(\.| |/)'
}
