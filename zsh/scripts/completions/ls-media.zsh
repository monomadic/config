#compdef ls-media

_ls-media() {
  _arguments -s \
    '--match-string[Only list files where the path contains all the given strings (case-insensitive). Can be used multiple times]:string: ' \
    '--match-regex[Only list files where the path matches all the given regex patterns (case-insensitive). Can be used multiple times]:regex: ' \
    '--sort[Sort files by the given option (modified, size, name)]:sort-option:(modified size name)' \
    '--reverse[Reverse the sort order and the final list]' \
    '--verbose[Enable verbose logging]' \
    '*:filename:_files'
}

compdef _ls-media ls-media