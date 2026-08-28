#!/usr/bin/env bash
# Exercises every branch of the write-path decision tree against real files.
#
# The unit tests cover which writer gets *chosen*; this covers what actually
# happens on disk when it runs. The XMP case is the one that matters most: a
# bare remux annihilates everything rename-footage authored, so if that test
# ever fails, stop and fix it before shipping anything.
#
#   tests/write-paths.sh [path-to-tagform]
set -uo pipefail

BIN="${1:-$(cd "$(dirname "$0")/.." && pwd)/target/debug/tagform}"
[ -x "$BIN" ] || { echo "no binary at $BIN (cargo build first)" >&2; exit 1; }
for c in ffmpeg ffprobe exiftool python3; do command -v "$c" >/dev/null || { echo "missing $c" >&2; exit 1; }; done

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
cd "$WORK"
pass=0; fail=0
chk() { if [ "$2" = "$3" ]; then echo "  PASS $1"; pass=$((pass+1));
        else echo "  FAIL $1: got '$2' want '$3'"; fail=$((fail+1)); fi; }

cat > drive.py <<'PY'
import os, pty, sys, time, fcntl, termios, struct, select, signal, json
keys = json.loads(open(sys.argv[1]).read()); cmd = sys.argv[2:]
pid, fd = pty.fork()
if pid == 0: os.execvp(cmd[0], cmd)
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 36, 100, 0, 0))
alive = True
def pump(sec):
    global alive
    end = time.time()+sec
    while time.time() < end:
        r,_,_ = select.select([fd],[],[],0.1)
        if r:
            try:
                if not os.read(fd, 65536): alive = False; return
            except OSError: alive = False; return
pump(2.5)
for k in keys:
    if not alive: break
    try: os.write(fd, k.encode())
    except OSError: break
    pump(0.4)
if alive:
    try: os.write(fd, bytes([3]))
    except OSError: pass
    pump(1.5)
try: os.kill(pid, signal.SIGKILL)
except ProcessLookupError: pass
os.waitpid(pid, 0)
PY
cat > chain.py <<'PY'
import os,struct,sys
p=sys.argv[1]; size=os.path.getsize(p); out=[]
with open(p,'rb') as fh:
    pos=0
    while pos+8<=size and len(out)<6:
        fh.seek(pos); h=fh.read(8)
        if len(h)<8: break
        sz,ty=struct.unpack(">I4s",h); ty=ty.decode('latin-1'); hl=8
        if sz==1: sz=struct.unpack(">Q",fh.read(8))[0]; hl=16
        elif sz==0: sz=size-pos
        if sz<hl: break
        out.append(ty); pos+=sz
print(" ".join(out))
PY
# The form is modal: enter opens a field, enter saves it, and single letters are
# commands in select mode.
#   enter, NEW, enter (save), w (write), enter (confirm)
python3 -c "import json; json.dump(['\r','N','E','W','\r','w','\r'], open('title.json','w'))"
#   j x8 to Genre (a key the fixtures lack), enter, X, enter, w, enter
python3 -c "import json; json.dump(['j']*8 + ['\r','X','\r','w','\r'], open('genre.json','w'))"

tag() { ffprobe -v error -show_entries format_tags="$2" -of default=nw=1:nk=1 "$1"; }
mk_fast() { ffmpeg -v error -y -f lavfi -i testsrc=d=1:s=320x240:r=30 -c:v libx264 -pix_fmt yuv420p \
  -movflags "+faststart+use_metadata_tags" -metadata title="orig" "$1"; }
mk_slow() { ffmpeg -v error -y -f lavfi -i testsrc=d=1:s=320x240:r=30 -c:v libx264 -pix_fmt yuv420p \
  -movflags use_metadata_tags -metadata title="orig" "$1"; }

