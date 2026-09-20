# Frame interpolation on this machine

Measured on an Apple M4 Pro (Mac16,7 — 10P + 4E cores, 20-core GPU, 48 GB),
macOS 26.5, ffmpeg 9.0.1, DaVinci Resolve Studio 21.0.2. Two runs: 2026-09-10
by hand, and 2026-09-12 with `bin/interpolation-benchmark` covering 24→60
and 30→60 at 1080p and 4K.

Three tools in this repo do frame interpolation: **`smooth-fps`**,
**`interpolate-resolve`** and **`bin/rife-vapoursynth`**. `interpolate-resolve`
gives the best quality when Resolve is available — see the 2026-09-12 matrix
below, where all three of its optical flow modes beat both of the others.

**There are two RIFE things, and only one of them is a tool here.**
`rife-ncnn-vulkan` is the upstream binary (installed to `~/.local/bin` by
`scripts/install/install-rife-arm64`); `bin/rife-vapoursynth` is the repo's tool, which
runs that same v4.6 engine through the VapourSynth plugin in memory — no PNG
frames, no scratch disk. It is the RIFE path that gets benchmarked and the one
to use. An earlier wrapper called `rife-60fps` drove the binary through
per-frame PNGs; it has been removed, and the rows below marked "PNG wrapper"
are its historical numbers, kept only because they are what the 2026-09-10
column was measured with.

On a 30 s window through `vspipe` alone: decode + RGB round trip 2192 fps,
rife 2× 59.7 out fps, rife 3× 44.8 out fps, and adding x264 costs 3%. Both
rife figures are ≈30 inferences/s, i.e. the plugin's overhead is nil and the
ncnn/MoltenVK engine is the wall. End to end it managed 30.0 out fps on a
3-minute 720p clip (with `--dedup`, so 3× from 20 real fps) where the old PNG
wrapper managed about 18 on the same file.

| tool | engine | fast-clip VMAF | out fps |
|---|---|---|---|
| `interpolate-resolve --quality speed-warp` | Resolve Speed Warp (Neural Engine) | **92.8** | 10.3 |
| `interpolate-resolve` | Resolve Optical Flow | 87.6 – 89.8 | 61 – 145 |
| `smooth-fps` | ffmpeg minterpolate, segment-parallel, CPU | 86.4 | 50.7 |
| rife v4.6 (removed PNG wrapper) | rife-ncnn-vulkan v4.6 | 57.6 | 8.6 |
| (nothing — frame duplication) | — | 30.5 | — |

## How this was measured

Guessing at interpolation quality by eye does not work, so this used real
ground truth. A true-60fps 1080p source was verified frame-unique with
`mpdecimate` (360/360 unique — many "60fps" files are 30fps padded, which
would make the whole exercise meaningless). Two 6-second segments were taken,
one fast-motion and one slow. Each was decimated to 30fps losslessly, then
each tool rebuilt 60fps from it, and the result was compared against the
original 60fps frames.

Scores are **on the invented frames only** (`select='mod(n,2)'`). Including the
frames that were simply copied through roughly halves the apparent difficulty
and flatters everything equally.

**Read VMAF, not PSNR, when comparing across tools.** Rendering the ground
truth through Resolve *unchanged* scores 40.2 dB PSNR but VMAF 99.99 — its
colour pipeline imposes a hard ~40 dB PSNR ceiling while being perceptually
lossless. So PSNR understates Resolve and cannot be compared against the
ffmpeg/PNG path, whereas VMAF is directly comparable.

## Full results, invented frames only

