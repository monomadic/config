on run {}
  -- build a proper PATH
  set home to POSIX path of (path to home folder)
  set customPath to "/opt/homebrew/bin:" & home & ".bin:" & home & ".zsh/bin:/usr/local/bin:/usr/bin:/bin"
  -- grab Finder selection
  tell application "Finder" to set sel to selection as alias list
  if sel = {} then return
  -- construct the shell command
  set cmd to "PATH=" & quoted form of customPath & ":$PATH rename-media"
  repeat with f in sel
    set cmd to cmd & " " & quoted form of POSIX path of f
  end repeat
  -- run it
  do shell script cmd
end run
