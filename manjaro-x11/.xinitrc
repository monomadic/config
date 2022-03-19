#!/bin/sh

#xsetroot -bg black
#xsetroot -solid #CC0000
hsetroot -solid black
hsetroot -fill Pictures/Wallpapers/PagedOut_001_wallpaper_10.png

if [ -d /etc/X11/init/xinitrc.d ]; then
  for f in /etc/X11/xinit/xinitrc.d/*; do
    [ -x "$f" ] && . "$f"
  done
  unset f
fi

xrdb -merge ~/.Xresources
picom & exec i3
