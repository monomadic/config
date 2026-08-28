# tagform — a form-based metadata tagger for MP4/MOV

**Status:** design document / implementation prompt. No code yet.
**Language:** Rust. **Install target:** `~/.local/bin/tagform` via `setup/install/install-tagform.sh`.

---

## 1. What this is

`tagform` is a full-screen terminal application that edits container metadata on
`.mp4` / `.m4v` / `.mov` files through **real form controls** — labelled fields
with typed editors, validation, enums, star rows, tag chips and checkboxes —
instead of a list of key/value strings.

```
tagform ~/Movies/**/*.mp4              # multi-file, aggregated like an mp3 tagger
tagform --fetch clip.mp4               # seed from yt-dlp using the embedded URL
tagform --from-filename clip.mp4       # seed from the filename grammar
tagform --set genre=Karaoke --apply *.mov   # headless
```

It replaces [`config/zsh/bin/mp4-tui-tagger`](../../config/zsh/bin/mp4-tui-tagger),
which has the right *staging model* (nothing hits disk until `w`, multi-file
aggregation with `<multiple values>`) but the wrong *interface*: an fzf list
where every edit shells out to `$EDITOR` and every value is an untyped string.
A rating is not a string. A URL is not a string. A tag list is not a string.

Three existing things define its shape:

| Existing | What `tagform` takes from it |
|---|---|
| `mp4-tui-tagger` | staging model, multi-file value aggregation, write-on-`w` |
| [`utils/ytform`](../ytform) | the live form over title/actors/channel/origin/tags/rating, and the kitty-placeholder thumbnail |
| [`media-audit`](../../config/zsh/bin/media-audit) | thumbnail cache + cover-fit crop, faststart handling, the yt-dlp metadata fetch |

The difference from `ytform` is the direction of travel: `ytform` edits metadata
for a file being *downloaded*, and the output is a filename. `tagform` edits
metadata on files that already *exist*, and the output is atoms inside the
container (with the filename as an optional secondary sink).

### Non-goals

- Not a transcoder. The only ffmpeg invocation is `-c copy`.
- Not a library manager. It does not walk, index or organise; feed it paths
  (`fd-media`, `media-paths`, `fzf-media-select` already do that).
- Not a container repair tool. Fragmentation and moov placement stay
  [`mp4doctor`](../../config/zsh/bin/mp4doctor)'s job; `tagform` only rides the
  faststart flag along on the remux it is already doing.
- No Matroska, no audio-only formats. If it grows, `.mkv` goes through a
  separate backend, not by pretending MKV tags are MP4 atoms.
- Not a chapter or subtitle editor (see §14, deferred).

---

## 2. The container problem — read this before designing anything else

MP4 and MOV store "the same" metadata in **four** mutually incompatible places,
and which one you write decides which applications can read the file back. All
of the behaviour below was measured, not assumed — see
[docs/CONTAINER.md](docs/CONTAINER.md) for the fixtures and the numbers.

```
moov/udta/meta/hdlr = 'mdir' + ilst   →  iTunes-style four-char atoms (©nam, desc, keyw…)
                                          read by: Music/iTunes, Plex, Infuse, Jellyfin,
                                          AtomicParsley, mp4ameta, Emby
moov/udta/meta/hdlr = 'mdta' + keys/ilst → arbitrary-length string keys
                                          read by: QuickTime, Finder, AVFoundation, ffprobe,
                                          exiftool; keys can be any name at all
```

ffmpeg picks between them:

- **default** — `mdir`/ilst for `.mp4`/`.m4v`, plain `udta` for `.mov`. Only the
  keys ffmpeg has a mapping for survive; unknown keys are silently dropped.
- **`-movflags use_metadata_tags`** — `mdta`, every key preserved verbatim,
  **and the iTunes atoms are not written**.

The repo currently commits to `mdta` everywhere. `config/yt-dlp/config` says so
explicitly:

```
# let MP4 store non-standard tag names (actors, yt_dlp_*, source_url, ...)
--ppa "Metadata:-movflags use_metadata_tags"
```

and `media-embed`, `media-refresh-tags`, `media-audit` and `mp4doctor` all pass
`use_metadata_tags` on every rewrite. That is why `actors`, `source_url`,
`yt_dlp_id` and friends round-trip at all — and also why a file tagged by this
ecosystem shows up in Plex with no title.

### 2.1 Compatibility modes

`tagform` makes this an explicit setting rather than an accident, with three
modes:

| `--compat` | What is written | Use |
|---|---|---|
| `mdta` *(default)* | one ffmpeg pass with `use_metadata_tags`; every field, custom keys included | the house style — matches everything else in this repo |
| `ilst` | one ffmpeg pass without the flag; only fields with a real atom mapping (§4) | files headed for Plex/Infuse/Music |
| `both` | the `mdta` pass, then a second in-place injection of the ilst atoms (§9.3) | archival masters; the only mode readable by everything |

**Measured (milestone 0, done).** Of 20 keys written four ways:

| Container | Flags | Box | Kept |
|---|---|---|---|
| `.mp4` | *(default)* | `[ItemList]` ilst | 11/20 |
| `.mp4` | `use_metadata_tags` | `[Keys]` mdta | **20/20** |
| `.mov` | *(default)* | `[UserData]` udta | 9/20 |
| `.mov` | `use_metadata_tags` | `[Keys]` mdta | **20/20** |

ffmpeg writes one box **or** the other, never both — so `both` genuinely needs a
second tool (§9.3). The default `.mp4` path silently drops exactly this repo's
custom vocabulary: `actors`, `type`, `channel`, `rating`, `origin`,
`source_url`, `webpage_url`, `purl`, `yt_dlp_id`.

The `.mov` default path is not merely lossy, it is *wrong*: ffmpeg invents
unnamed atoms from the first three characters of keys it cannot map, so
`description` becomes `UserData_des` and `keywords` becomes `UserData_key`, and
nothing reads those back. **`--compat ilst` on a `.mov` input is a hard error**
naming the fields that would be lost.

One more measured consequence: with `-map_metadata 0`, `use_metadata_tags`
copies `major_brand`, `minor_version` and `compatible_brands` in as *real*
readable tags, which then accumulate on every rewrite. Hiding them from the form
(§3.6) is not enough — every write must actively clear them with
`-metadata major_brand=` and friends.

### 2.2 XMP — the fourth namespace, and the dangerous one

`rename-footage` stores **all** of its authored metadata as XMP via exiftool,
not as atoms at all. Two measured facts govern the entire write path:

- **ffprobe cannot see XMP.** A file carrying six XMP fields reports exactly the
  same `format_tags` as one carrying none.
- **An ffmpeg remux destroys XMP**, totally and silently, with no flag to
  prevent it. The `[Keys]` tags survive; the XMP does not.

So a `tagform` that always remuxed would erase everything `rename-footage`
authored — including `PreservedFileName`, which that script's comments call "the
only surviving copy" of a file's original name. The writer therefore chooses its
backend from the file's *contents*, never from a user preference (§9.2).

---

## 3. The field schema

