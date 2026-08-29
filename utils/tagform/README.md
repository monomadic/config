# tagform

A form-based metadata tagger for MP4/MOV — labelled fields with typed editors,
validation, enums, star rows and tag chips, instead of a list of key/value
strings. Replaces `config/zsh/bin/mp4-tui-tagger`.

- **[SPEC.md](SPEC.md)** — the design.
- **[docs/CONTAINER.md](docs/CONTAINER.md)** — what ffmpeg and exiftool
  *actually* write. Measured. Read this before changing the write path.

## Status

**Milestone 5 of 8: multi-file.** Probe → model → aggregate → typed controls →
verified write, across a whole selection. Edits stage until `w`, which shows a
plan to confirm; the original is only ever replaced by a result that has been
read back and checked.

Aggregation works like an mp3 tagger: a field that differs between files reads
`‹multiple›`, and is left alone unless you set it. `m` **merges** a list field —
the union of every file's values, first-seen order, folded case-insensitively —
which is the operation you actually want when tagging a batch and which none of
the scripts this replaces can do. Setting a `‹multiple›` field says how many
distinct values it is about to flatten, in the confirmation, before it happens.

```bash
cargo run -- FILE...                  # the form
cargo run -- --print-json FILE...     # the model, as JSON
cargo test
```

The form is **modal**. Select mode moves and commands; Edit mode types. That is
what frees the single-letter keys — `w` can mean write because in Select mode
nothing is listening for the letter w.

**Select** (default)

| key | |
|---|---|
| `j` / `k`, arrows, `tab` | move between fields (`g` / `G` first / last) |
| `enter` | edit the focused field |
| `w` | write staged edits (shows a plan first) |
| `m` | merge a list field across every file in the selection |
| `p` | inspector — per-file values for the focused field |
| `]` / `[` / `a` | next file / previous file / all files |
| `u` / `ctrl-r` / `r` | undo / redo / revert everything staged |
| `f` | toggle MOV faststart on the write (on by default) |
| `q` / `esc` | quit (asks if edits are staged) |

**Edit**

| key | |
|---|---|
| (type) | edit the field; `←` `→` cycle an enum or adjust a rating |
| `enter` | save and stop editing |
| `tab` / `shift-tab` | save and move to the next / previous field |
| `esc` | cancel this field's edit |
| `ctrl-c` | quit, from either mode |

The form paints its own chrome: a filled `tagform` badge heads the screen, every
field shows a coloured editable region whether or not it is focused, the focused
field is marked `▍` (`▶` while editing) and a staged one `●`, and a shortcut
strip along the bottom lists the keys that are live in the current mode. Colours
are true-colour throughout — the previous 16-colour `DarkGray` was so close to a
dark terminal background that long custom keys read as blank labels.

Controls: text, list chips, `#hashtags`, URL (validated, `not a URL: …`), a
0–5 star row, open enums (Genre, Type) and closed ones (Kind, stored as the
`stik` integer but shown as "Movie"), and dates.

Genre and Type are **not hardcoded** — they are parsed out of
`~/.config/yt-dlp/config`'s `--alias` lines, so adding an alias there adds a
dropdown value here. `Camera Footage` normalizes to `Footage`.

Two things it already does that the scripts it replaces could not: it reads XMP
and atoms together, so a `rename-footage` clip shows its people, location and
rating; and `--print-json` reports `ilst_lossy` — the fields on these files that
have no iTunes atom at all, i.e. exactly what `--compat ilst` would drop.

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
