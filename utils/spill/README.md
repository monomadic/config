# spill

Copy files named on **stdin** into a target directory until either the input
runs dry or the drive is full — with a Bubble Tea TUI built around two neon
gradient progress bars.

```
fd . -tf -0 | spill -0 -s highest-quality --fill --verify hash /Volumes/backup
```

`spill` reads a list of file paths (one per line, or NUL-separated with `-0`,
pairing with `fd`/`find -print0`) and copies them into `TARGET_DIR` — by
default all of them, in the order given. An opt-in **strategy** narrows that
down to the files worth taking. It shows:

- **Current file** bar — percentage, live write speed, bytes done / total, ETA.
- **Drive fill** bar — used %, free space remaining, average write speed, and an
  estimated time to fill the target at that average.

When the terminal supports it, a **thumbnail** of the current media file is
rendered inline (via `chafa`; video frames extracted and cached with `ffmpeg`).

## Strategies

`--strategy` (`-s`) picks *which* files are copied and in *what order*. It
defaults to `none` — every file, in the order given, straight to the copy loop
with nothing inspected and nothing sorted, which is also the fastest thing
spill can do. The three quality tiers read the `★★★☆☆` rating out of the filename (the
convention `media-set-rating` writes) and the resolution / frame rate with
`ffprobe`; each bar is a hard filter, and what clears it is copied best-first
(rating, then resolution, then frame rate). Unrated files and files with no
video stream never clear a quality bar.

| Strategy | Takes | Order |
|---|---|---|
| `none` *(default)* | everything | as given |
| `high-quality` | 3★+, 1080p+, 60fps+ | best first |
| `highest-quality` | 4★+, 4K+, 60fps+ | best first |
| `good-quality` | 3★+, 1080p+, 30fps+ | best first |
| `latest` | everything | newest `mtime` first |
| `audit` | files with a black intro, or an MP4/MOV whose `moov` sits after `mdat` | as they arrive |
| `url-missing` | files with no embedded source URL (or one set to `none`) | as they arrive |

`audit` is [media-audit](../../config/zsh/bin/media-audit)'s black-frames and
faststart checks — same atom walk, same `blackdetect` thresholds — except that
instead of asking what to do about a problem file, it copies it. `url-missing`
looks at the same tag list media-audit does: `source_url`, `webpage_url`,
`purl`, `url`, `comment`, `description`.

The sorting strategies drain stdin and inspect everything before the first
copy (probing runs four files at a time, with a progress line). The streaming
ones judge each path as it arrives, so they still work on a list that is being
produced live.

## Behaviour

- Relative input paths **keep their structure** under the target:
  `b/clip.mp4` lands at `TARGET_DIR/b/clip.mp4`, creating directories as
  needed. Absolute input paths have nowhere sensible to nest, so they always
  land in the target root — as does everything under `--flatten`.
- Stops when the **next file won't fit**. With `--fill` it instead skips the
  misfit and keeps going, copying whatever still fits until the input ends.
- Each copy goes to a temp file in the destination, is `fsync`ed, then atomically
  renamed — a failed or interrupted copy never leaves a partial file behind.
- `F_NOCACHE` is set on both fds (macOS) so huge media copies don't evict the
  page cache and the measured throughput stays honest.
- Reads and writes are **pipelined**: a reader goroutine fills a ring of three
  16 KiB-aligned 8 MiB buffers while the writer drains it, so neither drive
  idles waiting on the other. With `F_NOCACHE` and a serial loop, throughput
  lands at the harmonic mean of read and write speed instead of at the slower
  of the two — a fast source feeding a slow target gives away roughly a third
  of the write bandwidth. `--verify hash` is hashed on the reader side, so the
  checksum rides along with a write that was happening anyway.
- The destination is preallocated with `F_PREALLOCATE`, which keeps a
  multi-gigabyte file from landing in fragments and turns "the drive is full"
  into an error before the first byte rather than six gigabytes in.
- Existing destination files are skipped — never a failure — unless `--force`.
- When stdout is **not** a terminal, it drops the TUI and prints terse status to
  stderr while echoing each copied destination path to stdout (so it pipes).

## Flags

| Flag | Meaning |
|---|---|
| `-s`, `--strategy S` | Which files to copy, and in what order (default: `none`) |
| `-0`, `--null` | Input paths are NUL-separated |
| `--fill` | Skip non-fitting files and keep going until the drive is full |
| `--flatten` | Copy everything into the target root, ignoring input structure |
| `--verify size\|hash` | Verify each copy after writing (default: off). `hash` = xxhash64, re-read from the target |
| `--retry N` | Extra attempts after a failed copy/verify (default: 2) |
| `--reserve SIZE` | Keep SIZE free on the target, e.g. `1G`, `500M` (default: `1G`) |
| `--force` | Overwrite files that already exist in the target |
| `--modest` | Never render thumbnails |
| `-h`, `--help` | Help |

## Build

```
setup/install/install-spill.sh          # → ~/.bin/spill
```

The quality strategies and `url-missing` need `ffprobe`, and `audit` needs
`ffmpeg`; spill refuses to start without the one it needs. `none` and `latest`
need neither. `chafa` (images) and
`ffmpeg` (video frames) are optional and only used for thumbnails — without
them, `--modest` behaviour is the effective default.