A **field** is what the user sees: one label, one control, one value. A **key**
is what lands in the container. The relation is one-to-many — the URL field
writes five keys — and that fan-out is the whole reason this tool exists.

### 3.1 The primary fields

The ten the brief requires, plus the ones the yt-dlp config already produces and
would otherwise be silently dropped on every rewrite.

| # | Field | Control (§5) | Container keys (`mdta`) | ilst atom | Notes |
|---|---|---|---|---|---|
| 1 | **Title** | Text | `title` | `©nam` | title-case helper on `⌃T`, matching `media-parse-filename-to-json` |
| 2 | **Actors** | List (chips, `,`) | `actors`, `artist` | `©ART` + `iTunMOVI` cast | yt-dlp writes both from `%(cast,uploader)l` |
| 3 | **Artist** | Text | `artist` | `©ART` | separate from Actors; when Actors is non-empty and Artist is untouched it mirrors the joined list (§3.4) |
| 4 | **Rating** | Stars 0–5 | `rating`, `comment` JSON | freeform `com.apple.iTunes:rating` | see §3.3 — this is *not* `rtng` |
| 5 | **Description** | TextArea | `description` | `desc` (+ `ldes` if >255 B) | |
| 6 | **URL** | URL (validated) | `webpage_url`, `source_url`, `purl`, `comment`, `original_url` | `purl` | one field, five keys — §3.2 |
| 7 | **Channel** | Text + completion | `album_artist`, `album`, `channel` | `aART`, `©alb`, `tvnn` | yt-dlp maps `%(channel,uploader)s` to both `album_artist` and `album` |
| 8 | **Tags** | HashTag chips | `keywords` | `keyw` | comma-joined on disk, `#tag` in filenames |
| 9 | **Genre** | Enum (open) | `genre` | `©gen` | enum seeded from the yt-dlp aliases — §3.5 |
| 10 | **Type** | Enum (open) | `type` | — | `Clip` / `Master` / `Original`, from the yt-dlp aliases |
| 11 | **Kind** | Enum (closed) | `media_type` | `stik` | the iTunes media kind — §3.3 |

### 3.2 Secondary fields

Shown under a collapsed **More ▸** section (`⇥` past field 11, or `m`), so the
default screen stays the eleven above.

| Field | Control | Keys (`mdta`) | ilst |
|---|---|---|---|
| Date | Date (`YYYY-MM-DD`) | `date` | `©day` |
| Comment | TextArea | `comment` | `©cmt` |
| Synopsis | TextArea | `synopsis` | `ldes` |
| Composer | Text | `composer` | `©wrt` |
| Director | List | `director` | `iTunMOVI` |
| Producer | List | `producer` | `iTunMOVI` |
| Studio | Text | `studio` | `iTunMOVI` |
| Copyright | Text | `copyright` | `cprt` |
| Grouping | Text | `grouping` | `©grp` |
| Language | Enum (ISO 639-2) | `language` | — |
| Show | Text | `show` | `tvsh` |
| Season / Episode | Number ×2 | `season_number`, `episode_sort` | `tvsn`, `tves` |
| Episode ID | Text | `episode_id` | `tven` |
| Advisory | Enum (closed) | `advisory` | `rtng` |
| Content rating | Text | `content_rating` | `iTunEXTC` |
| Origin | Text | `origin` | — | the `(fh_881)` bracket in the filename grammar |

### 3.3 Three different things called "rating"

This trips up every MP4 tagger and the schema must keep them apart:

1. **Stars, 0–5.** The user's own convention. Lives in the filename as a
   trailing ` ★★★☆☆` (`media-set-rating`, `media-parse-filename-to-json`) and in
   the `comment` JSON blob `media-write-tags` emits. **There is no standard atom
   for it** — iTunes keeps star ratings in its library database, not in the
   file. `tagform` writes it to the `rating` key in `mdta` mode and to the
   freeform `com.apple.iTunes:rating` atom in `ilst` mode, and keeps the
   filename in sync when `sync_filename` is on (§8).
2. **Advisory** (`rtng`): `0` none / `2` clean / `1` explicit. A closed enum in
   **More**.
3. **Content rating** (`iTunEXTC`): `mpaa|R|400|`, `us-tv|TV-MA|600|`. Free text
   in **More**, format-validated with a warning only.

Field 4 is sense (1). The other two are never conflated with it.

### 3.4 Two different things called "type"

Same discipline:

- **Type** (field 10) is the user's own axis, and it already exists — the yt-dlp
  config has `--alias clip/master/original` writing `meta_type`. Open enum,
  free text allowed.
- **Kind** (field 11) is `stik`, a closed integer enum the Apple ecosystem
  actually reads:

  | `stik` | Label |
  |---|---|
  | 0 | Home Video |
  | 1 | Normal |
  | 2 | Audiobook |
  | 6 | Music Video |
  | 9 | Movie |
  | 10 | TV Show |
  | 21 | Podcast |

  Rendered as labels, stored as the integer. Default `9` (Movie) for video
  files with no existing value, or `10` when Show is non-empty.

### 3.5 Enum sources — the yt-dlp config is the schema

The genre and type enums are not invented here. They are exactly the aliases in
`config/yt-dlp/config`:

```
--alias media    '… --parse-metadata "Media:%(meta_genre)s"'
--alias footage  '… --parse-metadata "Camera Footage:%(meta_genre)s"'
--alias karaoke  '… --parse-metadata "Karaoke:%(meta_genre)s"'
--alias vj       '--parse-metadata "VJ Clip:%(meta_genre)s"'

--alias clip     '… --parse-metadata "Clip:%(meta_type)s"'
--alias master   '… --parse-metadata "Master:%(meta_type)s"'
--alias original '… --parse-metadata "Original:%(meta_type)s"'
```

→ Genre: `Media`, `Footage`, `Karaoke`, `VJ Clip`
→ Type: `Clip`, `Master`, `Original`

**`Footage`, not `Camera Footage`.** The yt-dlp alias literal is currently
`Camera Footage`; `tagform` normalises it. A `[enums.aliases]` table maps stored
values to canonical ones on read, so existing files tagged `Camera Footage`
display and re-save as `Footage` without a migration pass:

```toml
[enums.aliases]
"Camera Footage" = "Footage"
```

Changing the yt-dlp alias itself (`config/yt-dlp/config`, the `--alias footage`
line) is a separate one-line edit that only affects *new* downloads; the alias
table above is what makes the two agree either way.

Hard-coding them would guarantee drift the first time an alias is added, so
`tagform` **parses `~/.config/yt-dlp/config` at startup**: any
`--alias NAME '...meta_genre...'` or `...meta_type...` line contributes its
literal to the enum. Config `enums.genre` / `enums.type` (§10) extend or
override. Parse failure is not fatal — it falls back to the four/three above and
notes it in the status line.

### 3.6 The Footage profile: XMP fields from `rename-footage`

When Genre is `Footage`, six more fields appear, and they live in XMP rather
than in atoms because that is where `rename-footage` put them:

| Field | Control | XMP tag | Atom fallback on read |
|---|---|---|---|
| **Actors** | List | `XMP-iptcExt:PersonInImage` (true list) | `Keys:Actors`, `Keys:Artist` |
| **Channel** | Text | `XMP-xmpDM:Album` | `Keys:AlbumArtist`, `Keys:Album` |
| **Tags** | HashTag | `XMP-dc:Subject` (true list) | `Keys:Keywords` |
| **Location** | Text | `XMP-iptcExt:LocationCreatedCity` | — |
| **Rating** | Stars | `XMP-xmp:Rating` (0–5) | `Keys:Rating` |
| **Original name** | Text (read-only) | `XMP-xmpMM:PreservedFileName` | — |

Notes that are not optional:

- **Actors/Channel/Tags/Rating are the *same fields* as §3.1**, not new ones.
  Only their storage differs. The Footage profile changes where a field is
  written, never what the user sees. `rename-footage` already reads the atoms as
  a fallback, and `tagform` does the same in both directions.
- `XMP-xmp:Rating` is a **standard 0–5 rating field** — which largely settles
  open question 2. On Footage files the stars have a real home; the freeform
  atom is only needed elsewhere.
- **`PreservedFileName` is write-once and read-only in the form.** It is the
  only record of a camera's original `IMG_4855.MOV`. `tagform` displays it,
  offers `⌃T` to copy it into Title, and never overwrites it. If it is absent
  and the file is being renamed, `tagform` stamps it — same rule as
  `rename-footage`.
- **XMP list tags do not replace on assignment, they append.** Clearing requires
  an empty assignment *first*, and the values that follow must use `=` and not
  `+=`, because an append is applied against the original list and survives the
  clear — quietly doubling the list on every run. `rename-footage`'s
  `build_metadata_args()` documents this trap; `tagform` reuses its exact
  argument order.
- **The filename is a source, not just a sink.** `rename-footage` resolves every
  field by a fixed precedence, and `tagform` follows it so the two cannot
  disagree about the same file:

  1. an explicit edit wins outright and is written; for a list, the edited
     values *are* the list, replacing what was stored. The filename is not
     consulted.
  2. no edit, field already has metadata → keep it; ignore the filename.
  3. no edit, no metadata → parse it out of the filename **and embed it**.

  Rule 3 is what keeps the name disposable without it ever being the only copy
  of something. It is also why `--from-filename` (§10) is not really an optional
  seeding mode: for an empty field it is the *default* behaviour, and only the
  writing of it back is opt-in.
- **A camera's own name is not a title.** `IMG_4855`, `GX010042`, `C0001` and
  friends are recognised and refused for the Title field under rule 3;
  `PreservedFileName` already holds them verbatim.
- Kind (`stik`) defaults to `0` (Home Video) when Genre is `Footage`.
- Device (`com.apple.quicktime.model`) and the `[RES FPS LENGTH …]` spec block
  are **probed, never authored** — shown in the header line, never editable.

### 3.7 Keys `tagform` never shows

`major_brand`, `minor_version`, `compatible_brands`, `encoder`, `handler_name`,
`vendor_id`, `creation_time` — muxer bookkeeping, hidden from the form, and
actively cleared on every write rather than merely ignored (§2.1).

Everything else found on disk but absent from the schema appears in a
**Custom** section at the bottom: an editable key/value list, so no existing tag
is ever lost by being unrecognised. `yt_dlp_extractor`, `yt_dlp_id`,
`yt_dlp_slug`, `yt_dlp_info_json` and their siblings land here, read-only by
default (`--edit-custom` to unlock) since they are provenance, not user data.

---

## 4. Reading and writing keys

### 4.1 Read

**Two readers, always.** `ffprobe -v error -show_entries format_tags -of json`
for the atoms, and one `exiftool -f -T -G1` call for the XMP that ffprobe is
blind to (§2.2). Keys are lower-cased for lookup; the original casing is
retained for round-tripping unrecognised keys.

Reading with ffprobe alone would report every footage file as having no people,
no tags, no channel, no location and no rating — and the form would then offer
to write that emptiness back. exiftool is a hard dependency of the *read* path,
not an optional enhancement.

Precedence per field is XMP → atoms, matching `rename-footage`'s
`first_present()`. A value seen in neither is `Unset`; a value seen in both that
disagrees is surfaced in the inspector rather than silently resolved.

A second `ffprobe -show_streams` call supplies the header line: resolution,
duration, codecs, bitrate, and stream-level `tags` (which the form does not
edit, but must not clobber — hence `-map_metadata 0` on write).

Both run on a worker thread pool; the UI opens immediately with a skeleton and
fills in as probes land, because a 40-file selection on an SMB volume takes
seconds.

### 4.2 The mapping table is data, not code

```rust
struct KeyMap {
    field:   FieldId,
    mdta:    &'static [&'static str],  // every key this field writes in mdta mode
    ilst:    Option<Ilst>,             // Fourcc | Freeform{mean, name} | ITunMovi(role)
    read:    &'static [&'static str],  // aliases accepted on read, first match wins
}
```

`read` is wider than `mdta` on purpose: a file might carry `purl` but not
`webpage_url`, or `cast` but not `actors`. Read accepts any alias; write emits
the canonical set. That asymmetry is what makes the tool idempotent across files
tagged by different generations of these scripts.

The ilst column in §3 is now **measured** rather than asserted: `type`,
`origin`, `channel`, `rating` and `actors` have no ilst mapping at all and exist
only because `use_metadata_tags` allows arbitrary keys (§2.1). `keymap.json` is
generated by `tests/container-experiment.sh` and checked in, so the table is
regenerable rather than hand-maintained.

One measured trap for the read path: a value can be written by exiftool and read
back by exiftool while ffprobe reports it as **empty** — observed on a large
`.mov` whose padding atom had been consumed, and *not* predicted by value size
(docs/CONTAINER.md §4). `tagform` therefore never writes an empty value over a
field whose two readers disagree about emptiness, and warns on the
disagreement — which is observable — rather than on a byte count, which is not.

### 4.3 Multi-file aggregation

Straight from `mp4-tui-tagger`, which got this right:

| State | Meaning | Display |
|---|---|---|
| `Same(v)` | present and identical in every file | the value |
| `Mixed` | differs, or present in only some | `‹multiple›`, dimmed italic |
| `Set(v)` | user assigned a unified value | the value, marked changed |
| `Unset` | user cleared it | struck through |

`Mixed` is preserved on write: a field left alone keeps each file's own value.
Only `Set`/`Unset` touch disk. A `Mixed` field shows its per-file values in the
inspector pane (`⇥` to it, `p`), and typing into it promotes it to `Set` — with
a confirmation the first time, since that overwrites N distinct values.

For list-valued fields (Actors, Tags) `Mixed` additionally offers **merge**
(`M`): the union of all files' values, order-preserving. That is the operation
you actually want when tagging a batch, and no existing tool here has it.

---

## 5. Controls

The heart of the app. Every control implements one trait; the form is a `Vec` of
them plus a focus index.

