zmodload zsh/mapfile

# -- Run broot, cd into pathfile if successful --
# Depends: zmapfile
br () {  # [<broot-opt>...]
  emulate -L zsh

  local pathfile=$(mktemp)
  trap "rm ${(q-)pathfile}" EXIT INT QUIT
  if { broot --verb-output "$pathfile" $@ } {
    if [[ -r $pathfile ]] {
      local folder=${mapfile[$pathfile]}
      if [[ $folder ]]  cd $folder
    }
  } else {
    return
  }
}
