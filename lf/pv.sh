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
				*.chunk) kontakt-info "$1";;
				*.kontakt) kontakt-info "$1";;
				*.kon) kontakt-info "$1";;
				*.ens) ni-tree "$1"; ni-info "$1";;
				*.nbcl) ni-tree "$1"; ni-info "$1";;
				*.nbfx) ni-tree "$1"; ni-info "$1";;
				*.nbkt) ni-tree "$1"; ni-info "$1";;
				*.nkx) ni-tree "$1"; ni-info "$1";;
				*.nkg) ni-tree "$1"; ni-info "$1";;
				*.ncw) ni-tree "$1"; ni-info "$1";;
				*.nkr) ni-tree "$1"; ni-info "$1";;
				*.nkc) ni-tree "$1"; ni-info "$1";;
				*.nki) ni-tree "$1"; ni-info "$1";;
				*.nkb) ni-tree "$1"; ni-info "$1";;
				*.nkm) ni-tree "$1"; ni-info "$1";;
				*.ksd) ni-tree "$1"; ni-info "$1";;
				*.nkp) ni-tree "$1"; ni-info "$1";;
				*.nfm8) ni-tree "$1"; ni-info "$1";;
				*.mxfx) ni-tree "$1"; ni-info "$1";;
				*.mxsnd) ni-tree "$1"; ni-info "$1";;
				*.mxgrp) ni-tree "$1"; ni-info "$1";;
				*.mxinst) ni-tree "$1"; ni-info "$1";;
				*.nmsv) ni-tree "$1"; ni-info "$1";;
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
