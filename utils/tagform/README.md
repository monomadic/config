# tagform

A form-based metadata tagger for MP4/MOV — labelled fields with typed editors,
validation, enums, star rows and tag chips, instead of a list of key/value
strings. Replaces `config/zsh/bin/mp4-tui-tagger`.

- **[SPEC.md](SPEC.md)** — the design.
- **[docs/CONTAINER.md](docs/CONTAINER.md)** — what ffmpeg and exiftool
  *actually* write. Measured. Read this before changing the write path.

## Status

**Milestone 1 of 8: read-only.** Probe → model → aggregate → JSON. No UI, no
writes.

```bash
cargo run -- --print-json FILE...
cargo test
```

Milestone 0 (the container experiment) is done and reshaped the design; its
findings are in `docs/CONTAINER.md` and reproducible with
`tests/container-experiment.sh`.

## The three things to know

**1. This library is `mdta`, not iTunes.** `config/yt-dlp/config` sets
`-movflags use_metadata_tags` globally, so tags live in `moov/udta/meta` under
the `mdta` handler with arbitrary key names. The default ffmpeg path writes
iTunes `ilst` atoms instead and **silently drops** `actors`, `type`, `channel`,
`rating`, `origin`, `source_url`, `webpage_url`, `purl` and `yt_dlp_id` — 9 of
20 keys. The two boxes are mutually exclusive.

**2. `rename-footage` puts everything in XMP, and ffprobe cannot see it.**
People, tags, channel, location, rating and `PreservedFileName` are XMP written
by exiftool. A reader using ffprobe alone concludes a footage file has no
metadata at all. So `tagform` always runs both readers.

**3. An ffmpeg remux destroys XMP.** Totally, silently, with no flag to prevent
it. That is why the writer chooses its backend from the file's *contents*, not
from a user preference — and why `--writer ffmpeg` on a file carrying XMP
requires `--force`.

## Dependencies

`ffmpeg`/`ffprobe` and `exiftool`, both required. `assets/tagform.exiftool.cfg`
is a required runtime asset: without it exiftool refuses to write this repo's
custom `Keys:` tags (`Sorry, Keys:Actors doesn't exist or isn't writable`) —
the same wall `rename-footage` hit before it retreated to XMP.
