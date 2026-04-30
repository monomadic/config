# Application Internals

Low-level reference for VirtualDJ's macOS application layout, user data, database files, stem sidecars, and practical open-source tooling.

Last reviewed against local files and live VirtualDJ sources on 2026-05-01 (Asia/Manila).

## Scope

This is an operational reference. It is meant to answer questions like:

- Where does VirtualDJ keep preferences, databases, skins, pads, mappers, and caches?
- What does `database.xml` look like?
- What is in `extra.db`?
- How are prepared stem files shaped on disk?
- Which parts can be edited safely with command-line tools?
- Which parts are still unknown or only locally inferred?

Source labels used below:

- `Official`: current VirtualDJ manual or VDJPedia.
- `Official forum`: post by VirtualDJ staff, Development Manager, CTO, or Support staff.
- `External`: non-VirtualDJ technical standard or tooling reference.
- `Local observation`: verified on this machine.
- `Inference`: conclusion drawn from local files, official docs, or repeatable CLI inspection.
- `Unknown`: documented explicitly as not yet understood.

## Quick Map

macOS paths to know first:

| Path | Purpose | Source |
| --- | --- | --- |
| `/Applications/VirtualDJ.app` | Signed application bundle. Treat as read-only. | `Local observation` |
| `/Applications/VirtualDJ.app/Contents/Resources` | Bundled default skins, pad XML pages, sample banks, icons, controller metadata, and language archives. | `Local observation` |
| `/Applications/VirtualDJ.app/Contents/Frameworks/ml113.dylib` | Bundled machine-learning runtime library. | `Local observation` |
| `~/Library/Application Support/VirtualDJ` | Active VirtualDJ home folder on this macOS install. | `Official forum`, `Local observation` |
| `~/Library/Application Support/VirtualDJ/settings.xml` | Preferences, audio setups, skin state, controller choices, browser settings, sampler settings, etc. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/database.xml` | Main XML track database for media on the same drive as the home folder. | `Official forum`, `Local observation` |
| `~/Library/Application Support/VirtualDJ/extra.db` | SQLite database for related tracks, extra track identity rows, and lyrics cache. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/Cache/cache.db` | SQLite waveform cache. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/Pads` | User pad pages. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/Mappers` | User controller and keyboard mappings. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/Skins` | Installed user skins. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/Plugins64` | Intel/x86_64 plugin files and settings. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/PluginsMacArm` | Apple Silicon plugin files and settings. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/MyLists` | VirtualDJ 2024+ XML lists. | `Official`, `Local observation` |
| `~/Library/Application Support/VirtualDJ/Folders` | Virtual/favorite/filter folder metadata. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/History` | Daily history playlists, usually `.m3u`. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/Sampler` | Sampler banks, bank order, and `.vdjsample` files. | `Local observation` |
| `~/Library/Application Support/VirtualDJ/ScratchBanks` | Scratch bank XML files. | `Local observation` |
| `/Volumes/<Drive>/VirtualDJ/database.xml` | Local database for media on a different drive from the VirtualDJ home folder. | `Official forum`, `Inference` |

Older answers and some Windows-centric forum posts refer to a VirtualDJ folder under `Documents`. On this macOS install the active folder is `~/Library/Application Support/VirtualDJ`, and a 2024 CTO forum reply points macOS users there for `database.xml`.

## Application Bundle

The app bundle is useful for inspection, but user work should live in the home folder.

Observed local app version:

```sh
defaults read /Applications/VirtualDJ.app/Contents/Info.plist CFBundleShortVersionString
defaults read /Applications/VirtualDJ.app/Contents/Info.plist CFBundleVersion
```

On this machine those commands returned:

```text
8.5.8769
18.0.9295
```

Useful bundled resources:

```sh
find /Applications/VirtualDJ.app/Contents/Resources -maxdepth 1 -type f \
  \( -name '*.xml' -o -name '*.zip' -o -name '*.dat' \) \
  -print | sort
```

Typical results include:

