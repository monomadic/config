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
        *.jpg) chafa --format=symbols --size=60 "$1";;
        *.jpeg) chafa --format=symbols --size=60 "$1";;
        *.gif) chafa --format=symbols --size=60 "$1";;
        *.png) chafa --format=symbols --size=60 "$1";;
        # *.jpg) chafa --format=symbols --size=60 "$1";;
        # *.jpeg) chafa --format=symbols --size=60 "$1";;
        # *.gif) chafa --format=symbols --size=60 "$1";;
        # *.png) chafa --format=symbols --size=60 "$1";;
        *.doc) catdoc < "$1";;
        *.docx) docx2txt < "$1";;
        *) bat --style=plain --paging=never --terminal-width="$2" --tabs=2 --color=always --theme="Visual Studio Dark+" "$1"
    esac
fi
