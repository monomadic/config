# rife-video

Streaming RIFE interpolation for Apple silicon, with hardware H.264/HEVC decode,
hardware HEVC/ProRes encode, or software SVT-AV1 encode. Frames stay in pixel
buffers; AV1 receives raw BGRA through a bounded pipe. No PNGs or raw-frame disk
staging. MKV/TS inputs and `hev1`-tagged HEVC are losslessly remuxed to a
compatible MOV for native decoding when needed. A compressed video is staged beside the destination for audio muxing
and atomic publication.

```sh
scripts/install/install-rife-video.sh
rife-video input.mp4 output.mp4 --codec hevc --tier hq --stats hevc.json
rife-video input.mp4 output.mov --codec prores --prores-profile hq --tier balanced
rife-video input.mp4 output.mkv --codec av1 --crf 25 --preset 8 --tier fast
```

Default target is **60 fps**, preserving playback speed. `--fps 60000/1001` is
also supported. Scheduling uses rational source presentation timestamps, not
an integer doubling factor: e.g. 24→60 generates phases 0.4 and 0.8 in the first
source interval. Fractional rates and variable frame timing use the same path.
Exact matching source timestamps are copied. The last source frame is held
through its sample/track duration; the output duration rounds up by less than
one output frame. No motion is extrapolated past the final frame.

## Options

- `--tier hq|balanced|fast`: full-, half-, quarter-resolution motion estimation;
  output always retains source dimensions. Tier changes speed and quality.
- `--codec hevc|prores|av1`: HEVC/ProRes require hardware encoding and fail if it
  is unavailable. AV1 is software on M4 Pro; it needs FFmpeg with `libsvtav1`.
- `--mbps N`: HEVC target bitrate; default 20 Mbps at 1080p, scaled with pixel
  count, minimum 2 Mbps. ProRes uses `--prores-profile` (`proxy`, `lt`, `standard`,
  `hq`, `4444`, `xq`). AV1 uses `--crf 25 --preset 8` by default.
- `--audio copy|aac|none`: default copies all audio tracks with original relative
  timing. Unsupported audio/container combinations fail; select AAC explicitly
  when needed. Subtitles, data tracks and chapters are not copied.
- `--stats FILE`: benchmark JSON. Must be a new file distinct from source/output.
- `--overwrite`: replace an existing output atomically **only after success**.
  Without it the output is never replaced, even if another job creates it during
  the encode. Input/output cannot be the same path.
- `--model PATH`: optional matching RIFE 4.26 `.rmw` model override.

Output: `.mp4`, `.mov`, `.mkv`; ProRes requires `.mov`. FFmpeg is required for final
muxing even for native hardware output. No Homebrew package is installed by the
installer. Source and destination filenames are passed as arguments, never shell
commands.

## Pixel and timeline contract

This version is **8-bit BT.709 SDR**, because upstream RIFE accepts BGRA8/RGBA8.
10-bit SDR decoding is possible but interpolation reduces precision to 8-bit;
ProRes's higher storage precision cannot recover that precision. PQ/HLG HDR,
other explicitly tagged color spaces and non-square pixels fail before encoding.
Untagged sources are assumed BT.709. No automatic tone mapping is performed.

H.264/HEVC hardware decode is required via VideoToolbox's
`RequireHardwareAcceleratedVideoDecoder`, then confirmed via
`UsingHardwareAcceleratedVideoDecoder`. Compressed samples are decoded in decode
order and delivered in PTS order, using DTS as a watermark, with a bounded
fallback for missing DTS. Midstream format changes and out-of-order/duplicate
PTS fail instead of silently changing timing. Gaps longer than ten seconds or requiring over sixteen output frames in one
source interval fail.

Hardware output uses AVAssetWriter with
`RequireHardwareAcceleratedVideoEncoder`; there is no software fallback. Native
output preserves the track display transform. The AV1 path restores right-angle
rotation in the final mux; mirrored/scaled transforms are rejected for AV1.

One model/stream stays loaded. A bounded decode-ahead worker overlaps decoding
with interpolation and the encoder's own buffering. Pixel buffers and the AV1
pipe bound memory; the whole video is never collected in RAM. The library may
copy/convert pixels internally: **no PNG intermediate does not mean zero-copy**.

Scene-cut detection is not implemented; interpolation across a cut can blend
unrelated frames. There is no claim of real-time HQ performance at 1080p/4K.

## Timing / later NCNN comparison

`wall_s` and `output_fps_end_to_end` include probe, model load, hardware decode,
RIFE, encoding, audio muxing and output publication. Use these for matched
whole-pipeline comparisons: same source, fps, pixel/color settings, encoder,
bitrate/preset, audio and overwrite/storage behavior.

`interpolation_call_s` includes the first graph warm-up and cache-rebase calls;
`stream_prime_s` is reported separately. `generated_frames` excludes copied and
held frames. `decode_call_s`, `writer_append_s`, `encoder_flush_s`, and `mux_s`
expose other costs. **Stages overlap; do not sum their times.** Compare a separate
resident NCNN API timer to RIFE's API time, not NCNN's PNG-directory wall time.

## Build and test

```sh
cd src/utils/rife-video
./prepare.sh
swift build -c release
.build/release/timeline-tests
python3 Tests/integration.py .build/release/rife-video
```

`prepare.sh` fetches upstream `cinemore/rife-metal` at commit
`1fef4869a44fca32848102b9f6a774a20eb50db6` (0.1.7, RIFE 4.26). A verified SHA-256
release archive supplies its precompiled Metal shader library, so full Xcode's
Metal compiler is not required. The generated dependency manifest only changes
resource/build integration; upstream algorithm sources are unmodified. All
upstream source/models/shaders remain under ignored `.build-deps/`.

The installer puts the executable and model/shader bundles together under
`~/.local/libexec/rife-video/`, with a launcher at `~/.local/bin/rife-video`.
Installing is separate from dotfile deployment; no Deployfile mapping changes.

The scheduler tests use a standalone executable because Command Line Tools may
not ship XCTest. Integration tests create disposable H.264/HEVC fixtures and
exercise all codecs/tiers, fractional rates, audio, rotation, and safe failures;
they require GPU access and the actual hardware encoders.

## Validation on 2026-10-15

Built and installed on M4 Pro (20 GPU cores), macOS 26.5 with SwiftPM.
Scheduler checks passed for 24/25/30/50 fps, fractional 23.976→59.94 over 1001
seconds, VFR phases, final-frame hold, and invalid timestamps/rates.

Twelve distinct hardware integration cases passed across the codec/tier matrix,
10-bit HEVC input, fractional rates, delayed audio, rotated HEVC/AV1, MKV and VFR
input, and short native 1080p/4K smoke conversions. Tests checked decoded frame
counts, exact output rates, duration, dimensions, rotation, audio offsets,
hardware requirements, and copied-frame fidelity (>35 dB PSNR on the small test
fixture). Existing-output preservation and HDR rejection also passed. These are
functional checks, not a sustained 1080p/4K benchmark.

The installed executable also converted a 235-frame 720×720/30 fps clip to 470
HEVC frames at exactly 60 fps in 4.84 seconds end-to-end, using HQ. The model was
loaded from the installed resource bundle. A matched NCNN benchmark remains a
separate next step.
