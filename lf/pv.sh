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
        *.rar) unrar l "$1";;
        *.7z) 7z l "$1";;
        *.pdf) pdftotext "$1" -;;
        *.jpg) viu "$1";;
        *.jpeg) viu "$1";;
        *.png) viu "$1";;
        *.doc) catdoc < "$1";;
        *.docx) docx2txt < "$1";;
        *) bat --style=plain --paging=never --terminal-width="$2" --color=always --theme=dracula "$1"
    esac
fi