- `skin.zip`
- `pads_stems.xml`
- `pads_hotcues.xml`
- `pads_sampler.xml`
- `controllers.dat`
- default sampler banks such as `AUDIO FX.xml`, `FAMOUS.xml`, and `INSTRUMENTS.xml`

Do not edit these bundled files in place. Put overrides or custom work under the VirtualDJ home folder.

## Home Folder Rules

VirtualDJ has a "home" folder that contains the master database and most user configuration. Staff forum guidance describes this model:

- Files on the same drive as the home folder use the master database in the home folder.
- Files on other drives can use a local database in a `VirtualDJ` folder at that drive's root.
- Moving the home folder changes which files land in the master database versus a local drive database.

macOS default to check:

```sh
VDJ_HOME="$HOME/Library/Application Support/VirtualDJ"
ls -la "$VDJ_HOME"
```

External drive database check:

```sh
find /Volumes -maxdepth 3 -path '*/VirtualDJ/database.xml' -print 2>/dev/null
```

## Safe Editing Rules

Use these rules before touching live data:

1. Quit VirtualDJ.
2. Back up the file you will edit.
3. Validate XML before and after an XML edit.
4. For SQLite, expect write-ahead log files such as `extra.db-wal` and `cache.db-wal`.
5. Prefer VirtualDJ's own UI for destructive operations, re-analysis, and database repair.
6. Treat `Cache/`, `license.dat`, binary `.dat`, `.mlmodelc`, `.vdjsample`, and unknown binary files as read-only until their format is understood.

Quick backup and XML validation:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

VDJ_HOME="${VDJ_HOME:-$HOME/Library/Application Support/VirtualDJ}"
stamp="$(date +%Y%m%d-%H%M%S)"

cp -p "$VDJ_HOME/database.xml" "$VDJ_HOME/database.xml.$stamp.bak"
xmllint --noout "$VDJ_HOME/database.xml"

print "backup: $VDJ_HOME/database.xml.$stamp.bak"
print "database.xml is well-formed"
```

## settings.xml

`settings.xml` is a single XML document rooted at `<settings>`.

Observed top-level sections include:

- `audioConfig`
- `automation`
- `controls`
- `skins`
- `audio`
- `controllers`
- `sampler`
- `browser`
- `options`

The file stores both durable settings and UI state. Examples observed locally:

- audio setups under `<audioConfig>`
- current skin under `<skins><skin>`
- skin panel and split states under `<skinPanels>` and `<skinSplitState>`
- controller to mapper choices under `<controllers>`
- browser columns and shortcuts under `<browser>`
- last database backup timestamp under `<options><databaseBackupLast>`

Inspect setting names without dumping personal values:

```sh
xmlstarlet sel -t -m '/settings/*' -v 'name()' -n \
  "$HOME/Library/Application Support/VirtualDJ/settings.xml"
```

That XPath is intentionally minimal. For a practical overview, this is usually easier:

```sh
rg -n '<[A-Za-z][A-Za-z0-9_]*' \
  "$HOME/Library/Application Support/VirtualDJ/settings.xml" |
  sed -E 's/^([0-9]+:)[[:space:]]*<([^ >]+).*/\1 \2/' |
  head -120
```

## database.xml

`database.xml` is the main track metadata database. It is XML:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<VirtualDJ_Database Version="8.5">
  <Song FilePath="/Music/Tracks/example.flac" FileSize="12345678" Flag="33554432">
    <Tags Author="Artist" Title="Title" Remix="Extended Mix" Stars="5" Key="Cm" Bpm="0.468750"/>
    <Infos SongLength="240.000000" FirstSeen="1767225600" PlayCount="0" Bitrate="1411.2" Cover="1"/>
    <Comment>#tag #another-tag</Comment>
    <Scan Version="801" Bpm="0.468750" Volume="1.250000" Key="Cm"/>
    <Poi Pos="0.000000" Type="beatgrid" Bpm="128.000000"/>
    <Poi Type="cue" Pos="32.000000" Name="DROP" Num="1" Color="4278255360"/>
    <Poi Type="loop" Pos="224.000000" Name="OUTRO" Num="2" Size="8" Slot="2" Color="4278190208"/>
  </Song>
</VirtualDJ_Database>
```

