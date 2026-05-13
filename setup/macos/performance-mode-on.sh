
#!/bin/bash

#
# Turns off power saving tricks
# - no app nap
#

sudo pmset -a powernap 0
sudo pmset -a usbwake 0
sudo pmset -a standby 0
sudo pmset -a tcpkeepalive 0
