# run 'brew bundle'

# note: add https://github.com/wader/fq

tap "homebrew/bundle"
tap "homebrew/services"
#tap "homebrew/cask-fonts"
tap "rschmitt/heatseeker"

# localsend (airdrop alternative)
# tap "localsend/localsend"
# brew "localsend"

cask_args appdir: "/Applications"

# make macos default tools behave more like unix/linux
brew "coreutils"
brew "findutils"
brew "gnu-sed"
brew "gawk"
brew "gnu-tar"
brew "gnu-which"
brew "grep"

brew "mas"

# git
brew "git"
brew "gh" # github cli tool
brew "git-delta" # rust git diff
brew "ghq" # git repo management https://github.com/x-motemen/ghq
brew "lazygit"
# brew "degit" # git cloner (note: not on brew)

brew "gzip"
brew "shfmt" # formatter for zsh/sh/etc

brew "dotter" # rust dotfiles manager
# brew "homebrew/dotter.rb"
# brew "chezmoi" # go-based dotfiles manager

brew "rm-improved" # rm replacement (rip)
# brew "clear" # regex file delete
# brew "most" # more replacement (use less -R)
brew "bat"		# cat replacement
brew "mdless" # markdown viewer
brew "glow"		# markdown viewer golang
brew "bk"			# ebook reader
brew "eza"		# ls/exa replacement (rust)
brew "lsd"		# ls replacement (rust)
brew "duf"		# disk usage go
brew "dust"		# better ncdu in rust
brew "dua-cli" # another ncdu rust
brew "ncdu"		# disk usage
brew "fzf"		# fuzzy filter (go)
brew "sk"			# fuzzy filter (rust)
brew "heatseeker" # fuzzy filter for small menus (rust)
brew "jq"			# json tool
brew "jless"	# json tree viewer
brew "bottom" # htop / sysperf monitor
brew "fclones" # rust duplicate finder
brew "fdupes" # another rust dupe finder
brew "czkawka" # gui dupe finder
brew "starship" # cli prompt in rust

cask "journey" # diary

# video tools
brew "mp4v2" # mp4 tools like mp4info etc
cask "losslesscut" # lossless slicing of videos (mp4, webm, etc)
cask "qlvideo" # quicklook/finder preview and info panel for unsupported videos (webm, mkv, etc)

brew "blueutil" # bluetooth util

# file manager
brew "yazi", args: ["HEAD"] # rust based
brew "xplr" # rust file explorer
brew "lf" # ranger, but in c
brew "joshuto" # ranger, but in rust. better than lf.

brew "broot" # lists tree output

brew "poppler" # pdf renderer
brew "unar" # unarchiver

brew "rclone" # disk clone (cloud)
brew "rg" # ripgrep grep replacement (rust)
brew "rga" # ripgrep-all (search pdf, zip etc)
brew "fd" # find
brew "neovim" # editor
brew "helix" # editor (rust)
brew "amp" # editor (rust)
brew "kakoune" # editor
brew "youtube-dl"
brew "ffmpeg" # for youtube-dl
brew "ffmpegthumbnailer"
brew "wget"
brew "bandwhich" # bandwidth monitor
brew "diskonaut" # disk usage
brew "zellij" # tmux replacement
brew "chafa" # sixel
brew "viu" # sixel

brew "fdupes" # file duplicates

# video
brew "mpv"
brew "vapoursynth" # frame interpolation for mpv

brew "procs" # ps replacement (rust)

brew "ouch" # general decompressor (rust)

# llm / openai
tap "simonw/llm"
brew "ttok" # token counter
brew "strip-tags" # strip tags from html with gpt
brew "llm" # cli tool to interact with large language models
# brew "lm-studio"
# brew "ollama"
brew "aichat" # rust-based client for many llm platforms

cask "megasync" # mega.nz

# encryption
brew "age"
brew "ssss" # shamirs secret sharing scheme (multikey)

# development
#	brew "cargo-nextest" # testing NEVER INSTALL THIS, brew should never manage uust EVERQ
brew "cloc" # loc
brew "prettier"
brew "tokei" # loc
brew "pastel" # colors
brew "aicommits" # gpt generated commit messages

# yabai + skhd
# tap "koekeishiya/formulae"
# tap "jorpilo/formulae" # replacement for koekeishiya
# brew "koekeishiya/formulae/skhd" # brew services start skhd
# brew "koekeishiya/formulae/yabai"

# tabs
# tap "austinjones/taps"
# brew "tab"

# mas "", id: 1521432881 # session pomodoro timer

# atuin (history db)
tap "ellie/atuin"
brew "atuin"

# network
brew "rustscan" # port scanner
brew "trippy" # network diagnostic `sudo trip crates.io`

# cask "airpass" # MAC address renewal for wifi

# fonts
cask "font-ark-pixel-10px-monospaced"
cask "font-ark-pixel-12px-monospaced"
cask "font-ark-pixel-16px-monospaced"
cask "font-hack"
cask "font-hack-nerd-font"
cask "font-gohufont-nerd-font"
cask "font-symbols-only-nerd-font"
brew "font-jetbrains-mono-nerd-font"

cask "kitty" # term
cask "alacritty" # term
cask "1password" # password manager
cask "1password-cli"
cask "protonvpn"
cask "proton-mail"
cask "proton-drive"
# cask "hammerspoon"

cask "coconutbattery" # battery info

cask "openinterminal" # opens current finder folder in terminal

# cask "cheatsheet" # show a cheat sheet by holding ⌘

cask "smooze-pro"

# utils
# cask "daisydisk"
brew "rnr" # rust based rename
cask "transnomino" # gui renamer
brew "mmv" # go based util for renaming with vim

# bitcoin
cask "bluewallet"
cask "sparrow"
cask "ledger-live"

# browser
cask "firefox"
cask "brave-browser"
cask "mullvad-browser" # firefox-based privacy browser

# music
cask "spotify"

cask "numi" # calculator
cask "caffeine" # sleep prevention

# messaging
cask "signal"
cask "telegram-desktop"
cask "session", target: 'Session Desktop.app' # signal fork with private keys instead of phone numbers
cask "whatsapp"

# reversing
brew "binwalk" # binary analyser
brew "bingrep" # binary analyser
brew "radare2" # debugger / decompiler
cask "cutter" # debugger / compiler
cask "corretto" # jdk alternative by amazon
cask "ghidra" # debugger / decompiler
brew "rizin" # fork of radare2
cask "temurin" # jre replacement (for ghidra)
brew "bitwise" # bit conversion
brew "demumble" # demangling
cask "hex-fiend" # hex editor

brew "aria2" # cli download client

cask "airflow" # airplay streamer

# manga
tap "metafates/mangal"
brew "mangal" # tui comic downloader

# things.app shell access
tap "AlexanderWillner/tap"
brew "things.sh"