Observed structure on this machine:

```sh
VDJ_DB="$HOME/Library/Application Support/VirtualDJ/database.xml"

xmllint --xpath 'name(/*)' "$VDJ_DB"
xmllint --xpath 'string(/*/@Version)' "$VDJ_DB"
xmllint --xpath 'count(/VirtualDJ_Database/Song)' "$VDJ_DB"

rg -o '<[A-Za-z][A-Za-z0-9_:-]*' "$VDJ_DB" |
  sed 's/^<//' |
  sort |
  uniq -c |
  sort -nr
```

Local counts at time of review:

```text
VirtualDJ_Database
8.5
1566 songs
17529 Poi
1566 Tags
1566 Song
1566 Scan
1566 Infos
1566 Comment
2 LockedCues
```

### Song

`<Song>` identifies one database item.

Common attributes:

- `FilePath`: absolute path, URL-like source path, or other VirtualDJ source path.
- `FileSize`: size in bytes when known.
- `Flag`: bit field. Some values are discussed in forums, but the complete current map is not maintained here yet.

Do not use `FilePath` alone as a stable identity. VirtualDJ also uses file size and internal identifiers elsewhere.

### Tags

`<Tags>` contains user-facing metadata:

- `Author`
- `Title`
- `Genre`
- `Album`
- `Label`
- `Remix`
- `Remixer`
- `TrackNumber`
- `Grouping`
- `Year`
- `Stars`
- `User1`, `User2`, etc.
- `Key`
- `Bpm`
- `Flag`

Important BPM detail:

- `Tags/@Bpm` and `Scan/@Bpm` are observed as beat duration in seconds.
- Display BPM can be derived as `60 / value`.
- `Poi[@Type="beatgrid"]/@Bpm` is observed as display BPM.

Example:

```sh
awk 'BEGIN { stored = 0.468750; printf "%.3f BPM\n", 60 / stored }'
```

### Infos

`<Infos>` stores analysis and library state:

- `SongLength`: seconds.
- `FirstSeen`: Unix timestamp.
- `LastModified`: Unix timestamp when present.
- `PlayCount`
- `LastPlay`
- `Bitrate`
- `UserColor`: decimal color value.
- `Cover`: cover-art state.

Convert a timestamp:

```sh
date -r 1767225600 '+%Y-%m-%d %H:%M:%S %Z'
```

### Scan

`<Scan>` stores analysis output such as:

- `Version`
- `Bpm`
- `Volume`
- `Key`
- `Flag`

Treat `Scan/@Flag` as an unknown bit field unless a specific value has been verified.

### Comment

`<Comment>` contains the browser comment field. It may be empty:

```xml
<Comment/>
```

or contain text:

```xml
<Comment>#minimal #loud</Comment>
```

### Poi

`<Poi>` stores points of interest.

Observed `Type` values:

```sh
rg -o 'Type="[^"]+"' "$HOME/Library/Application Support/VirtualDJ/database.xml" |
  sort |
  uniq -c |
  sort -nr
```

Local counts at time of review:

```text
11253 Type="cue"
2858 Type="automix"
1721 Type="beatgrid"
1199 Type="remix"
498 Type="loop"
```

Common forms:

```xml
<Poi Pos="0.000000" Type="beatgrid" Bpm="128.000000"/>
<Poi Type="cue" Pos="64.000000" Name="BREAK" Num="2" Color="4293375736"/>
<Poi Type="loop" Pos="224.000000" Name="OUTRO" Num="8" Size="16" Slot="8" Color="4278190208"/>
<Poi Pos="0.050000" Type="automix" Point="realStart"/>
<Poi Name="Break 1" Pos="90.000000" Type="remix"/>
```

Observed interpretation:

