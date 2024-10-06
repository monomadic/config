#compdef media

_media() {
  local line state

  _arguments -C \
    "1: :->cmds" \
    "*::arg:->args"

  case "$state" in
  cmds)
    _values "media command" \
      "play[Play videos]" \
      "search[Search videos]" \
      "list[List videos]"
    ;;
  args)
    case $line[1] in
    play)
      _values -s ' ' "play options" \
        "clips[Match videos in /clips/]" \
        "originals[Match videos in /originals/]" \
        "latest[Sort by latest]" \
        "--shuffle[Shuffle videos during playback]" \
        "*:custom arg"
      ;;
    search | list)
      _values -s ' ' "search/list options" \
        "clips[Match videos in /clips/]" \
        "originals[Match videos in /originals/]" \
        "*:custom arg"
      ;;
    esac
    ;;
  esac
}

compdef _media media
