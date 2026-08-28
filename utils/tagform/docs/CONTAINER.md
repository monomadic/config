# What ffmpeg and exiftool actually write

Milestone 0 of [SPEC.md](../SPEC.md). Every claim here was measured, not read off
a wiki. Reproduce with `tests/container-experiment.sh`.

Environment: ffmpeg 8.1.2, exiftool 13.55, macOS 25.5 / APFS.

Method: a fixture tagged with 20 keys — 11 that ffmpeg has an ilst mapping for,
9 custom ones this repo's yt-dlp config produces (`actors`, `type`, `channel`,
`rating`, `origin`, `source_url`, `webpage_url`, `purl`, `yt_dlp_id`) — written
four ways, then dumped with `exiftool -a -G1 -s` and `ffprobe -show_entries
format_tags`.

---

## 1. The three namespaces are mutually exclusive

| Fixture | Container | Flags | Handler | exiftool group | Keys kept |
|---|---|---|---|---|---|
| A | `.mp4` | *(default)* | `Metadata` | `[ItemList]` | **11 of 20** |
| B | `.mp4` | `use_metadata_tags` | `Metadata Tags` | `[Keys]` | **20 of 20** |
| C | `.mov` | *(default)* | `URL` | `[UserData]` | **9 of 20** |
| D | `.mov` | `use_metadata_tags` | `Metadata Tags` | `[Keys]` | **20 of 20** |

**Open question 1 in the spec is settled: ffmpeg writes one box or the other,
never both.** A file written with `use_metadata_tags` has no `[ItemList]` at all,
and a file written without it has no `[Keys]`. So `--compat both` genuinely
requires a second tool, and cannot be had from one ffmpeg pass.

### 1.1 What the default mp4 path drops

Silently, with no warning and exit status 0:

```
actors  type  channel  rating  origin  source_url  webpage_url  purl  yt_dlp_id
```

That is the entire custom vocabulary of this repo's yt-dlp config. It is why
`--movflags use_metadata_tags` appears on every rewrite in `media-embed`,
`media-refresh-tags`, `media-audit` and `mp4doctor`, and why `tagform` defaults
to `mdta`.

### 1.2 The `.mov` default path is not merely lossy — it is wrong

```
description=…  ->  [UserData] UserData_des : …
keywords=…     ->  [UserData] UserData_key : …
```

ffmpeg invents unnamed `UserData_xxx` atoms from the first three characters of
keys it has no MOV mapping for. Nothing reads those back as description or
keywords. **`--compat ilst` on a `.mov` input must be a hard error**, as the
spec requires.

### 1.3 `use_metadata_tags` leaks muxer bookkeeping into the tag set

With `-map_metadata 0`, fixtures B and D come back carrying `major_brand`,
`minor_version` and `compatible_brands` as ordinary readable tags — they are
format tags on the input, so they get copied into `[Keys]` as real metadata.
They then accumulate on every subsequent rewrite.