- `Pos` is seconds.
- cue `Num` is the cue number.
- loop `Size` is in beats.
- `Color` is a decimal ARGB-like integer. For example, red is commonly `4294901760`, which is `0xFFFF0000`.

Convert decimal color to hex:

```sh
printf '0x%08X\n' 4294901760
```

### LockedCues

`<LockedCues>` appears inside some `<Song>` entries:

```xml
<LockedCues>1</LockedCues>
```

Observed meaning is cue locking state, but the full set of values is not documented here yet.

## Querying database.xml

Install helper tools:

```sh
brew install xmlstarlet
```

Export a browser-like TSV:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

VDJ_DB="${VDJ_DB:-$HOME/Library/Application Support/VirtualDJ/database.xml}"

xmlstarlet sel -T -t \
  -m '/VirtualDJ_Database/Song' \
  -v '@FilePath' -o $'\t' \
  -v 'Tags/@Author' -o $'\t' \
  -v 'Tags/@Title' -o $'\t' \
  -v 'Tags/@Remix' -o $'\t' \
  -v 'Tags/@Key' -o $'\t' \
  -v 'format-number(60 div number(Tags/@Bpm), "0.000")' -o $'\t' \
  -v 'Infos/@PlayCount' \
  -n \
  "$VDJ_DB"
```

Find songs that have more than eight cue points:

```sh
xmlstarlet sel -T -t \
  -m '/VirtualDJ_Database/Song[count(Poi[@Type="cue"]) > 8]' \
  -v 'Tags/@Author' -o ' - ' -v 'Tags/@Title' -o $'\t' \
  -v 'count(Poi[@Type="cue"])' \
  -n \
  "$HOME/Library/Application Support/VirtualDJ/database.xml"
```

## Editing database.xml

Prefer VirtualDJ's UI for normal tagging. Direct XML edits are useful for controlled bulk changes.

The script below updates `Tags/@User1` for a song selected by exact file path. It:

- requires `xmlstarlet`
- requires VirtualDJ to be closed
- backs up `database.xml`
- validates XML before and after
- handles file paths containing either single or double quotes

```zsh
#!/usr/bin/env zsh
set -euo pipefail

command -v xmlstarlet >/dev/null || {
  print -u2 "missing dependency: brew install xmlstarlet"
  exit 1
}

VDJ_DB="${VDJ_DB:-$HOME/Library/Application Support/VirtualDJ/database.xml}"
filepath="${1:?usage: vdj-db-set-user1 <exact-file-path> <new-user1-value>}"
value="${2:?usage: vdj-db-set-user1 <exact-file-path> <new-user1-value>}"

xpath_literal() {
  local s="$1"

  if [[ "$s" != *"'"* ]]; then
    printf "'%s'" "$s"
    return
  fi

  if [[ "$s" != *'"'* ]]; then
    printf '"%s"' "$s"
    return
  fi

  local out="concat("
  local first=1
  local rest="$s"
  local part

  while [[ "$rest" == *"'"* ]]; do
    part="${rest%%\'*}"
    if (( ! first )); then
      out+=","
    fi
    out+="'$part',\"'\""
    rest="${rest#*\'}"
    first=0
  done

  out+=",'$rest')"
  printf "%s" "$out"
}

xmllint --noout "$VDJ_DB"

stamp="$(date +%Y%m%d-%H%M%S)"
cp -p "$VDJ_DB" "$VDJ_DB.$stamp.bak"

literal="$(xpath_literal "$filepath")"
song="/VirtualDJ_Database/Song[@FilePath=${literal}]"

matches="$(xmlstarlet sel -t -v "count($song)" "$VDJ_DB")"
if [[ "$matches" != "1" ]]; then
  print -u2 "expected exactly one matching Song, found $matches"
  exit 2
fi

has_attr="$(xmlstarlet sel -t -v "count($song/Tags/@User1)" "$VDJ_DB")"
if [[ "$has_attr" == "0" ]]; then
  xmlstarlet ed -P -L \
    -i "$song/Tags" -t attr -n User1 -v "$value" \
    "$VDJ_DB"
