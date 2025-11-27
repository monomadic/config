# run 'brew bundle'

# note: add https://github.com/wader/fq

tap "homebrew/bundle"
tap "homebrew/services"
tap "rschmitt/heatseeker"
tap "sumoduduk/tap" # terminusdm
tap "nijaru/tap" # sy

# localsend (airdrop alternative)
cask "localsend"

cask_args appdir: "/Applications"

# make macos default tools behave more like unix/linux
brew "coreutils"
brew "findutils"
brew "gnu-sed"
brew "gawk"
brew "gnu-tar"
brew "gnu-which"
brew "grep"
brew "uni" # unicode db
brew "sy" # rsync replacement

brew "m-cli" # system config tool

# cask "hazeover" # dim non-focus windows

brew "sd" # find and replace

brew "karabiner-elements" # keyboard shortcuts

#brew "default-handler" # Utility for changing default URL scheme handlers
# brew "screen"

brew "f3" # flash fraud detection

# manage app store
brew "mas"

# brew "clamxav" # antivirus
# brew "lynis" # antivirus
brew "powermetrics" # benchmarking, profiling

cask "monitorcontrol" # independent controls for each monitor

cask "jdownloader" # download manager
# cask "motrix" # download manager
cask "terminusdm"

cask "swift-shift" # window manager with mouse

brew "flac" # flac support
cask "nifty-file-lists" # metadata gui for various files


# git
brew "git"
brew "gh" # github cli tool
brew "git-delta" # rust git diff
brew "ghq" # git repo management https://github.com/x-motemen/ghq
brew "lazygit"
# brew "degit" # git cloner (note: not on brew)

cask "alt-tab" # better alt-tab switcher

brew "gzip"
brew "shfmt" # formatter for zsh/sh/etc

brew "marta" # dual pane file manager

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
# cask "calibre" # ebook converter
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

brew "mpv", tap: "homebrew/cask/stolendata-mpv"

# brew "battop", tap: "indigoviolet/tap" # battery info
# brew "battop", url: "https://raw.githubusercontent.com/indigoviolet/homebrew-tap/main/Formula/battop.rb"
# tap "indigoviolet/tap"
# brew "battop"

cask "journey" # diary

# video tools
brew "mp4v2" # mp4 tools like mp4info etc
cask "losslesscut" # lossless slicing of videos (mp4, webm, etc)
# cask "vidcutter" # qt5 based cutter and jointer
cask "qlvideo" # quicklook/finder preview and info panel for unsupported videos (webm, mkv, etc)
cask "djv" # video review / ab testing
cask "iina" # mac gui for mpv

brew "blueutil" # bluetooth util

# file manager
brew "yazi", args: ["HEAD"] # rust based
brew "xplr" # rust file explorer
brew "lf" # ranger, but in c
brew "fff" # file manager
brew "nnn" # tui file manager
brew "joshuto" # ranger, but in rust. better than lf.

brew "broot" # lists tree output

brew "poppler" # pdf renderer
brew "unar" # unarchiver

brew "rclone" # disk clone (cloud)
brew "rg" # ripgrep grep replacement (rust)
brew "rga" # ripgrep-all (search pdf, zip etc)
brew "fd" # find
brew "fselect" # sql like find
brew "neovim" # editor
brew "helix" # editor (rust)
brew "amp" # editor (rust)
#brew "kakoune" # editor

# brew "youtube-dl" # deprecated
brew "yt-dlp"

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
# brew "stolendata-mpv" # version with more gui-like integration
brew "vapoursynth" # frame interpolation for mpv

brew "procs" # ps replacement (rust)

brew "ouch" # general decompressor (rust)
brew "zstd" # facebook zip compression (zstandard)

# llm / openai
cask "claude" # claude.ai gui
tap "simonw/llm"
brew "ttok" # token counter
brew "strip-tags" # strip tags from html with gpt
brew "llm" # cli tool to interact with large language models
# brew "lm-studio"
# brew "ollama"
brew "aichat" # rust-based client for many llm platforms

cask "megasync" # mega.nz

# macos desktop
# cask "hiddenbar" # hides menu items (bartender is better)
cask "bartender" # hides menu items
cask "jordanbaird-ice" # menubar manager (like bartender, open source)
cask "topnotch" # makes the menu bar entirely black
cask "notchnook" # menu under notch

# encryption
brew "age"
brew "ssss" # shamirs secret sharing scheme (multikey)

brew "there" # display local times of friends in any time zone
brew "menuwhere" # global drill-down menu https://manytricks.com/menuwhere/

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
cask "font-jetbrains-mono-nerd-font"

cask "kitty" # term
cask "alacritty" # term
cask "1password" # password manager
cask "1password-cli"
cask "protonvpn"
cask "proton-mail"
cask "proton-drive"
# cask "hammerspoon" # lua script automation for macos

cask "coconutbattery" # battery info

cask "openinterminal" # opens current finder folder in terminal

# cask "cheatsheet" # show a cheat sheet by holding ⌘

cask "smooze-pro"

# utils
# cask "daisydisk"
#
# FILE RENAMING
brew "rnr" # rust based rename
cask "transnomino" # macos native gui renamer
# brew "mmv" # go based util for renaming with vim
brew "moreutils" # includes vidir, for renaming with vi
brew "advanced-renamer"

brew "wallpaper" # manage desktop wallpaper

# bitcoin wallets
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
# cask "chai" # sleep prevention (using caffeinated from MAS)
cask "keepingyouawake" # sleep prevention
cask "stats" # menu stats (like iStat)

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
cask "temurin" # jre replacement (for ghidra)
cask "ghidra" # debugger / decompiler
brew "rizin" # fork of radare2
brew "bitwise" # bit conversion
# brew "demumble" # demangling
cask "hex-fiend" # hex editor
brew "hevi" # tui hex viewer

brew "aria2" # cli download client

cask "airflow" # airplay streamer

cask "aichat" # chatgpt etc

# manga
# tap "metafates/mangal"
# brew "mangal" # tui comic downloader

# things.app shell access
# tap "AlexanderWillner/tap"
# brew "things.sh"