| method | VMAF fast | VMAF slow | out fps |
|---|---|---|---|
| Resolve SpeedWarp/better | **92.83** | **97.51** | 10.3 |
| Resolve SpeedWarp/faster | 91.20 | 93.45 | 30.6 |
| Resolve OF enhanced/better | 89.78 | 92.17 | 61.2 |
| Resolve OF enhanced/faster | 88.82 | 91.73 | 120.6 |
| Resolve OF standard/better | 88.80 | 92.19 | 102.7 |
| ffmpeg minterpolate mci | 88.32 | 92.74 | 13.2 serial / 50.7 parallel |
| Resolve OF standard/faster | 87.62 | 91.65 | 145.0 |
| Resolve frame blend | 39.61 | 83.50 | 277.9 |
| rife v4.6 | 57.56 | 82.51 | 20.1 |
| rife v4.18 | 57.47 | 83.33 | 11.7 |
| rife v4.15-lite | 56.83 | 83.16 | 17.1 |
| rife v4.22 | 56.68 | 82.44 | 10.5 |
| rife v4.26-large | 55.34 | 81.92 | 9.3 |
| rife v4.26 | 55.29 | 81.76 | 12.1 |
| rife v4.25-heavy | 55.26 | 82.32 | 9.3 |
| rife v4.25 | 55.10 | 81.61 | 11.9 |
| rife v4.25-lite | 54.17 | 80.99 | 13.0 |
| frame duplication | 30.48 | 71.03 | — |

`out fps` is output frames per second end-to-end, including encode. On the slow
clip Resolve's *frame blend* (83.50 at 278 out fps) matches the best rife model
running at a twelfth of the speed.

## Frame rate and resolution: 24→60 and 30→60, at 1080p and 4K

Measured 2026-09-12 on the same machine with `bin/interpolation-benchmark`,
against a **4K 120 fps** master (verified 240/240 unique frames in both windows).
A 120 fps source is what makes this table possible: 120 divides evenly by 60,
30 *and* 24, so every rate is an exact decimation of one original and 24→60 gets
real ground truth instead of throughput only.

Two 2-second windows, 20 s and 120 s into the clip; cells show the range across
both. 4K was run against the Resolve modes only. The frame-duplication baseline
lands at 27–41 VMAF, so this is fast-motion content.

### Quality — VMAF on the invented frames only

| method | 1080p 24→60 | 1080p 30→60 | 2160p 24→60 | 2160p 30→60 |
|---|---|---|---|---|
| Resolve Speed Warp Better | **88.03 – 91.07** | **92.60 – 94.90** | 86.61 – 89.44 | 90.44 – 92.44 |
| Resolve OF Enhanced Better | 85.73 – 88.73 | 90.64 – 93.17 | **88.02 – 89.78** | **91.35 – 92.83** |
| Resolve OF Standard Better | 84.49 – 88.15 | 89.62 – 92.79 | 86.79 – 89.01 | 90.15 – 92.11 |
| `rife-vapoursynth` v4.6 | 81.43 – 82.27 | 87.71 – 87.91 | — | — |
| `smooth-fps` (minterpolate) | 76.36 – 79.10 | 85.59 – 88.41 | — | — |
| frame duplication | 27.09 – 35.52 | 31.23 – 40.77 | — | — |

1080p columns use `vmaf_v0.6.1`, 4K columns `vmaf_4k_v0.6.1`. **Read down a
column, never across those two** — they are different models.

### Throughput — output frames per second, end to end

| method | 1080p 24→60 | 1080p 30→60 | 2160p 24→60 | 2160p 30→60 |
|---|---|---|---|---|
| Resolve OF Standard Better | 28.0 – 34.7 | 27.6 | 18.1 | 17.1 – 17.4 |
| Resolve OF Enhanced Better | 22.5 – 26.8 | 22.3 – 22.6 | 12.6 | 11.2 |
| `rife-vapoursynth` v4.6 | 15.6 – 15.8 | 22.7 – 23.0 | — | — |
| `smooth-fps` (minterpolate) | 12.3 – 13.8 | 18.4 | — | — |
| Resolve Speed Warp Better | 6.9 – 7.4 | 8.9 | 1.9 – 2.0 | 2.3 |
| frame duplication | 93.0 – 96.8 | 85.1 – 88.9 | — | — |

### What this says

**At 4K, Speed Warp stops being worth it.** It wins at 1080p by 2–4 VMAF, but at
2160p *Enhanced Better beats it* on both ratios — 88.02–89.78 against
86.61–89.44 at 24→60 — while running **5× faster** (12.6 vs 1.9 out fps). At 4K
the ordering flips and the expensive mode is also the losing one.

