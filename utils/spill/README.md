# spill

Copy files named on **stdin** into a target directory until either the input
runs dry or the drive is full — with a Bubble Tea TUI built around two neon
gradient progress bars.

```
fd . /Volumes/src -tf -0 | spill -0 --fill --verify hash /Volumes/backup
```

`spill` reads a list of file paths (one per line, or NUL-separated with `-0`,
pairing with `fd`/`find -print0`) and copies each into `TARGET_DIR`. It shows:

- **Current file** bar — percentage, live write speed, bytes done / total, ETA.
- **Drive fill** bar — used %, free space remaining, average write speed, and an
  estimated time to fill the target at that average.

When the terminal supports it, a **thumbnail** of the current media file is
rendered inline (via `chafa`; video frames extracted and cached with `ffmpeg`).

## Behaviour

- Reads paths incrementally, so it works on a live stream that is still being
  produced.
- Stops when the **next file won't fit**. With `--fill` it instead skips the
  misfit and keeps going, copying whatever still fits until the input ends.
- Each copy goes to a temp file in the destination, is `fsync`ed, then atomically
  renamed — a failed or interrupted copy never leaves a partial file behind.
- `F_NOCACHE` is set on both fds (macOS) so huge media copies don't evict the
  page cache and the measured throughput stays honest.
- Existing destination files are skipped unless `--force`.
- When stdout is **not** a terminal, it drops the TUI and prints terse status to
  stderr while echoing each copied destination path to stdout (so it pipes).

## Flags

| Flag | Meaning |
|---|---|
| `-0`, `--null` | Input paths are NUL-separated |
| `--fill` | Skip non-fitting files and keep going until the drive is full |
| `--verify size\|hash` | Verify each copy after writing (default: off). `hash` = xxhash64, re-read from the target |
| `--retry N` | Extra attempts after a failed copy/verify (default: 2) |
| `--reserve SIZE` | Keep SIZE free on the target, e.g. `1G`, `500M` (default: 0) |
| `--force` | Overwrite files that already exist in the target |
| `--modest` | Never render thumbnails |
| `-h`, `--help` | Help |

## Build

```
setup/install/spill.sh          # → ~/.bin/spill
```

Runtime deps are optional and only used for thumbnails: `chafa` (images) and
`ffmpeg` (video frame extraction). Without them, `--modest` behaviour is the
effective default.