else
  xmlstarlet ed -P -L \
    -u "$song/Tags/@User1" -v "$value" \
    "$VDJ_DB"
fi

xmllint --noout "$VDJ_DB"
print "updated User1"
print "backup: $VDJ_DB.$stamp.bak"
```

## extra.db

`extra.db` is SQLite. It can be locked while VirtualDJ is running. For read-only inspection while VirtualDJ is open, SQLite's immutable URI mode is useful:

```sh
sqlite3 "file:$HOME/Library/Application Support/VirtualDJ/extra.db?mode=ro&immutable=1" '.schema'
```

Observed schema:

```sql
CREATE TABLE related_tracks (id INTEGER PRIMARY KEY, sid1 INTEGER, sid2 INTEGER);
CREATE INDEX idx_sid1 ON related_tracks (sid1);
CREATE INDEX idx_sid2 ON related_tracks (sid2);

CREATE TABLE track_data (
  id INTEGER PRIMARY KEY,
  sid INTEGER,
  file TEXT,
  filesize INTEGER,
  artist TEXT,
  title TEXT,
  remix TEXT,
  UNIQUE(sid)
);
CREATE INDEX idx_sid ON track_data (sid);

CREATE TABLE lyrics (lid BLOB NOT NULL PRIMARY KEY, xml TEXT NOT NULL);
```

Observed local counts at time of review:

```text
track_data: 61
related_tracks: 81
lyrics: 331
```

### Linked Tracks

VirtualDJ's linked/remix relationships are stored in `extra.db`:

- `track_data.sid`: signed 64-bit opaque song identifier.
- `track_data.file`: path to a track.
- `track_data.filesize`: file size in bytes.
- `track_data.artist`, `title`, `remix`: display metadata.
- `related_tracks.sid1`, `related_tracks.sid2`: relationship edges between `track_data.sid` values.

Known unknown:

- The algorithm that creates `sid` is not known yet.
- It appears to be a signed 64-bit identifier, not a plain path string.
- Because the hash/ID algorithm is unknown, creating linked-track rows from scratch is unsafe unless VirtualDJ has already created `track_data` rows for both files.

Read related tracks:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

DB="${DB:-$HOME/Library/Application Support/VirtualDJ/extra.db}"

sqlite3 "file:$DB?mode=ro&immutable=1" <<'SQL'
.headers on
.mode tabs
select
  a.file as file1,
  b.file as file2,
  a.artist || ' - ' || a.title as track1,
  b.artist || ' - ' || b.title as track2
from related_tracks r
join track_data a on a.sid = r.sid1
join track_data b on b.sid = r.sid2
order by track1, track2;
SQL
```

Add a relationship only when both `track_data` rows already exist:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

DB="${DB:-$HOME/Library/Application Support/VirtualDJ/extra.db}"
left="${1:?usage: vdj-link-existing <left-file> <right-file>}"
right="${2:?usage: vdj-link-existing <left-file> <right-file>}"

sqlite_literal() {
  local s
  s="$(printf '%s' "$1" | sed "s/'/''/g")"
  printf "'%s'" "$s"
}

stamp="$(date +%Y%m%d-%H%M%S)"
cp -p "$DB" "$DB.$stamp.bak"

left_sql="$(sqlite_literal "$left")"
right_sql="$(sqlite_literal "$right")"

sqlite3 "$DB" <<SQL
insert into related_tracks (sid1, sid2)
select a.sid, b.sid
from track_data a, track_data b
where a.file = $left_sql
  and b.file = $right_sql
  and not exists (
    select 1
    from related_tracks r
    where r.sid1 = a.sid
      and r.sid2 = b.sid
  );

select changes() as inserted_rows;
SQL