```rust
trait Control {
    fn render(&self, f: &mut Frame, area: Rect, focused: bool, state: &ValueState);
    fn handle(&mut self, key: KeyEvent) -> Reaction;   // Consumed | Pass | Commit | Cancel
    fn value(&self) -> Value;                          // Text|List|Int|Bool|Enum|Null
    fn set_value(&mut self, v: &ValueState);
    fn validate(&self) -> Validation;                  // Ok | Warn(msg) | Error(msg)
    fn height(&self, width: u16) -> u16;               // 1 for most; TextArea grows
}
```

`Reaction::Pass` is what makes navigation work: a control that does not consume
`↑`/`↓`/`⇥` hands it back to the form. A `TextArea` consumes `↑`/`↓` internally
(cursor movement) and only passes `⇥`, which is exactly the behaviour a GUI form
has.

### 5.1 Text

`tui-input` under the hood: cursor, horizontal scroll for values wider than the
field, home/end/word-motion, and a masked variant that is unused here but free.
Rendered as a single line inside a `▏ ▕` gutter that colours by validation state.

Optional **completion**: `⌃Space` opens a dropdown of values seen in the current
selection plus the frecency list from `~/.local/share/tagform/values.json`,
filtered with `nucleo` (the matcher `grid`'s spec picks, and Helix's). Channel,
Studio, Artist and Genre use it. This is the single highest-value ergonomic
feature — 90% of tagging is retyping a channel name you have typed before.

### 5.2 TextArea

`ratatui-textarea` for Description / Synopsis / Comment. Soft-wrapped, grows to
`min(content, 8)` rows and scrolls beyond that. `⌃E` opens `$EDITOR` on a temp
file for anything longer, then reads it back — keeping `mp4-tui-tagger`'s escape
hatch without making it the only path.

Description validates: over 255 bytes emits `Warn("desc truncated by some
readers; ⌃L moves overflow to Synopsis")`, and `⌃L` performs that split. Warnings
never block a write.

### 5.3 List (Actors, Director, Producer)

Chips on one line: `Sasha Grey · Manuel Ferrara · +`.

