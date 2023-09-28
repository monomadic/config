#!/bin/sh

if file --mime "$1" | grep "executable" >/dev/null
then
    file -b "$1"
    echo
    rabin2 -I "$1"
else
    case "$1" in
        *.tar*) tar tf "$1";;
        *.zip) unzip -l "$1";;
        *.epub) unzip -l "$1";;
        *.rar) unrar l "$1";;
        *.7z) 7z l "$1";;
        *.pdf) file "$1" -;;
        # *.jpg) chafa-select  $1;;
        # *.jpeg) chafa-select $1;;
        # *.gif) chafa-select  $1;;
        # *.png) chafa-select  $1;;
        *.jpg) chafa --size=$2x$3 --format=symbols "$1";;
        *.jpeg) chafa --size=$2x$3 --format=symbols "$1";;
        *.gif) chafa --size=$2x$3 --format=symbols "$1";;
        *.png) chafa --size=$2x$3 --format=symbols "$1";;
        *.doc) catdoc < "$1";;
        *.docx) docx2txt < "$1";;
				*.tga) file "$1";;
				*.wav) file "$1";;
				*.nkx) ni-info "$1";;
				*.nkg) ni-info "$1";;
				*.ncw) ni-info "$1";;
				*.nkr) ni-info "$1";;
				*.nkc) ni-info "$1";;
				*.nki) ni-info "$1";;
				*.nkm) ni-info "$1";;
				*.ksd) ni-info "$1";;
				*.nkp) ni-info "$1";;
				*.nfm8) ni-info "$1";;
				*.mxfx) ni-info "$1";;
				*.nmsv) ni-info "$1";;
				*.mp4)
					ffmpeg -y -i "$1" -vframes 1 -ss 5 "/tmp/lf-thumbnail.png"
					chafa --size=$2x$3 --format=symbols "/tmp/lf-thumbnail.png"
					;;
				*.mov)
					ffmpeg -y -i "$1" -vframes 120 "/tmp/lf-thumbnail.png"
					chafa --size=$2x$3 --format=symbols "/tmp/lf-thumbnail.png"
					;;
        *) bat --style=plain --paging=never --terminal-width="$2" --tabs=2 --color=always --theme="Visual Studio Dark+" "$1";;
    esac
fi