print "backup: $DB.$stamp.bak"
```

If that script prints `0`, one or both files probably do not have `track_data` rows yet, or the relationship already exists.

### Lyrics Cache

`lyrics` contains:

- `lid`: 18-byte BLOB in local samples.
- `xml`: text payload.

Despite the column name, observed lyric text can look like line-oriented timestamp ranges:

```text
[0.69-0.83] I'm
[0.89-0.90] a
```

or a sentinel:

```text
#NOLYRICS
```

Known unknown:

- The exact `lid` derivation is unknown.
- The full lyric payload grammar is not documented here yet.
- How lyric cache rows relate to audio signatures and server-side cache behavior needs more verification.

## cache.db

`Cache/cache.db` is SQLite and stores waveform blobs.

Observed schema:

```sql
CREATE TABLE waveforms (
  id INTEGER PRIMARY KEY,
  filepath TEXT,
  filename TEXT,
  filesize INTEGER,
  type INTEGER,
  version INTEGER,
  valuesPerSecond REAL,
  waveform BLOB
);
CREATE INDEX idx_waveform_filename ON waveforms (filename, type);
```

Use this for inspection only. It is a generated cache.

Read-only waveform count:

```sh
sqlite3 "file:$HOME/Library/Application Support/VirtualDJ/Cache/cache.db?mode=ro&immutable=1" \
  'select count(*) from waveforms;'
```

Other observed cache files:

- `Cache/cache.db-shm`
- `Cache/cache.db-wal`
- `Cache/cache2.db`
- `Cache/cache3.db`
- `Cache/cache4.db`
- `Cache/fft`

The non-SQLite cache files are currently `Unknown`.

## Lists And Virtual Folders

VirtualDJ 2024+ lists are XML files under `MyLists` by default. VDJPedia says the only required song entry attribute is `path`, and `size` is highly recommended.

Generic list:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<VirtualFolder noDuplicates="yes" ordered="yes">
  <song path="/Music/Tracks/example.flac" size="12345678" artist="Artist" title="Title" idx="0"/>
</VirtualFolder>
```

Observed local empty folder/list forms:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<VirtualFolder />
```

```xml
<?xml version="1.0" encoding="UTF-8"?>
<VirtualFolder noDuplicates="no" ordered="yes" />
```

## Sampler And Scratch Banks

Sampler bank XML lives under `Sampler/`.

Example shape:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<samplerbank>
  <sample path="Instruments\Kick.vdjsample" group="BEATS" color="red" col="0" row="0" />
</samplerbank>
```

Scratch banks live under `ScratchBanks/` and can embed a nested `<Song>` record:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<scratchbank name="Bank A">
  <sample path="/Music/Samples/example.flac" filesize="108637" idx="1" color="#00FF37">
    <Song FilePath="/Music/Samples/example.flac" FileSize="108637" Flag="1">
      <Tags Flag="1" />
      <Infos SongLength="2.181812" FirstSeen="1755968487" Bitrate="398" Cover="2" />
    </Song>
  </sample>
</scratchbank>
```

`.vdjsample` is binary. Some files show a `VDJ` header followed by Matroska-like content, but the full format is not mapped here.

## Stem Files

VirtualDJ's public docs describe five stem components:

- vocal
- instruments
- bass
- hihat
- kick

The docs also say that when real-time separation is too expensive, VirtualDJ can prepare tracks in advance by saving stems in a separate file.

### Observed VirtualDJ Sidecars

VirtualDJ-prepared `.vdjstems` files observed locally are Matroska containers with five stereo AAC streams:

```text
0 vocal
1 hihat
2 bass
3 instruments
4 kick
```

Inspect one:

```sh
ffprobe -v error -select_streams a \
  -show_entries stream=index,codec_name,sample_fmt,channels:stream_tags=title \
  -of compact=p=0:nk=1 \
  "/path/to/track.flac.vdjstems"
