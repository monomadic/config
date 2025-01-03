#compdef media

# Define all commands
local -a commands
commands=(
  'play:Play videos'
  'search:Search videos'
  'list:List videos'
)

# Define video types
local -a video_types
video_types=(
  'clips:Match videos in /clips/'
  'originals:Match videos in /originals/'
  'library:Match videos in /clip-library/'
  'loops:Match videos in /loops/'
  'scenes:Match videos in /scenes/'
  'top-clips:Match top-rated clips'
  'latest:Sort by creation date'
)

# Define options
local -a options
options=(
  '--shuffle:Shuffle videos during playback'
  '--start-random:Start playback from a random position'
  '--verbose:Enable verbose output'
)

# Main completion function
_media() {
  local curcontext="$curcontext" state line
  typeset -A opt_args

  # Define argument structure
  _arguments -C \
    '1: :->command' \
    '*: :->args'

  case $state in
  command)
    # Complete first argument with commands
    _describe -t commands 'media commands' commands
    ;;
  args)
    case $line[1] in
    play)
      # For play command, allow video types, options, and tags
      if [[ ${words[CURRENT]} == \#* ]]; then
        # If current word starts with #, don't complete
        return
      elif [[ ${words[CURRENT]} == \[* ]]; then
        # If current word starts with [, don't complete
        return
      else
        local -a play_args
        play_args=(
          $video_types
          $options
        )
        _describe -t arguments 'play arguments' play_args
      fi
      ;;
    search | list)
      # For search and list commands, allow video types and tags
      if [[ ${words[CURRENT]} == \#* ]]; then
        # If current word starts with #, don't complete
        return
      elif [[ ${words[CURRENT]} == \[* ]]; then
        # If current word starts with [, don't complete
        return
      else
        local -a search_list_args
        search_list_args=(
          $video_types
          '--verbose:Enable verbose output'
        )
        _describe -t arguments 'arguments' search_list_args
      fi
      ;;
    esac
    ;;
  esac
}

# Register the completion function
compdef _media media
