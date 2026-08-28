#!/usr/bin/env bash
# Regenerates every measurement in docs/CONTAINER.md.
#
# The findings are version-specific: re-run after any ffmpeg or exiftool
# upgrade, because the whole design rests on them. In particular, if ffmpeg ever
# starts writing mdir and mdta together, or ever learns to carry XMP across a
# remux, several sections of SPEC.md collapse.
set -euo pipefail

WORK="${1:-$(mktemp -d)}"
CFG="$(cd "$(dirname "$0")/.." && pwd)/assets/tagform.exiftool.cfg"
cd "$WORK"
echo "workdir: $WORK"

for c in ffmpeg ffprobe exiftool; do
  command -v "$c" >/dev/null || { echo "missing: $c" >&2; exit 1; }
done

echo "== building fixtures"
ffmpeg -v error -y -f lavfi -i testsrc=d=2:s=320x240:r=30 -f lavfi -i sine=d=2 \
  -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest src.mp4
ffmpeg -v error -y -i src.mp4 -c copy src.mov

# 11 keys ffmpeg has an ilst mapping for, 9 custom ones this repo produces.
TAGS=(-metadata "title=The Title" -metadata "artist=Actor A, Actor B"
  -metadata "album_artist=ChannelName" -metadata "album=ChannelName"
  -metadata "description=Short desc" -metadata "synopsis=Long desc"
  -metadata "genre=Footage" -metadata "keywords=pov,hd" -metadata "date=2026-08-28"
  -metadata "comment=https://example.com/v/1" -metadata "media_type=9"
  -metadata "actors=Actor A, Actor B" -metadata "type=Clip" -metadata "channel=ChannelName"
  -metadata "rating=4" -metadata "purl=https://example.com/v/1"
  -metadata "source_url=https://example.com/v/1" -metadata "webpage_url=https://example.com/v/1"
  -metadata "origin=ex_1" -metadata "yt_dlp_id=abc123")

count() { ffprobe -v error -show_entries format_tags -of json "$1" |
  python3 -c 'import json,sys;print(len(json.load(sys.stdin)["format"].get("tags",{})))'; }
group() { exiftool -a -G1 -s "$1" 2>/dev/null | grep -oE '^\[(ItemList|Keys|UserData)' | sed -n 1p; }

echo
echo "== 1. the three namespaces are mutually exclusive"
ffmpeg -v error -y -i src.mp4 -map 0 -c copy -map_metadata 0 "${TAGS[@]}" A.mp4
ffmpeg -v error -y -i src.mp4 -map 0 -c copy -map_metadata 0 -movflags use_metadata_tags "${TAGS[@]}" B.mp4
ffmpeg -v error -y -i src.mov -map 0 -c copy -map_metadata 0 "${TAGS[@]}" C.mov
ffmpeg -v error -y -i src.mov -map 0 -c copy -map_metadata 0 -movflags use_metadata_tags "${TAGS[@]}" D.mov
for f in A.mp4 B.mp4 C.mov D.mov; do
  printf '  %-8s %-12s %s tags\n' "$f" "$(group $f)" "$(count $f)"
done
echo "  .mov default invents unnamed atoms:"
exiftool -a -G1 -s C.mov 2>/dev/null | grep -E 'UserData_' | sed 's/^/    /' || echo "    (none)"

echo
echo "== 2. XMP: invisible to ffprobe, destroyed by remux"
cp B.mp4 xmp.mp4
exiftool -q -overwrite_original_in_place \
  -XMP-iptcExt:PersonInImage= -XMP-iptcExt:PersonInImage="Person One" \
  -XMP-dc:Subject= -XMP-dc:Subject="tag1" \
  -XMP-xmpDM:Album="FootageChannel" -XMP-iptcExt:LocationCreatedCity="Berlin" \
  -XMP-xmp:Rating=4 -XMP-xmpMM:PreservedFileName="IMG_4855.MOV" -- xmp.mp4
echo "  ffprobe tag count with XMP: $(count xmp.mp4)  (same as without => invisible)"
echo "  before remux: $(exiftool -s3 -PersonInImage -PreservedFileName xmp.mp4 2>/dev/null | tr '\n' '|')"
ffmpeg -v error -y -i xmp.mp4 -map 0 -c copy -map_metadata 0 -movflags "+faststart+use_metadata_tags" xmp_r.mp4
after=$(exiftool -s3 -PersonInImage -PreservedFileName xmp_r.mp4 2>/dev/null | tr '\n' '|')
echo "  after remux : ${after:-<<< ALL XMP DESTROYED >>>}"