```

Expected shape:

```text
0|aac|fltp|2|vocal
1|aac|fltp|2|hihat
2|aac|fltp|2|bass
3|aac|fltp|2|instruments
4|aac|fltp|2|kick
```

The sidecar naming pattern is usually:

```text
/path/to/original.ext.vdjstems
```

### Open-Source Recreation: Five-Stream Matroska

This creates a VirtualDJ-like five-stream `.vdjstems` container from prepared WAV files:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

command -v ffmpeg >/dev/null || {
  print -u2 "missing dependency: brew install ffmpeg"
  exit 1
}

dir="${1:?usage: vdjstems-pack-matroska <stem-dir> [output.vdjstems]}"
out="${2:-$dir/$(basename "$dir").vdjstems}"

for name in vocal hihat bass instruments kick; do
  [[ -f "$dir/$name.wav" ]] || {
    print -u2 "missing $dir/$name.wav"
    exit 2
  }
done

ffmpeg -y \
  -i "$dir/vocal.wav" \
  -i "$dir/hihat.wav" \
  -i "$dir/bass.wav" \
  -i "$dir/instruments.wav" \
  -i "$dir/kick.wav" \
  -map 0:a -map 1:a -map 2:a -map 3:a -map 4:a \
  -metadata:s:a:0 title="vocal" \
  -metadata:s:a:1 title="hihat" \
  -metadata:s:a:2 title="bass" \
  -metadata:s:a:3 title="instruments" \
  -metadata:s:a:4 title="kick" \
  -c:a aac -b:a 320k \
  -f matroska \
  "$out"

ffprobe -v error -select_streams a \
  -show_entries stream=index,codec_name,channels:stream_tags=title \
  -of compact=p=0:nk=1 \
  "$out"
```

### Open-Source Separation Pipeline

One workable pipeline is:

1. Use Demucs or another open model to produce `vocals.wav`, `drums.wav`, `bass.wav`, and `other.wav`.
2. Rename `vocals.wav` to `vocal.wav`.
3. Rename `other.wav` to `instruments.wav`.
4. Split `drums.wav` into `kick.wav` and `hihat.wav` with a drum separation model.
5. If no drum-element splitter is available, use `drums.wav` for both `kick.wav` and `hihat.wav` as a low-quality compatibility fallback.
6. Pack with the Matroska script above.

Minimal Demucs example:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

command -v demucs >/dev/null || {
  print -u2 "missing dependency: pipx install demucs"
  exit 1
}
command -v ffmpeg >/dev/null || {
  print -u2 "missing dependency: brew install ffmpeg"
  exit 1
}

input="${1:?usage: vdjstems-demucs-basic <audio-file> [out-dir]}"
outdir="${2:-stems-basic}"
work="$outdir/_work"
mkdir -p "$work"

ffmpeg -y -v error -i "$input" -ar 44100 -ac 2 -c:a pcm_s16le "$work/mix.wav"

model="${DEMUCS_MODEL:-htdemucs_ft}"
demucs -n "$model" -o "$work" "$work/mix.wav"

stemdir="$(find "$work" -type d -path "*/$model/mix" -print -quit)"
[[ -n "$stemdir" ]] || {
  print -u2 "could not find Demucs output under $work"
  exit 2
}

cp "$stemdir/vocals.wav" "$outdir/vocal.wav"
cp "$stemdir/bass.wav" "$outdir/bass.wav"
cp "$stemdir/other.wav" "$outdir/instruments.wav"

# Compatibility fallback until a real drum-element splitter is added.
cp "$stemdir/drums.wav" "$outdir/kick.wav"
cp "$stemdir/drums.wav" "$outdir/hihat.wav"

print "wrote $outdir/{vocal,hihat,bass,instruments,kick}.wav"
```

### Six-Stream MP4/M4A Variant

Local helper scripts outside this reference have also experimented with a six-stream MP4/M4A layout:

```text
0 mixed track
1 vocal
2 hihat
3 bass
4 instruments
5 kick
```

This is useful for archival or compatibility experiments, especially with ALAC:

```zsh
#!/usr/bin/env zsh
set -euo pipefail

dir="${1:?usage: vdjstems-pack-mp4-6 <stem-dir> [output.mp4]}"
out="${2:-$dir/$(basename "$dir").vdjstems.mp4}"

