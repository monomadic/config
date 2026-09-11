# Frame interpolation on this machine

Measured 2026-09-10 on an Apple M4 Pro (10P + 4E cores, 20-core GPU, 48 GB),
macOS 26.5, ffmpeg 9.0.1, DaVinci Resolve Studio 21.0.2.4.

Three tools in this repo do frame interpolation. **`smooth-fps` is the default
choice**; `interpolate-resolve` is better if Resolve is available and you want
the best quality; `rife-60fps` is superseded and should not be used.

| tool | engine | fast-clip VMAF | out fps |
|---|---|---|---|
| `interpolate-resolve --quality speed-warp` | Resolve Speed Warp (Neural Engine) | **92.8** | 10.3 |
| `interpolate-resolve` | Resolve Optical Flow | 87.6 – 89.8 | 61 – 145 |
| `smooth-fps` | ffmpeg minterpolate, segment-parallel, CPU | 86.4 | 50.7 |
| `rife-60fps` | rife-ncnn-vulkan v4.6 | 57.6 | 8.6 |
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

**Beware `-video_track_timescale` on a concatenated stream.** Rescaling the
per-frame timestamps rounds them unevenly and cost 25 VMAF points (86.4 → 61.3)
with the frame count intact — a silent, invisible-in-metadata corruption.

**Beware remuxing raw H.264 with `-framerate`.** A raw Annex-B stream carries no
timestamps, so this drops frames: 30 in, 28 out in a controlled test. Frame
*order* survives, but a single dropped frame shifts everything after it.
`smooth-fps` joins with the concat demuxer instead, which is exact. `rife-60fps`
still has this bug (358 frames instead of 360, 33 ms short).

## Reproducing

The harness is not checked in — it needs a true-60fps source, which is
machine-specific. The shape of it:

```bash
# 1. verify the source is really 60fps, not 30 padded
ffmpeg -ss "$T" -t 6 -i "$SRC" -vf mpdecimate -fps_mode passthrough -f null -

# 2. lossless ground truth + the 30fps input every tool consumes
ffmpeg -ss "$T" -t 6 -i "$SRC" -an -vf "fps=60,setpts=N/60/TB" \
  -c:v libx264 -qp 0 -x264-params keyint=1 gt.mp4
ffmpeg -i gt.mp4 -an -vf "select='not(mod(n,2))',setpts=N/30/TB" -r 30 \
  -c:v libx264 -qp 0 -x264-params keyint=1 in30.mp4

# 3. score only the invented frames
ffmpeg -i out.mp4 -i gt.mp4 -lavfi \
  "[0:v]select='mod(n\,2)',setpts=N/60/TB,format=yuv420p[a];\
   [1:v]select='mod(n\,2)',setpts=N/60/TB,format=yuv420p[b];[a][b]libvmaf" -f null -
```

Sanity checks that catch a broken harness: the 30fps input must be bit-identical
to every other ground-truth frame (`psnr` reports `inf`), and a frame-duplication
baseline must land near 30 VMAF on fast motion.
