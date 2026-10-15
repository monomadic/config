# Resident NCNN benchmark — 2026-10-15

Apple M4 Pro (20 GPU cores, 48 GB), macOS 26.5. Native ARM64 Release + LTO;
NCNN `ec19da2b`, RIFE `13338e38`, statically linked MoltenVK 1.4.2. Both backends
run in `rife-video` with the same hardware decoder, rational timestamp scheduler,
hardware HEVC writer, bitrate, audio mux and output publication. No PNGs in the
video path. All GPU runs are sequential, not competing benchmark processes.

## Original real clip, HQ

The supplied `amona_re [DccFm9MDYa4_1].mp4` is 720×720, 30 fps, 235 frames,
7.833 s. Each run produces 470 frames at 60 fps: 234 synthesized, 234 coincident
source frames and 2 held tail frames. These are startup-inclusive full-pipeline
measurements. Median of three passes:

| Backend | Workers | Wall seconds | Output fps |
|---|---:|---:|---:|
| NCNN 4.26, FP32 arithmetic / FP16 storage | 1 | 7.385 | 63.64 |
| NCNN 4.26, FP32 arithmetic / FP16 storage | 2 | 6.734 | 69.80 |
| NCNN 4.26, FP32 arithmetic / FP16 storage | 3 | 6.746 | 69.67 |
| NCNN 4.26, FP32 arithmetic / FP16 storage | 4 | 6.772 | 69.40 |
| Metal 4.26, HQ | 1 | 5.043 | 93.21 |

Auto selects two workers below full 4K: more showed no useful improvement on this clip.
This narrows the earlier gap, but Metal remains about 1.34× faster end-to-end.
The previous PNG-directory NCNN time included a different I/O pipeline; do not
interpret the difference as a pure kernel speedup.

## Longer repeated-clip run

A compressed, video-only fourfold repeat of the real clip gives 31.333 seconds,
940 source frames and 1,880 output frames (939 synthesized). No frame images are
staged. Two passes through the installed command, with HQ and hardware HEVC:

| Backend | Median wall | Median output fps |
|---|---:|---:|
| NCNN 4.26, 2 workers | 26.83 s | 70.09 |
| Metal 4.26 HQ | 20.93 s | 89.85 |

Metal is 1.28× faster in this longer matched pipeline run. This amortizes startup;
it is still one short clip repeated, not a broad content or thermal endurance study.

## Arithmetic and output checks

A second two-pass sweep tested optional FP16 arithmetic. Two-worker FP16 runs
took 6.967/6.987 s; conservative arithmetic took 8.052/6.963 s in that sweep.
There is no consistent FP16 speed advantage in these runs; startup/system
variation matters. FP16 remains opt-in rather than silently changing precision.

Three identical RGB input-pair comparisons against the installed NCNN PNG CLI
(at phases 0.4 and 0.5) measured 63.65–66.96 dB RGB PSNR for native conservative
arithmetic, with mean absolute differences of 0.013–0.025 on the 0–255 scale.
The native backend is close, **not bit-identical**, to that older binary. FP16
arithmetic measured 46.42–48.78 dB and 0.256–0.371 mean absolute difference.
These measure implementation agreement, not perceptual quality against ground
truth. The earlier held-out reconstruction quality scores remain separate.

Decoded HEVC outputs from one versus four workers were pixel-identical for all
470 frames (infinite PSNR), confirming output order and concurrency consistency
on this fixture. Temporary PNGs are used only by the reference parity test.

## Resolution probes

Two-second fixtures scaled/padded from the supplied clip measure resolution
cost, not native high-resolution reconstruction quality. 1080p is H.264/24 fps;
4K is 10-bit HEVC/50 fps (interpolation remains 8-bit SDR). Both output 60 fps
hardware HEVC. These short probes include model startup and are not long-duration
thermal/throughput claims.

| Input → 60 fps | NCNN 1 worker | NCNN 2 workers | Metal HQ |
|---|---:|---:|---:|
| 1920×1080 / 24 fps | 12.66 fps | 13.04 fps | 15.39 fps |
| 3840×2160 / 50 fps | 3.15 fps | 3.06 fps | 3.70 fps |

The two 4K passes do not establish a decisive concurrency winner: two workers
were slower in the first pass and slightly faster in the second. Auto uses one
at full 4K to avoid extra concurrent model workspace without a demonstrated
throughput benefit. Override with `--workers` and measure on other hardware.
HQ is not real-time at either tested large resolution. A single additional 4K
fast-model probe (4.25-lite) gave 3.64 fps with one worker and 3.70 with two: a
small concurrency difference, also well below real-time. The auto choice of one
full-4K worker prioritizes avoiding extra workspace when no repeatable gain has
been demonstrated.

## Reproduction

```sh
python3 Tests/benchmark-ncnn.py .build/release/rife-video input.mp4 /tmp/new-run --fp16
python3 Tests/integration.py .build/release/rife-video --backend ncnn --large
# numpy and Pillow are required only for this reference test:
python3 Tests/ncnn-parity.py .build-deps/native/librife_native.dylib input.mp4 \
  --model "$HOME/.local/share/rife-ncnn-vulkan/rife-v4.26" --report /tmp/parity.json
```

For sustained measurements use a long real source. Keep encoder/audio settings,
model, precision and source identical; compare `wall_s` and
`output_fps_end_to_end`. NCNN's `hq/balanced/fast` models differ from Metal's
motion-estimation resolution tiers, so same tier labels do not establish
quality equivalence.

## Validation

The installed CLI passed 12 NCNN hardware integration cases, including all three
output codecs, all model tiers, 24/25/30/50 fps, 23.976→59.94, delayed audio,
rotation, VFR, MKV remux, 1080p and 4K. Ten Metal regression cases also passed.
The tests verify decoded frame counts/rates, dimensions, timing, hardware
requirements and copied-frame fidelity. Missing/corrupt model, invalid workers,
existing-output preservation and HDR rejection checks passed. Scheduler tests,
shell syntax, and repository setup checks passed.