echo "== 1. in place: existing key on an already-faststart file"
mk_fast a.mp4; xattr -w com.apple.metadata:tftest hello a.mp4 2>/dev/null
ino0=$(stat -f %i a.mp4 2>/dev/null || stat -c %i a.mp4)
python3 drive.py title.json "$BIN" --no-thumbnail a.mp4
chk "title written"    "$(tag a.mp4 title)" "origNEW"
chk "inode preserved"  "$(stat -f %i a.mp4 2>/dev/null || stat -c %i a.mp4)" "$ino0"
chk "xattr preserved"  "$(xattr -p com.apple.metadata:tftest a.mp4 2>/dev/null)" "hello"

echo "== 2. remux: faststart requested on a moov-at-end file"
mk_slow b.mp4
chk "starts moov-at-end" "$(python3 chain.py b.mp4)" "ftyp free mdat moov"
python3 drive.py title.json "$BIN" --no-thumbnail b.mp4
chk "now faststart"      "$(python3 chain.py b.mp4)" "ftyp moov free mdat"
chk "title survived"     "$(tag b.mp4 title)" "origNEW"

echo "== 3. two-pass: adding a key to a file carrying XMP"
mk_fast c.mp4
exiftool -q -overwrite_original_in_place -XMP-iptcExt:PersonInImage="Alice" \
  -XMP-iptcExt:PersonInImage="Bob" -XMP-dc:Subject="beach" \
  -XMP-xmpMM:PreservedFileName="IMG_4855.MOV" -- c.mp4
python3 drive.py genre.json "$BIN" --no-thumbnail c.mp4
chk "new key added"      "$(tag c.mp4 genre)" "X"
chk "XMP people kept"    "$(exiftool -s3 -PersonInImage c.mp4 | tr '\n' ',')" "Alice, Bob,"
chk "XMP tags kept"      "$(exiftool -s3 -Subject c.mp4)" "beach"
chk "PreservedFileName"  "$(exiftool -s3 -PreservedFileName c.mp4)" "IMG_4855.MOV"

echo "== 4. failure: a read-only file is refused, not damaged"
mk_fast d.mp4; before=$(md5 -q d.mp4 2>/dev/null || md5sum d.mp4 | cut -d' ' -f1)
chmod 444 d.mp4
python3 drive.py title.json "$BIN" --no-thumbnail d.mp4
after=$(md5 -q d.mp4 2>/dev/null || md5sum d.mp4 | cut -d' ' -f1)
chk "original untouched" "$after" "$before"
chmod 644 d.mp4
chk "no temp left behind" "$(ls -a | grep -c tagform)" "0"

echo "== 5. batch: one bad file must not cost the others their write"
mk_fast e1.mp4; mk_fast e2.mp4; mk_fast e3.mp4; chmod 444 e2.mp4
python3 drive.py title.json "$BIN" --no-thumbnail e1.mp4 e2.mp4 e3.mp4
chk "first written"   "$(tag e1.mp4 title)" "origNEW"
chk "bad one skipped" "$(tag e2.mp4 title)" "orig"
chk "third written"   "$(tag e3.mp4 title)" "origNEW"
chmod 644 e2.mp4

echo "== 6. merge: union a list field across a heterogeneous selection"
ffmpeg -v error -y -f lavfi -i testsrc=d=1:s=320x240:r=30 -c:v libx264 -pix_fmt yuv420p \
  -movflags "+faststart+use_metadata_tags" -metadata actors="Alice, Bob" -metadata artist="Alice, Bob" m1.mp4
ffmpeg -v error -y -f lavfi -i testsrc=d=1:s=320x240:r=30 -c:v libx264 -pix_fmt yuv420p \
  -movflags "+faststart+use_metadata_tags" -metadata actors="bob, Carol" -metadata artist="bob, Carol" m2.mp4
#   j to Actors, m (merge), w (write), enter (confirm)
python3 -c "import json; json.dump(['j','m','w','\r'], open('merge.json','w'))"
python3 drive.py merge.json "$BIN" --no-thumbnail m1.mp4 m2.mp4
chk "merged onto first"  "$(tag m1.mp4 actors)" "Alice, Bob, Carol"
chk "merged onto second" "$(tag m2.mp4 actors)" "Alice, Bob, Carol"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