for name in mixed vocal hihat bass instruments kick; do
  [[ -f "$dir/$name.wav" ]] || {
    print -u2 "missing $dir/$name.wav"
    exit 2
  }
done

ffmpeg -y \
  -i "$dir/mixed.wav" \
  -i "$dir/vocal.wav" \
  -i "$dir/hihat.wav" \
  -i "$dir/bass.wav" \
  -i "$dir/instruments.wav" \
  -i "$dir/kick.wav" \
  -map 0:a -map 1:a -map 2:a -map 3:a -map 4:a -map 5:a \
  -metadata:s:a:0 title="mixed track" \
  -metadata:s:a:1 title="vocal" \
  -metadata:s:a:2 title="hihat" \
  -metadata:s:a:3 title="bass" \
  -metadata:s:a:4 title="instruments" \
  -metadata:s:a:5 title="kick" \
  -c:a alac \
  -f mp4 \
  "$out"
```

Known unknown:

- Current VirtualDJ recognition rules for the six-stream MP4/M4A variant need more cross-version testing.
- Exact MP4 metadata atoms required for the broadest compatibility are not fully mapped.

## File Formats We Know

| File | Format | Notes |
| --- | --- | --- |
| `settings.xml` | XML | Preferences and UI state. |
| `database.xml` | XML | Main track database. |
| `*.vdjfolder` | XML | Lists, virtual folders, sideview lists. |
| `Mappers/*.xml` | XML | Controller and keyboard mappings. |
| `Pads/*.xml` | XML | Pad pages. |
| `Skins/*/*.xml` | XML | Installed skin files. |
| `extra.db` | SQLite | Related tracks, track_data, lyrics. |
| `Cache/cache.db` | SQLite | Waveform cache. |
| `History/*.m3u` | M3U-like playlist text | Daily history. |
| `*.vdjstems` | Matroska in observed VirtualDJ-prepared files | Five AAC streams named vocal/hihat/bass/instruments/kick. |
| `*.vdjsample` | Binary | Not fully mapped. Some files show a `VDJ` header and media payload. |

## Known Unknowns

- `extra.db track_data.sid`: signed 64-bit identifier/hash for linked tracks. Algorithm unknown.
- `extra.db lyrics.lid`: 18-byte blob identifier. Algorithm unknown.
- Full `Song/@Flag`, `Tags/@Flag`, and `Scan/@Flag` bit maps.
- `Cache/cache2.db`, `Cache/cache3.db`, `Cache/cache4.db`, and `Cache/fft`.
- Complete `.vdjsample` structure.
- Whether prepared `.vdjstems` sidecar metadata has fields beyond stream titles that matter to every VirtualDJ version.
- Whether every VirtualDJ 2026 build accepts custom Matroska `.vdjstems` generated externally.
- Exact role and versioning of bundled `Drivers/model3.mlmodelc`, `Drivers/model4.mlmodelc`, and `ml113.dylib`.

## Sources

- [VirtualDJ forum: macOS `database.xml` location](https://www.virtualdj.com/forums/261193/VirtualDJ_Technical_Support/Where_can_I_find_the_database_xml_file_on_MacOS_.html) - `Official forum`
- [VirtualDJ forum: home folder, master database, and per-drive local databases](https://www.virtualdj.com/forums/223863/VirtualDJ_Technical_Support/Machine_specific_settings_xml_and_licence_dat_.html) - `Official forum`
- [VirtualDJ stems help](https://de.virtualdj.com/help/stems.html) - `Official`
- [VDJPedia: Lists](https://de.virtualdj.com/wiki/Lists.html) - `Official`
- [Matroska stem files Internet-Draft](https://www.ietf.org/archive/id/draft-swhited-mka-stems-06.html) - `External`
- Local files under `~/Library/Application Support/VirtualDJ` - `Local observation`
- Local app bundle under `/Applications/VirtualDJ.app` - `Local observation`
