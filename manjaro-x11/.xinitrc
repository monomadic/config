#!/bin/sh

#xsetroot -bg black
#xsetroot -solid #CC0000
hsetroot -solid black
hsetroot -fill Pictures/Wallpapers/PagedOut_001_wallpaper_10.png

# if [ -d /etc/X11/init/xinitrc.d ]; then
#   for f in /etc/X11/xinit/xinitrc.d/*; do
#     [ -x "$f" ] && . "$f"
#   done
#   unset f
# fi
#nvidia-settings --load-config-only
#xrandr --setprovideroutputsource modesetting NVIDIA-0
xrandr --auto
#xrandr --dpi 96

xrdb -merge ~/.Xresources
#picom & exec i3
#exec i3
exec xfce4-session