`tagform` must emit `-metadata major_brand= -metadata minor_version=
-metadata compatible_brands= -metadata encoder=` on every write to suppress
them. Hiding them from the form (the spec's §3.6) is not enough; they have to be
actively cleared.

---

## 2. XMP is a fourth namespace, and it is invisible and fragile

`rename-footage` stores all of its authored metadata as XMP via exiftool:
`PersonInImage`, `dc:Subject`, `xmpDM:Album`, `LocationCreatedCity`,
`xmp:Rating`, `xmpMM:PreservedFileName`.

Two measured facts about it:

**ffprobe cannot see any of it.** A file carrying all six XMP fields reports
exactly the same 24 `format_tags` as one carrying none. A tool that reads with
ffprobe alone concludes a footage file has no people, no tags, no channel, no
location and no rating.

**An ffmpeg remux destroys all of it.**

```
before: Person One, Person Two | tag1, tag2 | Berlin | 4 | IMG_4855.MOV
  ffmpeg -map 0 -c copy -map_metadata 0 -movflags +faststart+use_metadata_tags
after : <nothing — every XMP field gone>
```

The `[Keys]` tags survive that remux; the XMP does not, and no combination of
`-map_metadata` or `-movflags` carries it across. There is no ffmpeg flag for
this because ffmpeg does not model the `uuid`/XMP box at all.

**This is the finding that reshapes the design.** `tagform`'s write path as
originally specified — always remux with ffmpeg — would wipe every field
`rename-footage` authored, including `PreservedFileName`, which that script's
own comments describe as "the only surviving copy" of a file's original name.
For any file carrying XMP, the remux is not an optimisation choice. It is data
loss.

---

## 3. exiftool in-place writing

`exiftool -overwrite_original_in_place` measured against a 475 MB fixture:

| Property | Result |
|---|---|
| inode | preserved (`12462067` → `12462067`) |
| xattrs / Finder tags | preserved |
| atom chain | `ftyp moov free mdat` → `ftyp moov free mdat` — faststart intact |
| other 23 keys | all preserved |
| tag growth (+8 KB) | absorbed by consuming the `wide`/`free` padding atom; moov stayed ahead of mdat |

### 3.1 Custom `Keys:` tags need a user-defined config

Out of the box exiftool refuses them:

```
Warning: Sorry, Keys:Rating doesn't exist or isn't writable
Warning: Sorry, Keys:Actors doesn't exist or isn't writable
```

This is exactly the wall `rename-footage` hit — hence its comment that "the
atoms have no equivalent for a rating, and ItemList:Keywords is not writable at
all", and hence its retreat to XMP.

A user-defined config lifts it completely:

```perl
%Image::ExifTool::UserDefined = (
    'Image::ExifTool::QuickTime::Keys' => {
        actors      => { Name => 'Actors',      Writable => 'string' },
        channel     => { Name => 'Channel',     Writable => 'string' },
        type        => { Name => 'Type',        Writable => 'string' },
        rating      => { Name => 'RatingStars', Writable => 'string' },
        origin      => { Name => 'Origin',      Writable => 'string' },
        source_url  => { Name => 'SourceUrl',   Writable => 'string' },
        webpage_url => { Name => 'WebpageUrl',  Writable => 'string' },
    },
);
1;
```

With it, all four custom keys wrote and read back through ffprobe, and the other
20 keys were untouched. `tagform` ships this as `assets/tagform.exiftool.cfg`.

### 3.2 In place can UPDATE a key, but cannot ADD one

The sharpest limit found, and it is not documented anywhere obvious.

| Key | Existed in file? | exiftool reads back | **ffprobe reads back** |
|---|---|---|---|
| `origin` | yes | `UPDATED` | `UPDATED` |
| `brand_new` | no | `ADDED` | **empty** |
| `brand_new` via ffmpeg remux | no | — | `ADDED` |

exiftool genuinely writes the new key — it reads it back, and the Keys count in
the box goes up — but ffprobe cannot see it. Presumably the `keys`/`ilst` index
pairing that mdta relies on ends up in a state ffprobe indexes differently;
the mechanism was not isolated, only the behaviour. A full remux adds new keys
correctly every time.

This is also the most likely explanation for the anomaly in §4, and it is
load-bearing for the writer:

- **Changing a value that is already on the file** → in place is safe.
- **Adding a key the file does not have** → must remux.
- **A file that carries XMP *and* needs a new key** → neither writer works
  alone. The remux would destroy the XMP; the in-place write would produce a
  key ffprobe cannot see. This needs a **two-pass**: remux to add the keys, then
  re-apply the XMP from the snapshot taken during the read. `tagform` already
  holds that snapshot, because both readers always run (§2).

### 3.3 The fast path is not fast on local storage

| Path | 475 MB fixture |
|---|---|
| `ffmpeg -c copy` full remux | **0.25 s** |
| `exiftool -overwrite_original_in_place` | **0.54 s** |

The remux wins, because on APFS a 475 MB copy is memory-bandwidth work while
exiftool pays ~0.2 s of Perl startup. Extrapolated, a 6 GB local file remuxes in
about 3 s — perfectly acceptable.

**So the in-place path is not justified as a speed optimisation, and SPEC.md's
original milestone 8 framing was wrong.** It earns its place for two other
reasons: it is the only way to preserve XMP (§2), and the only way to preserve
inode and xattrs. Whether it also wins over SMB is unmeasured and should not be
claimed until it is — the benefit there is theoretical, resting on exiftool
rewriting only the moov region rather than streaming the file twice.

`mp4ameta` is not needed for any of this and is dropped from the plan.

---

## 4. Value size — a reader divergence, not a size limit

First reported here as "values above ~4 KB go blind to ffprobe". **That was
wrong**, and re-running the experiment disproved it:

| Bytes | Fixture | ffprobe reads | exiftool reads |
|---|---|---|---|
| 100 – 8000 | small `.mp4` (mdta) | same | same |
| 8000 | small `.mov` (mdta) | same | same |
| 8000 | **475 MB `.mov`** | **0** | 8001 |

So size is not the variable. The one reproducible failure is a specific file:
an 8 KB write into a 475 MB `.mov` that forced exiftool to consume the `wide`
padding atom (`ftyp moov wide mdat` → `ftyp moov mdat`). exiftool reads its own
value back correctly; ffprobe reports the tag as empty while still reading every
other tag in the same box. The mechanism has **not** been isolated — it may be a
moov layout ffprobe mis-parses after padding is absorbed, or a limit unrelated
to the value at all.

Two consequences, and the first matters more precisely *because* the trigger is
unpredictable:

1. `tagform` never writes an empty value over a field whose two readers
   disagree about emptiness. A reader using ffprobe alone would see nothing
   here and could commit that nothing to disk, destroying an 8 KB description
   that was on the file the whole time.
2. There is no useful size threshold to warn at. Warn on the *disagreement*
   instead, which is observable, rather than on a byte count that is not
   predictive.

Worth isolating before the writer lands (open question 6), because a file whose
padding has been consumed is exactly the file the in-place writer will hit next.
## 5. Consequences for the design

1. `mdta` stays the default. Confirmed correct for this library.
2. `--compat ilst` on `.mov` is a hard error — it invents unreadable atoms.
3. Every write clears `major_brand`, `minor_version`, `compatible_brands`,
   `encoder`, which `use_metadata_tags` otherwise promotes to real tags.
4. **Reading requires ffprobe *and* exiftool.** ffprobe is blind to XMP.
5. **Writing a file that carries XMP must not remux it.** The writer picks its
   backend from the file's contents, not from a user preference.
6. **The in-place writer can only update keys that already exist.** Adding a key
   requires a remux. A file needing both — new key *and* XMP preservation —
   needs the two-pass in §3.2.
7. The exiftool user-defined config is a required runtime asset (§3.1).
8. `mp4ameta` is dropped: it covers only ilst, and this library is mdta.
9. Warn on reader *disagreement*, not on value size (§4).

## 6. Not yet established

Things this document deliberately does not claim:

- Whether the in-place path is faster over SMB. Locally it loses (§3.3).
- The mechanism behind §3.2 and §4 — only the behaviour is measured.
- Whether the add-vs-update limit also applies to ilst atoms, which decides
  how `--compat both` has to be built.
- What Plex and Infuse actually read for a star rating.