**24→60 is the harder job, by about 4 VMAF everywhere.** It invents 4 of every 5
frames where 30→60 invents 1 of 2, and every method pays for it by a consistent
margin. Budget for 24 fps sources being meaningfully worse, not just slower.

**Standard Better is the value pick.** It is within ~1 VMAF of Enhanced across
all four configs and runs 1.3–1.5× faster; at 1080p30 it is faster than plain
`rife` while scoring 2–5 VMAF higher.

**4K costs less than the pixel count suggests.** Resolve Standard goes 27.6 →
17.1 out fps for 4× the pixels — 1.6×, not 4× — because a short job is
substantially fixed cost. Speed Warp is the exception at ~4× slower, being the
only mode whose work actually scales with resolution.

**Content matters more than any of this.** These are two windows of one clip.
The 2026-09-10 table above, on different footage, put `rife` at 57.6 VMAF on
fast motion where it scores 81–88 here. Re-run the harness on your own footage
before trusting any ordering.

## Things worth knowing

**Newer rife models are not better.** Of nine models spanning v4.6 to
v4.26-large, **v4.6 was both the most accurate and the fastest**. v4.25-lite is
0.65 dB worse at 35% slower; v4.26 is 0.48 dB worse at 40% slower. Nothing in
the v4.15–v4.26 range beats v4.6 by more than noise, and all of them cost
1.2–2.2× the time. Don't chase model versions without measuring on your own
footage.

**rife's TTA mode (`-x`) is not worth it:** 8.3× slower for +0.04 dB PSNR and
*slightly worse* VMAF. UHD mode (`-u`) changes nothing at 1080p.

**rife cannot be made faster by parallelising it.** It is genuinely GPU-bound —
92.6% mean GPU utilisation at `-j 4:4:4`. Running concurrent processes on
disjoint frame ranges yields 18.9 → 20.7 → 21.2 → 17.1 aggregate out fps for
1 → 2 → 3 → 4 processes: +12% at best, and *worse* than a single process at
four. The GPU is the wall.

**rife also cannot reach the Neural Engine.** `rife-ncnn-vulkan` statically
links MoltenVK, so it runs Vulkan translated to Metal and only ever touches the
GPU. Resolve's Speed Warp renders at 17.5% GPU utilisation while standard
optical flow sits at 76.4% — Speed Warp is running on the ANE, hardware
ncnn/Vulkan has no access to. That is the structural reason it wins.

**minterpolate is single-threaded**, which is why `smooth-fps` exists: identical
runtime at `-filter_threads` 1, 4, 8 and 14. Splitting the video into segments
and running them concurrently is the only way to use the other cores, and it
scales nearly linearly (12.7 → 24.0 → 42.5 → 66.1 out fps for 1/2/4/8
segments on filter-only work; 4.0× end-to-end including encode).

**`me_mode=bilat` beats `bidir`** — 0.6 VMAF better *and* 1.6× faster. It is
ffmpeg's default and `smooth-fps` keeps it.

**Comparing two clips needs a shared timebase, not just matching PTS.**
`setpts=N/60/TB` evaluates `TB` in each stream's own timebase, so scoring a
24 fps file (timebase 1/12288) against a 60 fps one (1/15360) puts them on
different timelines and `framesync` silently pairs the wrong frames — ffmpeg
says "not matching timebases found ... results may be incorrect" and carries
on. It cost a PSNR of 23.5 where the frames were provably bit-identical, and
it was understating Resolve's VMAF by ~20 points because Resolve renders HEVC
against an x264 reference. Pin both sides with `settb=1/60` before `setpts`.

**Resolve's process is named `Resolve`, not `DaVinci Resolve`.** So
`pgrep -x "DaVinci Resolve"` never matches, and `interpolate-resolve` was
re-launching the app and sleeping 6 s on *every* render even when it was
already open — which is most of what looked like a fixed per-render cost.
Match on the executable path instead:
`pgrep -qf "DaVinci Resolve.app/Contents/MacOS/Resolve"`.

