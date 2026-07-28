# ytform

yt-dlp downloader with a **live metadata form** — the Go/Bubble Tea sibling of
`config/zsh/bin/ytui`. Same preflight, same progress template, same filename
grammar; the difference is that title / actors / channel / origin / tags /
rating are real form fields you edit *while* the file downloads, instead of
ytui's one-modal-at-a-time keys.

```
ytform [yt-dlp args...] <url>      # URL last; the rest passes through to yt-dlp
ytform --porn https://…            # yt-dlp config aliases work as usual
```

## Keys

| key | action |
|---|---|
| tab / ↑↓ | move between fields |
| (typing) | edit the focused field |
| 0–5, ←→ | set rating (on the rating row) |
| ctrl+o | stream the growing `.part` in mpv |
| ctrl+p | pause / resume (SIGSTOP/SIGCONT) |
| esc | cancel the download |
| enter (when finished) | apply the form and exit |

## Behavior

- Untouched form → yt-dlp's own filename is kept. Any edit → the file is
  renamed to `Actor A, Actor B - [Channel] Title #tag (Origin) ★★★☆☆.ext`,
  the grammar `media-parse-filename-to-json` reads back.
- A channel equal to the first actor's name is dropped from the filename.
- Thumbnails share ytq's cache (`YT_DLP_THUMBCACHE_DIR`, default
  `~/.cache/ytq`) and render as chafa symbol art.
- Galleries / playlists are out of scope — use `ytui` for those.

## Build

```
setup/install/ytform.sh    # installs to ~/.bin
go test ./...
```

Runtime deps: `yt-dlp` (required); `mpv`, `chafa`, `lsof` (optional).
