function rsync-backup-media-to-babyblue {
  echo "FireBird1TB/Media/Porn		${du-sh /Volumes/FireBird1TB/Media/Porn/}"
  # rsync-archive /Volumes/FireBird1TB/Media/Porn/ /Volumes/BabyBlue2TB/Media/Porn/
  # echo "Backup successful."
}
alias @backup-media-to-babyblue

function rsync-clone-babyblue-to-firebird {
  rsync-archive /Volumes/BabyBlue2TB/Media/Porn/ /Volumes/FireBird1TB/Media/Porn/
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

# function rsync-backup-babyblue {
#   rsync-archive /Volumes/BabyBlue2TB/Media/Porn/ /Volumes/FireBird1TB/Media/Porn/
#   # check file count is the same
#   # run the indexer
#   index-all
#   echo "Backup successful."
# }
# alias @backup-babyblue=rsync-backup-babyblue
