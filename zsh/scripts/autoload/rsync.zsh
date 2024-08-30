function rsync-backup-media-to-babyblue {
  echo "FireBird1TB/Media/Porn		${du-sh /Volumes/FireBird1TB/Media/Porn/}"
  # rsync-archive /Volumes/FireBird1TB/Media/Porn/ /Volumes/BabyBlue2TB/Media/Porn/
  # echo "Backup successful."
}

function rsync-clone-babyblue-to-firebird {
  rsync-archive /Volumes/BabyBlue2TB/Movies/Porn/ /Volumes/FireBird1TB/Movies/Porn/
}

function rsync-archive {
  # Check for proper number of arguments
  if [ $# -ne 2 ]; then
    echo "Usage: ${0:t} <source_folder> <destination_folder>"
    return 1
  fi

  # Variables for source and destination
  local source_folder="$1"
  local destination_folder="$2"

  # Check if source folder exists
  if [ ! -d "$source_folder" ]; then
    echo "Error: Source folder does not exist."
    return 1
  fi

  # Check if the destination directory exists; create if it does not
  if [ ! -d "$destination_folder" ]; then
    echo "Destination folder does not exist. Creating it now..."
    mkdir -p "$destination_folder"
  fi

  # Perform rsync to backup the folder
  rsync --archive \
    --progress \
    --ignore-existing \
    --delete \
    "$source_folder" \
    "$destination_folder"

  # Check if rsync succeeded
  if [ $? -eq 0 ]; then
    echo "Backup completed successfully."
  else
    echo "Backup failed."
    return 1
  fi
}

function rsync-from-stdio {
  # Check for proper number of arguments
  if [ $# -ne 1 ]; then
    echo "Usage: ${0:t} <destination_folder>"
    return 1
  fi

  # Variable for destination
  local destination_folder="$1"

  # Check if the destination directory exists; create if it does not
  if [ ! -d "$destination_folder" ]; then
    echo "Destination folder does not exist. Creating it now..."
    mkdir -p "$destination_folder"
  fi

  # Create a temporary file to store the list of files
  local temp_file=$(mktemp)

  # Read the list of files from stdin and store in the temporary file
  cat >"$temp_file"

  # Create an array to store the list of files
  local files_to_sync=()

  # Read each line from the temporary file
  while IFS= read -r file; do
    # Check if the file exists
    if [ -f "$file" ]; then
      files_to_sync+=("$file")
    else
      echo "Warning: File not found: $file"
    fi
  done <"$temp_file"

  # Perform rsync to copy the files
  rsync --archive \
    --progress \
    --delete \
    --include-from="$temp_file" \
    --exclude='*' \
    "${files_to_sync[@]}" \
    "$destination_folder"

  # Check if rsync succeeded
  if [ $? -eq 0 ]; then
    echo "Sync completed successfully."
  else
    echo "Sync failed."
    rm "$temp_file"
    return 1
  fi

  # Clean up the temporary file
  rm "$temp_file"
}

# function rsync-backup-babyblue {
#   rsync-archive /Volumes/BabyBlue2TB/Media/Porn/ /Volumes/FireBird1TB/Media/Porn/
#   # check file count is the same
#   # run the indexer
#   index-all
#   echo "Backup successful."
# }
# alias @backup-babyblue=rsync-backup-babyblue