echo
echo "== 3. exiftool in place: custom Keys need the shipped config"
cp B.mp4 e1.mp4
echo "  without -config:"
exiftool -overwrite_original_in_place -Keys:Actors="X" -- e1.mp4 2>&1 | sed 's/^/    /' || true
cp B.mp4 e2.mp4
exiftool -config "$CFG" -q -overwrite_original_in_place -Keys:Actors="Custom Worked" -Keys:RatingStars=5 -- e2.mp4
echo "  with -config: actors=$(ffprobe -v error -show_entries format_tags=actors -of default=nw=1:nk=1 e2.mp4)"

echo
echo "== 4. in-place preserves inode, xattrs, faststart"
ffmpeg -v error -y -i B.mp4 -map 0 -c copy -map_metadata 0 -movflags "+faststart+use_metadata_tags" fs.mp4
xattr -w com.apple.metadata:tftest hello fs.mp4 2>/dev/null || true
before_ino=$(stat -f %i fs.mp4 2>/dev/null || stat -c %i fs.mp4)
exiftool -config "$CFG" -q -overwrite_original_in_place -Keys:Title="Edited" -- fs.mp4
after_ino=$(stat -f %i fs.mp4 2>/dev/null || stat -c %i fs.mp4)
echo "  inode : $before_ino -> $after_ino"
echo "  xattr : $(xattr -p com.apple.metadata:tftest fs.mp4 2>&1)"
echo "  chain : $(python3 - fs.mp4 <<'PY'
import os,struct,sys
p=sys.argv[1]; size=os.path.getsize(p); out=[]
with open(p,'rb') as fh:
    pos=0
    while pos+8<=size and len(out)<8:
        fh.seek(pos); h=fh.read(8)
        if len(h)<8: break
        sz,ty=struct.unpack(">I4s",h); ty=ty.decode('latin-1'); hl=8
        if sz==1: sz=struct.unpack(">Q",fh.read(8))[0]; hl=16
        elif sz==0: sz=size-pos
        if sz<hl: break
        out.append(ty); pos+=sz
print(" ".join(out))
PY
)"

echo
echo "== 4b. in place can UPDATE a key but not ADD one"
cat > addupd.cfg <<'CFGEOF'
%Image::ExifTool::UserDefined = (
    'Image::ExifTool::QuickTime::Keys' => {
        origin    => { Name => 'TfOrigin',   Writable => 'string' },
        brand_new => { Name => 'TfBrandNew', Writable => 'string' },
    },
);
1;
CFGEOF
cp B.mp4 au.mp4
exiftool -config addupd.cfg -q -overwrite_original_in_place \
  -Keys:TfOrigin="UPDATED" -Keys:TfBrandNew="ADDED" -- au.mp4
echo "  origin    (existed) ffprobe: '$(ffprobe -v error -show_entries format_tags=origin -of default=nw=1:nk=1 au.mp4)'"
echo "  brand_new (new key) ffprobe: '$(ffprobe -v error -show_entries format_tags=brand_new -of default=nw=1:nk=1 au.mp4)'"
echo "  brand_new (new key) exiftool: '$(exiftool -config addupd.cfg -s3 -Keys:TfBrandNew au.mp4 2>/dev/null)'"
ffmpeg -v error -y -i B.mp4 -map 0 -c copy -map_metadata 0 -movflags use_metadata_tags -metadata brand_new="ADDED" au2.mp4
echo "  brand_new via remux ffprobe: '$(ffprobe -v error -show_entries format_tags=brand_new -of default=nw=1:nk=1 au2.mp4)'"

echo
echo "== 5. value size: where ffprobe goes blind"
for n in 1000 4000 8000; do
  cp B.mp4 v.mp4
  V=$(python3 -c "print('d'*$n)")
  exiftool -q -overwrite_original_in_place -Keys:Description="$V" -- v.mp4 2>/dev/null
  ff=$(ffprobe -v error -show_entries format_tags=description -of json v.mp4 |
    python3 -c 'import json,sys;print(len(json.load(sys.stdin)["format"].get("tags",{}).get("description","")))')
  et=$(exiftool -s3 -Keys:Description v.mp4 2>/dev/null | tr -d '\n' | wc -c | tr -d ' ')
  echo "  wrote $n -> ffprobe $ff, exiftool $et"
done

echo
echo "done. Compare against docs/CONTAINER.md."