**Resolve's first render after launch is not representative.** Measured at
463 s, and twice past a 600 s timeout, against ~4 s for the identical job
immediately afterwards. The harness burns one discarded render before it times
anything; do the same in any script that benchmarks it.

**ffmpeg 9's `ffprobe` reports every stream twice** — once wrapped in a
`[STREAM_GROUP]` and once standalone — so `-show_entries stream=...` returns two
lines per stream. Any script that reads a single value into a shell variable
silently gets `"60\n\n60"`, which then breaks arithmetic and `awk -v`. This is
what made `smooth-fps` and `rife-vapoursynth` exit non-zero on their final stats
line; both now trim every probe to its first line.

**Beware `-video_track_timescale` on a concatenated stream.** Rescaling the
per-frame timestamps rounds them unevenly and cost 25 VMAF points (86.4 → 61.3)
with the frame count intact — a silent, invisible-in-metadata corruption.

**Beware remuxing raw H.264 with `-framerate`.** A raw Annex-B stream carries no
timestamps, so this drops frames: 30 in, 28 out in a controlled test. Frame
*order* survives, but a single dropped frame shifts everything after it.
`smooth-fps` joins with the concat demuxer instead, which is exact. The removed
PNG wrapper had this bug (358 frames instead of 360, 33 ms short);
`rife-vapoursynth` never did, because the timing comes straight from the y4m
frame rate and there is nothing to remux.

## Reproducing

`bin/interpolation-benchmark` is the harness. What is still
machine-specific is the *source*: it must be genuinely 60 fps, because a file
that is 30 fps padded to 60 makes every score meaningless. The harness checks
that with `mpdecimate` before it starts and warns if the source is padded.

```bash
bin/interpolation-benchmark ~/Movies/clip.mp4
bin/interpolation-benchmark -c 1080p30:30:1080 -m rife ~/Movies/clip.mp4
bin/interpolation-benchmark -s fast:12:2 -s slow:40:2 ~/Movies/clip.mp4
```

It writes `results.csv`, `report.md`, and a `machine.md` block recording the
hardware, macOS build, tool versions and dotfiles commit — so a number is never
separated from the machine that produced it.

Per configuration it:

1. builds a lossless (qp 0, keyint 1) 60 fps ground truth from the window,
2. derives the lower-rate input from it and **proves it is bit-identical** to
   the ground-truth frames it stands for (PSNR must be `inf`, or the harness is
   measuring itself instead of the tools),
3. runs each tool on that input,
4. scores **only the invented frames** (`select='mod(n,k)'`) against the
   originals.

Adding a tool is three edits, all next to each other: its id in `method_ids()`,
and an arm in `method_label()`, `method_needs()` and `method_cmd()`. A tool
whose binary or app is missing is skipped with a reason instead of failing the
run.

**Ground truth needs the input rate to divide 60 exactly.** 30→60 does — drop
every second frame. 24→60 does not: 2.5 is not an integer, so no subset of a
60 fps clip is a true 24 fps version of it. Those configurations still run,
from a real 24 fps decimation of the source, and report throughput only. Scoring
24→60 quality properly would need a true 120 fps source, which decimates to both
24 and 60.

**VMAF models are not interchangeable.** The default `vmaf_v0.6.1` is trained
for 1080p viewing; the harness switches to `vmaf_4k_v0.6.1` at 4K and records
which model produced each number. Compare down a column, never across the two.

Sanity checks that catch a broken harness: the low-rate input must be
bit-identical to the ground-truth frames it stands for (`psnr` reports `inf` —
the harness asserts this), and the frame-duplication baseline must land near 30
VMAF on fast motion.

```bash
# Disk: ground truth is lossless, ~65 MB per second at 1080p and ~260 MB at
# 2160p, which is why the default window is 2 seconds.
bin/interpolation-benchmark --dry-run ~/Movies/clip.mp4
```
