#!/usr/bin/env python3
"""Real hardware integration checks. Temporary fixtures only; no external media needed."""
import argparse
from fractions import Fraction
import json
import math
import re
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser()
parser.add_argument("binary", type=lambda s: Path(s).resolve())
parser.add_argument("--large", action="store_true", help="also run 1080p and 4K smoke conversions")
parser.add_argument("--report", type=Path)
args = parser.parse_args()
results = []

def run(command, ok=True):
    p = subprocess.run([str(x) for x in command], text=True, capture_output=True, timeout=240)
    if ok and p.returncode:
        raise RuntimeError(f"{command}\n{p.stdout}\n{p.stderr}")
    if not ok:
        assert p.returncode != 0, command
    return p

def ff(*arguments):
    return run(["ffmpeg", "-v", "error", "-y", *arguments])

def probe(path):
    return json.loads(run(["ffprobe", "-v", "error", "-count_frames", "-show_streams", "-of", "json", path]).stdout)["streams"]

def rotation(stream):
    return next((x["rotation"] for x in stream.get("side_data_list", []) if "rotation" in x), 0)

with tempfile.TemporaryDirectory(prefix="rife-video-integration-") as temp:
    d = Path(temp)
    def fixture(name, rate, size="320x180", count=24, hevc=False, delay=False):
        path = d / f"{name}.mp4"
        cmd = ["-f", "lavfi", "-i", f"testsrc2=size={size}:rate={rate}"]
        if delay: cmd += ["-itsoffset", "0.2"]
        cmd += ["-f", "lavfi", "-i", "sine=frequency=1000:sample_rate=48000:duration=1"]
        cmd += ["-frames:v", str(count), "-c:v", "hevc_videotoolbox" if hevc else "libx264"]
        if hevc: cmd += ["-allow_sw", "0", "-profile:v", "main10", "-pix_fmt", "p010le"]
        else: cmd += ["-bf", "3"]
        cmd += ["-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709", "-c:a", "aac", path]
        ff(*cmd)
        return path

    def convert(name, source, codec, tier, extra=(), target="60"):
        dest = d / (name + (".mov" if codec == "prores" else ".mp4"))
        stats = d / (name + ".json")
        run([args.binary, source, dest, "--codec", codec, "--tier", tier, "--fps", target, "--stats", stats, *extra])
        s = json.loads(stats.read_text())
        streams = probe(dest); v = next(x for x in streams if x["codec_type"] == "video")
        src = next(x for x in probe(source) if x["codec_type"] == "video")
        duration = int(src["duration_ts"]) * Fraction(src["time_base"]) if "duration_ts" in src else Fraction(int(src["nb_read_frames"]), 1) / Fraction(src["r_frame_rate"])
        expected = math.ceil(duration * Fraction(target))
        assert int(v["nb_read_frames"]) == expected, (name, v["nb_read_frames"], expected)
        assert s["output_frames"] == expected
        assert s["input_frames"] == int(src["nb_read_frames"])
        assert s["hardware_decode_verified"] is True
        assert s["hardware_encode_required"] == (codec != "av1")
        assert s["generated_frames"] + s["copied_frames"] + s["tail_frames"] == expected
        assert Fraction(v["avg_frame_rate"]) == Fraction(target), (name,v["avg_frame_rate"])
        assert v["codec_name"] == {"hevc": "hevc", "prores": "prores", "av1": "av1"}[codec]
        assert (v["width"],v["height"]) == (src["width"],src["height"])
        assert rotation(v) == rotation(src), (name,rotation(v),rotation(src))
        a = next(x for x in streams if x["codec_type"] == "audio")
        sa = next(x for x in probe(source) if x["codec_type"] == "audio")
        expected_audio = max(0, float(sa.get("start_time",0))-float(src.get("start_time",0)))
        assert abs(float(a.get("start_time",0))-expected_audio) < 0.026, (name,a.get("start_time"),expected_audio)
        results.append(dict(name=name,codec=codec,tier=tier,frames=expected,wall_s=s["wall_s"]))
        print("PASS", name, expected, "frames", flush=True)
        return dest

    base = fixture("24", "24")
    out = convert("24-hq-hevc", base, "hevc", "hq")
    # Verify decoded content/order at exactly coincident 24->60 timestamps.
    psnr = run(["ffmpeg","-v","info","-i",out,"-i",base,"-lavfi",
        "[0:v]select='not(mod(n,5))',settb=1/60,setpts=N,format=yuv444p[a];[1:v]select='not(mod(n,2))',settb=1/60,setpts=N,format=yuv444p[b];[a][b]psnr", "-f","null","-"])
    match = re.search(r"average:([\d.]+)", psnr.stderr)
    assert match and float(match[1]) > 35, psnr.stderr
    # A failed overwrite request must not touch the existing destination or input.
    before = out.read_bytes()
    run([args.binary, base, out], ok=False)
    assert out.read_bytes() == before
    run([args.binary, base, base, "--overwrite"], ok=False)
    run([args.binary, base, out, "--overwrite", "--model", d/"missing.rmw"], ok=False)
    assert out.read_bytes() == before
    assert not list(d.glob(".rife-video-*")), "failed run left staging behind"
    convert("25-balanced-prores", fixture("25", "25", count=25), "prores", "balanced")
    convert("30-fast-hevc10", fixture("30", "30", count=30, hevc=True), "hevc", "fast")
    convert("50-hq-av1", fixture("50", "50", count=50), "av1", "hq", extra=["--preset","12"])
    convert("23976-balanced", fixture("fractional", "24000/1001"), "hevc", "balanced", target="60000/1001")
    delayed = fixture("delayed", "24", delay=True)
    convert("audio-offset", delayed, "hevc", "fast")
    rotated = d/"rotated.mp4"
    ff("-display_rotation:v:0", "90", "-i", base, "-c", "copy", rotated)
    convert("rotated-hevc", rotated, "hevc", "fast")
    convert("rotated-av1", rotated, "av1", "fast", extra=["--preset","12"])
    mkv = d/"source.mkv"
    ff("-i",base,"-c","copy",mkv)
    convert("mkv-input",mkv,"hevc","fast")
    vfr = d/"vfr.mp4"
    ff("-i",base,"-vf","settb=1/120,setpts='if(lt(N,12),N/(24*TB),(0.5+(N-12)/30)/TB)'",
       "-fps_mode","passthrough","-enc_time_base:v","1:120","-c:v","libx264","-c:a","copy",vfr)
    convert("vfr-input",vfr,"hevc","fast")
    hdr = d/"hdr.mp4"
    ff("-i", base, "-an", "-c:v", "libx264", "-color_primaries", "bt2020", "-color_trc", "smpte2084", "-colorspace", "bt2020nc", hdr)
    rejected = d/"hdr-out.mp4"
    p = run([args.binary,hdr,rejected],ok=False)
    assert "HDR" in p.stderr and not rejected.exists()
    if args.large:
        convert("1080p", fixture("1080", "24", size="1920x1080", count=3), "hevc", "hq")
        convert("4k", fixture("4k", "50", size="3840x2160", count=3, hevc=True), "prores", "fast")
    assert not list(d.glob("*.png")) and not list(d.glob(".rife-video-*"))
print(f"PASS: {len(results)} hardware integration cases plus overwrite/HDR failure checks")
if args.report:
    args.report.write_text(json.dumps(results,indent=2)+"\n")
