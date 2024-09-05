#compdef download-video

_download_video() {
  local -a commands
  commands=(
    'music-video:Download music video'
    'audio-only:Download audio only'
    'video-only:Download video without audio'
    'porn:Download adult content'
    'youtube:Download YouTube video'
  )

  _arguments \
    '1: :->command' \
    '*: :->args'

  case $state in
  command)
    _describe -t commands 'download-video commands' commands
    ;;
  args)
    case $words[2] in
    music-video | audio-only | video-only | porn | youtube)
      _message 'URL'
      ;;
    *)
      _message 'no more arguments'
      ;;
    esac
    ;;
  esac
}

compdef _download_video download-video