- typing appends to the pending chip; `,` or `⏎` commits it
- `⌫` on an empty pending chip re-opens the previous chip for editing
- `←`/`→` move between chips, `⌥←`/`⌥→` reorder, `⌦` deletes the focused chip
- paste of `A, B, C` splits on commas (same rule as `ytform`'s `SplitList`)
- overflow past the field width collapses to `… +3` with the full list in the
  inspector

Stored comma-joined (`Sasha Grey, Manuel Ferrara`), matching what yt-dlp's
`%(cast)l` produces.

### 5.4 HashTag (Tags)

A List with a different grammar, because tags round-trip through *filenames*:

- displayed `#anal #pov #hd`, always with the `#`
- input accepts `#anal`, `anal`, `anal, pov`, `anal pov` — split on comma **or**
  space, leading `#` stripped then re-added (`ytform`'s `SplitTags`)
- the sanitiser mirrors the yt-dlp config's
  `--replace-in-metadata "tags" "[ _]+" "-"`: internal whitespace and
  underscores become `-`, so a tag is always one filename token
- **stored comma-joined without `#`** in `keywords` — `#` is presentation
- `Warn` on a tag containing `/`, `\`, `:` or a leading `.` (filename-hostile)
- `⌃Space` completes against the corpus of tags seen across the library index

### 5.5 URL

Text plus a `url::Url` parse on every keystroke:

| Condition | State |
|---|---|
| empty | `Ok` (absent is legal) |
| parses, scheme `http`/`https` | `Ok`, host shown dimmed to the right |
| parses, other scheme | `Warn("unusual scheme")` |
| no scheme but looks like a host | `Warn` + `⌃F` fixes it by prefixing `https://` |
| unparseable | `Error` — blocks write |

`⌃O` opens it (`open(1)`), `⌃Y` yanks it, and `⌃F` — the one that matters —
**fetches**: runs `yt-dlp --skip-download --dump-single-json` against it (via
the shared cache `media-audit` and `ytq` already use) and offers to fill Title,
Actors, Channel, Description, Tags, Date from the result, with a per-field diff
so nothing is silently overwritten. This is `media-refresh-tags` as an
interaction instead of a script.

Recognising a URL is already embedded is why the URL field reads five aliases
(§4.2): files in this library carry it as `comment` (old `media-write-tags`
output), `purl` (yt-dlp), `source_url` and `webpage_url` (`media-embed`), or
`original_url` (`media-audit`). All five are read; all five are written.

### 5.6 Stars (Rating)

The control from `media-set-rating`, made reusable:

```
  Rating      ★★★☆☆
```

`0`–`5` set directly, `←`/`→` or `h`/`l` step, `j`/`k` clear/full. Renders five
glyphs always (filled + hollow), which is the exact form the filename grammar
parses back. `Mixed` renders `☆☆☆☆☆` dimmed with a `‹multiple›` suffix.

### 5.7 Enum

Two variants sharing one widget:

- **closed** (Kind, Advisory, Language): `←`/`→` cycle, `⏎` opens a filtered
  popup list, arbitrary text rejected.
- **open** (Genre, Type): the same, plus typing enters free-text mode.
  Unrecognised values are accepted with a `Warn` naming the known set — the
  enum guides without imprisoning, which matters because these enums come from
  a config file that changes.

Selecting a value never blocks on the popup; the closed form is one line.

### 5.8 Checkbox

```
  [✓] MOV faststart      move moov to the front (recommended)
  [ ] Sync filename      rename to the Actors - [Channel] Title #tag grammar
  [ ] Back up originals  keep FILE.backup.ext next to each file
```

`Space` / `x` toggles, `y`/`n` set directly. These live in the **Write** panel
(§7), not among the metadata fields, because they describe the *operation* not
the *file*. Faststart defaults **on** and its default is settable per-machine
in config (`write.faststart = true`).

### 5.9 Date, Number

Date: `YYYY-MM-DD`, digit-only input with auto-inserted dashes, `↑`/`↓` on the
segment under the cursor increments it, `t` = today. Accepts and normalises the
`YYYYMMDD` form yt-dlp's `upload_date` uses. Number: digits only, `↑`/`↓`
increment, optional min/max.

### 5.10 Validation model

`validate()` runs per keystroke; the form aggregates:

- any `Error` → the write key is inert and the status bar names the first
  offending field. Only two things produce `Error`: an unparseable URL and a
  non-integer in a Number field.
- `Warn` → yellow gutter, listed in the confirmation dialog, never blocks.

Errors are rare by design. A tagger that refuses to save because it dislikes
your description is a worse tool than one that saves it.

---

## 6. TUI library choice

### 6.1 Survey

| Crate | Latest | Health | Verdict |
|---|---|---|---|
| [`ratatui`](https://crates.io/crates/ratatui) | 0.30 | the ecosystem standard; split into `ratatui-core`/`ratatui-widgets` at 0.30 | **base** — already what `leaf` uses |
| [`crossterm`](https://crates.io/crates/crossterm) | 0.29 | standard backend | **base**, with `use-dev-tty` on macOS as `leaf` does |
| [`tui-input`](https://crates.io/crates/tui-input) | 0.15.4 (Aug 2026) | ~494k recent downloads, actively maintained | **yes** — single-line editing, backend-agnostic, tiny |
| [`ratatui-textarea`](https://crates.io/crates/ratatui-textarea) | 0.9.2 (Jun 2026) | the ratatui-org fork of `tui-textarea`, tracks 0.30 | **yes** — multi-line fields |
| [`tui-textarea`](https://crates.io/crates/tui-textarea) | 0.7.0 (Oct 2024) | 725k downloads but stalled on ratatui 0.29 | no — the fork above supersedes it |
| [`ratatui-image`](https://crates.io/crates/ratatui-image) | 11.0.6 (Jun 2026) | ~301k recent downloads; kitty (incl. unicode placeholders), sixel, iterm2, halfblocks | **yes** — §8 |
| [`rat-widget`](https://crates.io/crates/rat-widget) | 3.2.1 | complete (checkbox, choice, focus, forms) but ~2.9k recent downloads and its own event/focus framework | no — adopting its framework costs more than writing eight controls |
| [`ratatui-interact`](https://crates.io/crates/ratatui-interact) | young | checkbox/input/button/select + focus + mouse | no — right idea, too new to depend on; revisit at v2 |
| `ratatui-form`, `ratatui-select` | young / placeholder | | no |
| [`tui-realm`](https://crates.io/crates/tuirealm) | mature | Elm-ish component framework over ratatui | no — the message indirection buys nothing at this size |
| `cursive` | mature | retained-mode, its own backend | no — will not compose with `ratatui-image` |
| `iocraft` | growing | React-like, hooks | no — different paradigm, weaker image story |

### 6.2 Recommendation

**`ratatui` + `crossterm`, with a hand-written control layer**, borrowing
`tui-input` for line editing, `ratatui-textarea` for multi-line, and
`ratatui-image` for thumbnails.

The reasoning: of the eleven controls in §5, exactly two (Text, TextArea) are
generic. The other nine — star row, hashtag chips with filename-safe
sanitisation, a URL field that fetches yt-dlp metadata, an enum sourced from a
yt-dlp config file — are domain controls that no widget library will ever ship.
A form framework would be adopted for two controls and fought for nine. The
`Control` trait in §5 is roughly 150 lines; the framework's focus/event model
would be more than that just to integrate.

This also keeps the dependency profile in line with `leaf`, the repo's other
ratatui program, so there is one ratatui version to track rather than two.

```toml
[dependencies]
ratatui           = "0.30"
crossterm         = "0.29"
tui-input         = "0.15"
ratatui-textarea  = "0.9"
ratatui-image     = "11"
nucleo            = "0.5"     # completion + the file picker
url               = "2"
serde             = { version = "1", features = ["derive"] }
serde_json        = "1"
toml              = "0.8"
anyhow            = "1"
unicode-width     = "0.2"

[target.'cfg(target_os = "macos")'.dependencies]
crossterm         = { version = "0.29", features = ["use-dev-tty"] }
```

### 6.3 Event loop and focus

Single-threaded UI; probes, thumbnails, yt-dlp fetches and the write pass run on
worker threads and report through an `mpsc::Sender<Msg>`. The loop selects over
crossterm events and that channel, redrawing only on change plus a 250 ms tick
for spinners.

```
crossterm events ─┐
                  ├─▶ App::update(Msg) ─▶ ratatui draw ─▶ /dev/tty
worker msgs ──────┘
```

Focus is an index into the visible control list, with `⇥`/`⇧⇥` and `↑`/`↓`
moving it (`↑`/`↓` only when the focused control passes them back). Hidden
sections (More, Custom) are excluded from the ring until expanded. Mouse click
sets focus when `--mouse` is on; off by default so terminal text selection keeps
working.

`/dev/tty` is opened read+write at startup for both input and rendering, so
`tagform` composes inside `$(...)` and under `fzf --bind execute(...)` — the
same rule `grid`'s spec sets out, and the reason `media-audit` opens fds 3 and 4.

### 6.4 Undo

The form model is a plain struct; undo is a bounded `Vec<FormState>` snapshot
stack (200 entries), pushed on every committed edit rather than every keystroke.
`u` undoes, `⌃R` redoes. Cheap at this data size and immune to the subtle bugs a
per-control undo would have.

---

## 7. Layout

```
┌─ tagform ──────────────────────────────── 3 files · 2 changed · mdta ─┐
│ ▛▀▀▀▀▀▀▀▀▀▀▀▀▜  Sasha Grey - [Brazzers] Some Title #pov (fh_881).mp4  │
│ ▌            ▐  1920×1080 · 24:11 · h264/aac · 1.4 GB · faststart ✓   │
│ ▌ thumbnail  ▐  ~/Movies/Porn/Downloads                               │
│ ▙▄▄▄▄▄▄▄▄▄▄▄▄▟  ‹ 1/3 ›                                               │
├───────────────────────────────────────────────────────────────────────┤
│  Title        ▏Some Title                                          ▕  │
│  Actors       ▏Sasha Grey · Manuel Ferrara · +                     ▕  │
│  Artist       ▏Sasha Grey, Manuel Ferrara                     (auto)▕  │
│  Rating        ★★★★☆                                                  │
│  Description  ▏Lorem ipsum dolor sit amet, consectetur adipiscing  ▕  │
│               ▏elit, sed do eiusmod tempor.                        ▕  │
│  URL          ▏https://faphouse.com/videos/881          ✓ faphouse ▕  │
│  Channel      ▏Brazzers                                            ▕  │
│  Tags         ▏#pov #hd #anal +                                    ▕  │
│  Genre        ▏‹ Media ›                                           ▕  │
│  Type         ▏‹ Clip ›                                            ▕  │
│  Kind         ▏‹ Movie ›                                           ▕  │
│                                                                       │
│  ▸ More (16)                        ▸ Custom (7)                      │
├─ write ───────────────────────────────────────────────────────────────┤
│  [✓] MOV faststart   [ ] Sync filename   [ ] Back up originals        │
├───────────────────────────────────────────────────────────────────────┤
│ ⇥ field  ⌃Space complete  ⌃F fetch  ]/[ file  w write  u undo  q quit │
└───────────────────────────────────────────────────────────────────────┘
```

Two-column labels/controls at ≥100 columns; labels move above their controls
below that. Below 60 columns or 20 rows the thumbnail band is dropped first,
then the write panel collapses into the status line.

The **inspector** (`p`) replaces the thumbnail band with a per-file value table
for the focused field — the answer to "what does `‹multiple›` actually contain",
which `mp4-tui-tagger` could only show in an fzf preview.

---

## 8. Thumbnails

Cheap to add here because two working implementations already exist in-repo.

**Rendering** — `ratatui-image`, which detects the terminal's best protocol
(kitty with unicode placeholders, sixel, iTerm2, halfblocks) and provides both a
stateful widget for animated resizing and a static one. The unicode-placeholder
path is the important one: the image is transmitted once and the cells hold real
text (`U+10EEEE` plus row/column diacritics), so an immediate-mode redraw does
not tear the image. `utils/ytform/thumb.go` implements that protocol by hand and
is the reference if `ratatui-image` ever has to be dropped — and the fallback
ladder is kitty → sixel → halfblocks → nothing, never an error.

**Extraction** — `media-audit`'s recipe verbatim:

```
ffmpeg -v error -y -ss 2 -i FILE -frames:v 1 \
  -vf "scale=W:H:force_original_aspect_ratio=increase:flags=lanczos,crop=W:H" \
  -q:v 3 -- OUT.jpg
```

Seek 2 s in to clear black leader, fall back to frame 0 on failure. Cover-fit
(scale-to-cover then crop) rather than fit-inside, so the band is always exactly
filled whatever the source aspect ratio is.

**Cache** — `${XDG_CACHE_HOME:-~/.cache}/tagform/thumbs/<md5>.jpg`, keyed on
`path:mtime:size:boxW×boxH`. Including the box dimensions means a terminal
resize regenerates rather than stretching a stale crop; including mtime+size
means a re-encoded file gets a new thumbnail. Straight from
`media-audit:thumb_cache_path()`.

Generation is off-thread and the UI never blocks on it. `⌃G`/`⌃⇧G` re-seeks
±10 s to pick a better frame — which, since the star rating is often a judgement
about *this specific clip*, matters more than it sounds.

Disable with `--no-thumbnail` or `thumbnail = false`.

---

## 9. The write path

### 9.1 Plan before act

`w` builds a `WritePlan` and shows it for confirmation:

```
Write 3 files

  Genre       →  Karaoke              (all 3)
  Tags        →  #pov #hd #anal       (all 3)
  Rating      →  ★★★★☆                (1 file; 2 unchanged)
  Description →  removed              (all 3)

  faststart on · originals not backed up · mdta
  2 warnings: description >255 B (2 files)

  ⏎ write   e edit plan   esc cancel
```

Nothing before this point touches disk. `mp4-tui-tagger`'s staging model,
kept — it is the reason that script is trustworthy.

### 9.2 Choosing a backend — from the file, not from a flag

There are two writers, and the choice between them is a **correctness** decision
the tool makes, never a preference the user expresses:

```
                    ┌─ adding a key the file does not have?
                    │        (in place writes it unreadably — CONTAINER §3.2)
          ┌─────────┴─────────┐
        no│                   │yes
          ▼                   ▼
   carries XMP?          carries XMP?
    ┌─────┴─────┐         ┌────┴────┐
  yes│          │no     no│         │yes
    ▼           ▼         ▼          ▼
 exiftool   exiftool    ffmpeg    TWO-PASS
 in place   in place    remux     remux, then re-apply
 (must —    (cheap,               the XMP snapshot
 remux      keeps inode           taken at read time
 destroys   + xattrs)
 XMP)
```

Two measured facts drive every branch: a remux destroys XMP (§2.2), and an
in-place write cannot *add* an mdta key — exiftool writes it, exiftool reads it
back, and ffprobe cannot see it (docs/CONTAINER.md §3.2). Updating a key that is
already present is safe in place; adding one is not.

The two-pass case is not exotic — it is what happens the first time you set
Genre on a footage clip that has XMP but no `genre` atom. It works only because
both readers always run, so the XMP snapshot exists before the remux eats it.

`--writer ffmpeg|exiftool|auto` exists for debugging and defaults to `auto`.
Forcing `ffmpeg` on a file carrying XMP prints what will be lost and requires
`--force`. This is the one place `tagform` overrides the user, and it does so
because the alternative is silent, unrecoverable data loss.

#### 9.2.1 The remux

Per file, sequentially (parallel ffmpeg on one volume is slower, not faster):

```
ffmpeg -hide_banner -loglevel error -nostdin -y \
  -i FILE -map 0 -c copy -map_metadata 0 \
  [-movflags "+faststart+use_metadata_tags"] \
  -metadata major_brand= -metadata minor_version= \
  -metadata compatible_brands= -metadata encoder= \
  -metadata KEY=VALUE ... \
  -- TMP
```

- `-map 0 -c copy -map_metadata 0` — every stream, no re-encode, stream-level
  tags preserved. A metadata edit must never be lossy.
- the four empty `-metadata` assignments clear the muxer bookkeeping
  `use_metadata_tags` would otherwise promote to real tags (§2.1). Without them
  every rewrite accumulates them.
- `-metadata KEY=` (empty value) is how a key is *deleted*.
- `TMP` is `mktemp` in the **same directory** so the swap is a rename, and
  carries the source extension so ffmpeg picks the right muxer mode (§2.1).
- faststart costs a second pass over the temp file, not a second copy.

Then, before anything is replaced — `mp4doctor`'s discipline, which exists
because this library is 200 MB–6 GB files often on network volumes:

1. **Free space check first.** Need `size + 64 MiB`; a 6 GB file on a volume
   with 4 GB free fails *before* the ffmpeg run, with its own exit code and a
   message that says "not enough space", not "could not write tags".
2. **Verify duration** carried over (±5 s).
3. **Verify the tags read back** — re-probe the temp and diff against the plan.
   This is what catches a key ffmpeg silently dropped (§4.2), and it turns the
   mapping table from an assumption into something the tool checks every run.
4. **Verify faststart** if requested: parse the atom chain, `moov` before
   `mdat`. `mp4doctor`'s `atom_state()` in Rust, ~40 lines.
5. Restore mtime and (macOS) creation date.
6. `rename(2)` over the original. Optionally `cp -p` the original to
   `FILE.backup.ext` first.

Any failure leaves the original untouched and the temp removed. The run
continues to the next file and the failure is reported in the summary — a bad
file in a batch of 40 must not abort the other 39.

### 9.3 The exiftool in-place path (milestone 4, not 8)

Originally scoped as a late speed optimisation using `mp4ameta`. Milestone 0
overturned both halves of that:

- **It is not a speed win on local storage.** Measured on a 475 MB fixture:
  ffmpeg remux **0.25 s**, exiftool in place **0.54 s**. On APFS the remux is
  memory-bandwidth work while exiftool pays ~0.2 s of Perl startup. A 6 GB local
  file remuxes in about 3 s. Whether it wins over SMB is *unmeasured*, and this
  document does not claim it does.
- **It is mandatory for correctness anyway**, because it is the only writer that
  preserves XMP (§2.2), the inode, and xattrs. That moves it from milestone 8 to
  milestone 4, and demotes speed to a side effect.

`mp4ameta` is dropped entirely — it covers only ilst, and the library is `mdta`.

Measured properties of `exiftool -overwrite_original_in_place`:

| Property | Result |
|---|---|
| inode | preserved — Finder tags and xattrs survive |
| atom chain | `ftyp moov free mdat` unchanged — faststart intact |
| unrelated keys | all preserved |
| tag growth (+8 KB) | absorbed by consuming the `wide`/`free` padding atom |

#### 9.3.1 Custom `Keys:` tags need a shipped exiftool config

Out of the box exiftool refuses them:

```
Warning: Sorry, Keys:Rating doesn't exist or isn't writable
Warning: Sorry, Keys:Actors doesn't exist or isn't writable
```

That is the exact wall `rename-footage` hit — hence its comment that "the atoms
have no equivalent for a rating, and ItemList:Keywords is not writable at all",
and hence its retreat to XMP. A user-defined config lifts it completely, and all
four custom keys then round-trip through ffprobe with the other 20 untouched.

`tagform` ships `assets/tagform.exiftool.cfg` declaring `actors`, `channel`,
`type`, `rating`, `origin`, `source_url` and `webpage_url` on
`QuickTime::Keys`, and passes `-config` on every exiftool invocation. It is a
**required runtime asset**, installed alongside the binary — not an optional
extra, and not something to regenerate at runtime.

#### 9.3.2 Fallbacks

If the moov cannot grow into the available padding, or exiftool exits non-zero,
the writer falls back to the remux — **unless** the file carries XMP, in which
case it fails loudly instead. Falling back to a writer known to destroy data is
not a fallback.

### 9.4 Filename sync

When **Sync filename** is checked, the file is renamed — but to **one of two
grammars**, selected by Genre, because this library has two and they are close
to inverses of each other:

**Media** (`ytform`, `media-parse-filename-to-json`) — Genre is anything but
`Footage`:

```
Actor A, Actor B - [Channel] Title #tag1 #tag2 (Origin) ★★★☆☆.ext
```

**Footage** (`rename-footage`) — Genre is `Footage`:

```
YYYY-MM-DD--HH-MM-SS People (Channel) - Title Location #tags [1080p 30fps 4min h264 iPhone15Pro H].ext
```

Note how they invert: media puts Channel in `[...]` and Origin in `(...)`;
footage puts Channel in `(...)` and the probed spec block in `[...]`. Composing
one file with the other's grammar produces a name that the other parser reads
back **wrongly rather than not at all**, which is the worst possible failure. So
the grammar is chosen explicitly from Genre, the choice is shown in the write
plan, and `--grammar media|footage|auto` overrides it.

Rules shared by both, taken from `ytform`'s `compose.go` and `rename-footage` so
the three tools cannot disagree: `/` becomes `-`; newlines and tabs become
spaces; the stem is truncated to 240 bytes (`media-audit`'s
`MAX_FILENAME_BYTES`). Media-only: a Channel equal to the first Actor is
dropped, and a rating of 0 emits no stars. Footage-only: **the rating never
appears in the name** (it is embedded only), every segment drops out when empty,
the ` - ` appears only when something follows it, and the `[...]` block is
re-probed fresh rather than carried over.

Rename happens after the metadata write succeeds, never before. When a file is
renamed and carries no `PreservedFileName`, `tagform` stamps the pre-rename name
into it first (§3.6) — write-once, exactly as `rename-footage` does.

This is the one place `tagform` changes something outside the container, so it
is off by default and shown in the plan as an explicit line.

---

## 10. CLI surface

```
tagform [OPTIONS] FILE...

Input
      --from-filename       seed empty fields from the filename grammar
      --fetch               seed from yt-dlp using the embedded URL (implies --from-url)
      --from-json FILE      seed from a yt-dlp .info.json
  -R, --recurse             expand directory arguments via fd-media

Output / mode
      --compat MODE         mdta | ilst | both                      [mdta]
      --set KEY=VALUE       set a field non-interactively (repeatable)
      --unset KEY           clear a field (repeatable)
      --apply               with --set/--unset: write and exit, no TUI
      --print-json          dump the aggregated tag model and exit
      --dry-run             build and print the write plan, write nothing

Write
      --no-faststart        do not add +faststart to the remux      [on]
      --backup              keep FILE.backup.ext
      --sync-filename       rename to the filename grammar
      --fast                allow the in-place ilst path (§9.3)
      --edit-custom         make unrecognised keys editable

Presentation
      --no-thumbnail
      --mouse
      --theme NAME
      --config PATH
```

Exit codes: `0` written or nothing to do, `1` one or more files failed, `2`
usage/config error, `3` insufficient disk space, `130` aborted.

`--apply` with no TUI is what makes `tagform` usable from `.job` scripts and
from `media-audit`'s fix path — the headless mode is a first-class surface, not
an afterthought, and it must never open `/dev/tty`.

---

## 11. Keys

| Key | Action |
|---|---|
| `⇥` / `⇧⇥`, `↑` / `↓` | next / previous field |
| `⏎` | commit field; on an enum, open the picker |
| `⌃Space` | completion popup |
| `⌃E` | edit the focused field in `$EDITOR` |
| `⌃F` | URL field: fetch metadata · other fields: fix-up (title case, scheme) |
| `⌃O` / `⌃Y` | open / yank the focused value |
| `0`–`5`, `h` `l` | rating (on the Rating row) |
| `Space` / `x` | toggle checkbox |
| `M` | merge a `Mixed` list field across files |
| `p` | inspector — per-file values for the focused field |
| `]` / `[` | next / previous file (single-file view) |
| `a` | show all files (aggregate view) |
| `m` / `c` | expand More / Custom |
| `⌃G` / `⌃⇧G` | thumbnail seek ±10 s |
| `u` / `⌃R` | undo / redo |
| `r` | revert — re-probe from disk, discard staged edits |
| `w` | write (plan → confirm) |
| `q` / `esc` | quit (confirms if there are staged edits) |
| `?` | key help |

vim motion (`hjkl`) is deliberately *not* bound outside the star row: this is a
text-entry app and `h` must insert an `h`.

---

## 12. Config

`~/.config/tagform/config.toml`, deployed from `config/tagform/` per the repo's
usual recipe (`dotter/global.toml` entry, `local.toml.example` line).

```toml
compat        = "mdta"
thumbnail     = true
theme         = "gruvbox"
sync_filename = false

[write]
faststart = true
backup    = false
fast      = false

[enums]
# extends what is parsed out of ~/.config/yt-dlp/config
genre = ["Media", "Camera Footage", "Karaoke", "VJ Clip", "Concert"]
type  = ["Clip", "Master", "Original"]

[defaults]
# applied only to fields that are empty on load, never overwriting
kind  = "Movie"
genre = "Media"

[fields]
# reorder, hide, or promote a Custom key into the main form
order  = ["title", "actors", "rating", "url", "channel", "tags", "genre", "type"]
hidden = ["composer", "grouping"]

[completion]
history = 500        # values remembered per field
```

Value history lives separately in `~/.local/share/tagform/values.json` (data,
not config, so it is not a candidate for the dotfiles repo).

---

## 13. Crate layout

```
utils/tagform/
├── SPEC.md
├── README.md
├── Cargo.toml
├── docs/CONTAINER.md          # milestone 0's findings — measured, done
├── assets/tagform.exiftool.cfg  # required runtime asset (§9.3.1)
├── tests/container-experiment.sh # regenerates CONTAINER.md's numbers + keymap.json
└── src/
    ├── main.rs                # CLI, headless mode, exit codes
    ├── config.rs              # config.toml + the yt-dlp alias parse (§3.5)
    ├── model/
    │   ├── schema.rs          # FieldId, the KeyMap table
    │   ├── value.rs           # Value, ValueState (Same|Mixed|Set|Unset)
    │   ├── form.rs            # the form model, dirty tracking, undo stack
    │   └── filename/
    │       ├── media.rs       # the ytform / media-parse-filename-to-json grammar
    │       └── footage.rs     # the rename-footage grammar (§9.4)
    ├── tags/
    │   ├── probe.rs           # ffprobe → Value map
    │   ├── plan.rs            # WritePlan construction and diffing
    │   ├── ffmpeg.rs          # the remux (§9.2.1)
    │   ├── exiftool.rs        # in-place writer + the shipped -config (§9.3)
    │   ├── xmp.rs             # the rename-footage XMP field set (§3.6)
    │   └── atoms.rs           # atom-chain parse: faststart verification
    ├── seed/
    │   ├── ytdlp.rs           # --fetch, the shared metadata cache
    │   └── infojson.rs
    ├── ui/
    │   ├── app.rs             # event loop, focus ring, messages
    │   ├── layout.rs
    │   ├── inspector.rs
    │   └── controls/
    │       ├── mod.rs         # the Control trait
    │       ├── text.rs  textarea.rs  list.rs  chips.rs
    │       └── url.rs   stars.rs     enums.rs checkbox.rs  date.rs  number.rs
    └── thumb.rs               # extraction, cache, ratatui-image
```

Every module above the `ui/` line is pure enough to unit test without a
terminal, which is the point of the split.

---

## 14. Testing

No CI in this repo, so tests have to be worth running by hand.

**Unit** (`cargo test`, no fixtures):
- filename grammar round-trip: `compose(parse(s)) == s` over a corpus of real
  names pulled from the library, including the pathological ones (parentheses
  inside titles, `#` in a title, unicode actors, 240-byte stems)
- list/tag splitting against `ytform`'s `SplitList`/`SplitTags` cases
- aggregation: `Same`/`Mixed`/`Set`/`Unset` transitions, merge
- validation: URL table in §5.5, date normalisation

**Container** (`tests/container-experiment.sh`): regenerates every number in
[docs/CONTAINER.md](docs/CONTAINER.md) and rewrites `keymap.json`. Run it after
any ffmpeg or exiftool upgrade — the findings are version-specific.

**Fixture** (`cargo test --features fixtures`, generates with ffmpeg):
```bash
ffmpeg -f lavfi -i testsrc=d=2:s=320x240 -f lavfi -i sine=d=2 \
  -c:v h264 -c:a aac tests/fixtures/tiny.mp4
```
- write every field, re-probe, assert every key round-trips — **this is the test
  that validates §3's mapping table**, and it must run in both compat modes and
  against both `.mp4` and `.mov`
- faststart verification: a deliberately moov-at-end fixture, fixed and detected
- delete semantics: `-metadata key=` actually removes rather than emptying
- stream tags and a second audio track survive the remux
- failure paths: read-only file, no space (temp dir on a small ramdisk), a
  truncated input
- **the XMP regression**: write XMP with exiftool, run a `tagform` write, assert
  every XMP field survives. This is the test that would have caught the §2.2
  data loss, and it must run in CI-by-hand before every release.
- backend selection: assert a file carrying XMP never routes to the remux

**Manual**: kitty / Ghostty / iTerm2 / tmux / plain xterm-256color for the
thumbnail ladder, and one run over SMB for the timing story in §9.3.

---

## 15. Integration

- `setup/install/install-tagform.sh` — `cargo build --release`, install to
  `~/.local/bin/tagform`, following the repo's installer naming rule.
- `config/tagform/config.toml` + a `[tagform.files]` section in
  `dotter/global.toml` + a line in `dotter/local.toml.example`.
- `media-audit` gains `t` on the metadata issue screen → `tagform --fetch FILE`,
  replacing the current fetch-or-skip prompt with an editable form.
- `fzf-media-select` / `ls-media` bind a key to `tagform {+}` (multi-select
  passes straight through as multiple file arguments).
- `mp4-tui-tagger` stays in place until `tagform` covers the custom-key editing
  it does, then gets a deprecation banner pointing at `tagform --edit-custom`.
  It is not deleted in the same change that lands the replacement.

---

## 16. Milestones

| # | Deliverable |
|---|---|
| **0** | ✅ **done** — [docs/CONTAINER.md](docs/CONTAINER.md). Settled open question 1, killed `mp4ameta`, and promoted the exiftool writer from an optimisation to a correctness requirement. |
| 1 | Probe (ffprobe **+ exiftool**) → model → aggregate → `--print-json`. No UI. |
| 2 | Read-only TUI: layout, focus ring, thumbnails, inspector. |
| 3 | The controls + validation + undo. Still no writes. |
| 4 | Both writers, backend selection (§9.2), verification, atomic swap, faststart. Feature-complete for one file. |
| 5 | Multi-file: merge, per-file inspector, batch summary, partial-failure reporting. |
| 6 | The Footage profile: XMP fields, the second filename grammar, `PreservedFileName`. |
| 7 | Seeding: `--from-filename`, `--fetch`, completion history. |
| 8 | Headless `--set`/`--apply`, More/Custom sections, config file, `--compat both`. |

Milestones 1–4 are the useful tool; 5–8 are what make it replace the scripts.
Milestone 6 is what makes it safe to point at `~/Movies` — until it lands,
`tagform` must **refuse** any file carrying XMP rather than risk the remux.

---

## 17. Open questions

1. ~~Does ffmpeg write `mdir` and `mdta` together?~~ **Settled: no.** One box or
   the other (§2.1). `--compat both` needs the exiftool second pass.
2. **Where should the star rating live?** Partly settled: on Footage files
   `XMP-xmp:Rating` is a real standard 0–5 field and is the answer. Elsewhere,
   `com.apple.iTunes:rating` remains a guess — worth checking what Infuse and
   Plex actually read before committing, with "stars are a filename-and-XMP
   concept, not an ilst one" as the fallback position.
3. **Is the exiftool path actually faster over SMB?** Unmeasured. Locally it
   *loses* (0.54 s vs 0.25 s on 475 MB). It is being built for XMP/inode
   preservation regardless, so this only affects whether it is also preferred
   for large local-network files. One measurement on the Tower volume settles it.
4. **Should Artist auto-mirror Actors?** Unchanged. The yt-dlp config writes the
   same value to both; the sketch in §7 shows Artist dimmed with `(auto)`,
   breaking the link on first edit. May be more magic than it is worth.
5. **`iTunMOVI`** — the plist blob holding cast/directors/producers/studio — is
   the only way Apple software sees an actor list. Deferred to milestone 8.
6. **What actually triggers the ffprobe blind spot?** Not value size — that
   first guess was disproved on re-run. The one reproducible case is a 475 MB
   `.mov` whose `wide` padding atom was consumed by an 8 KB write
   (docs/CONTAINER.md §4). Worth isolating before the writer lands, because a
   file with consumed padding is exactly what the in-place writer meets next.
7. **Should `config/yt-dlp/config` change `Camera Footage` to `Footage`?** The
   alias table (§3.5) makes `tagform` correct either way, so this is the user's
   call and affects only new downloads. One line, not made as part of this work.
