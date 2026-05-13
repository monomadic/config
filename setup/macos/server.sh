# disable sleep
sudo systemsetup -setcomputersleep Never
sudo pmset -a disablesleep 1

# ssh remote login
sudo systemsetup -setremotelogin on

# smb
sudo launchctl load -w /System/Library/LaunchDaemons/com.apple.smbd.plist
