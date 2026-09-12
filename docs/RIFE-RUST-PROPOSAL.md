# Proposal: RIFE frame interpolation as a Rust tool

Status: **proposal, not approved**. Written 2026-09-11 against the measurements
in [INTERPOLATION.md](INTERPOLATION.md).

`rife-60fps` appears below only as history — it was the PNG-per-frame wrapper
this proposal argued against, and it has since been removed. The two RIFE
things that exist now are the upstream `rife-ncnn-vulkan` binary and
`bin/rife-vapoursynth`, which is the tool to use.

## Recommendation in one paragraph

Do not build a Rust port of `rife-ncnn-vulkan`. It would reproduce the engine
we already have, and that engine loses on this machine on both quality and
speed: RIFE through ncnn/Vulkan scores VMAF 57.6 on fast motion where ffmpeg
`minterpolate` scores 86.4 and Resolve Speed Warp 92.8, and it is GPU-saturated
so no amount of pipeline work makes it faster than about 20 output frames per
second at 1080p. The only Rust project worth considering is one that runs RIFE
on the **Neural Engine through Core ML**, because the Neural Engine is the
structural reason Speed Warp wins and it is unreachable from Vulkan. That
project has a real unknown at its core (whether RIFE's warp operator compiles
to the ANE at all), so it should start as a one-day feasibility spike with a
numeric go/no-go, not as a build.

## How we got here

1. **`rife-60fps` (2026-09-10).** Wrapped the existing `install-rife-arm64`
   binary. That binary only reads and writes image files, so the pipeline was
   ffmpeg → PNG → rife → PNG → x264, chunked through a RAM disk because the
   frames for one 1080p minute do not fit on the boot disk. Measured end to end
   at 8.6 out fps on the fast clip. Two defects were later found: remuxing the
   raw H.264 stream with `-framerate` drops frames (358 of 360), and
   `-video_track_timescale` on a concatenated stream corrupts timestamps
   invisibly (25 VMAF points).
2. **The assumption that image I/O was the bottleneck was wrong.** Profiling
   showed `rife-ncnn-vulkan` at 92.6% GPU utilisation with `-j 4:4:4`, and
   running two, three and four processes on disjoint ranges gave 18.9 → 20.7 →
   21.2 → 17.1 aggregate out fps. Removing every PNG pass would at best move
   the end-to-end number from 8.6 to roughly the engine's own 20.
3. **The assumption that newer models would score better was also wrong.**
   Nine models from v4.6 to v4.26-large were measured against true-60fps
   ground truth. v4.6 was the most accurate *and* the fastest. Nothing newer
   beat it by more than noise and everything newer cost 1.2 to 2.2× the time.
4. **`smooth-fps` replaced it.** `minterpolate` is single-threaded, so it is
   run on concurrent segments with a motion-estimation run-up and global-time
   trimming. 50.7 out fps, VMAF 86.4, no GPU.
5. **The question then became whether to write RIFE in Rust or build the
   VapourSynth plugin.** They are the same engine, so the quality is identical
   and the speed is within a few percent. That answer is still true. What the
   benchmark adds is that the shared engine is the wrong target.

## What the measurements say

Invented frames only, fast-motion 1080p clip, M4 Pro. Full table and method in
[INTERPOLATION.md](INTERPOLATION.md).

| engine | VMAF | out fps | compute |
|---|---|---|---|
| Resolve Speed Warp | 92.8 | 10.3 | Neural Engine (17.5% GPU) |
| Resolve Optical Flow | 87.6 – 89.8 | 61 – 145 | GPU (76% GPU) |
| ffmpeg minterpolate, `smooth-fps` | 86.4 | 50.7 | CPU, all cores |
| rife v4.6 via ncnn/Vulkan | 57.6 | 20.1 engine, 8.6 end to end | GPU (92.6%) |

Three conclusions follow directly:

- **A wrapper cannot change VMAF.** Same weights, same operators, same
  numbers. Rust, VapourSynth or the CLI produce the same frames.
- **A wrapper cannot beat the GPU wall.** The engine's ceiling is about 20 out
  fps. That is under `smooth-fps` and a seventh of Resolve Optical Flow.
- **The interesting hardware is the one ncnn cannot use.** Speed Warp is a
  learned model too, and it is both the best and the one running on the ANE.

## Why RIFE scores so low here, and what that means for a port

RIFE is trained on Vimeo-90K at 448×256 and evaluated in its paper on
similar low-resolution benchmarks. At 1080p the flow network's receptive field
covers a much smaller fraction of the frame, so large fast motion is
underestimated and the warped result smears. `minterpolate` and Resolve's
optical flow are block or pyramid based and scale with resolution. Two
consequences for any port:

- Running the same weights faster does not fix the score. Only a different
  model, or the ANE budget to run a heavier one, could.
- The UHD flag, which downsamples for flow estimation, "changes nothing at
  1080p" in measurement, so the cheap fix is already known not to work.

## Options for a Rust implementation

### A. ncnn through Rust bindings

The path most people mean by "RIFE in Rust". Load the `.param`/`.bin` pairs
already installed under `~/.local/share/rife-ncnn-vulkan` and run them with
ncnn's Vulkan backend.

- **Bindings are stale.** `ncnnrs` 0.1.7 (2024-09), `ncnn-sys` 0.2.0
  (2023-02), `ncnn-bind` and `ncnn-rs` 0.1.1 (2021-12). None tracks current
  ncnn, and none exposes custom-layer registration ergonomically.
- **RIFE needs custom layers.** `rife-ncnn-vulkan` defines a `Warp` layer with
  its own Vulkan compute shaders (`warp.comp`, `warp_pack4.comp`,
  `warp_pack8.comp`) plus preprocess, postprocess, timestep and flow-averaging
  shaders. The model files reference these layers. A port has to register
  them through the ncnn C API and ship the SPIR-V, or run the warp on the
  CPU, which at 1080p would cost more than the PNG passes did.
- **Result: same 57.6 VMAF, same 20 fps ceiling**, with a C++ build of ncnn
  inside a Cargo build and roughly a week of work. Not worth it.

### B. ONNX Runtime with the Core ML execution provider (`ort` crate)

Export the IFNet to ONNX, run it with `ort` and the Core ML provider.

- The `ort` crate is current and well maintained, and the Core ML provider
  can place supported subgraphs on the ANE.
- The blocker is operator coverage: RIFE's warp is `grid_sample`, and the
  Core ML provider has historically not accepted it, so the graph splits and
  the warp falls back to CPU between ANE segments. Each split is a copy across
  the device boundary. Whether this ends up faster than Vulkan is unknown and
  may well be slower.
- Worth trying only as a variant inside the spike below, because the export
  is a few lines once the Core ML conversion exists.

### C. Core ML directly through `objc2` (fits the repo)

Convert the IFNet with `coremltools` to an `.mlpackage`, then call
`MLModel` from Rust through `objc2` and `objc2-core-ml`, the same binding
family the menu bar widgets already use. Feed frames from ffmpeg over a raw
video pipe, write results to a second ffmpeg process over another pipe. No
image files, no chunking, no scratch disk, no C++.

- **Upside if it works:** the ANE has a separate power and compute budget from
  the GPU. Speed Warp shows a learned interpolator can run there at usable
  speed. It would also leave the GPU free for the encoder.
- **The unknown is `grid_sample`.** `coremltools` converts it, but the
  Neural Engine compiler may refuse it and schedule that op, and everything
  downstream of it, on the GPU or CPU. RIFE calls warp inside each of its four
  refinement blocks, so a fallback there means most of the network runs off
  the ANE and the tool is no faster than ncnn.
- **Shape is fixed at conversion.** One `.mlpackage` per resolution, or
  flexible shapes at a compile-time cost. Acceptable for a personal tool with
  two or three resolutions.
- **Timestep input.** v4 models take the interpolation fraction as an input
  tensor, which is what allows arbitrary ratios and proper dedup timing.
  Core ML supports it as a second input.

### D. Candle or Burn with the Metal backend

Reimplement the network in a pure-Rust framework.

- Metal only. The Neural Engine is not exposed to third-party frameworks
  outside Core ML. This is therefore the same GPU wall as ncnn with more work
  and less mature kernels. Dead end for this goal.

## The proposal: a spike with a numeric gate, then maybe a tool

### Phase 0, feasibility spike, about one day, Python only

1. Convert rife v4.6's IFNet to Core ML with `coremltools` at a fixed 1080p
   input shape, `compute_units = ALL`, FP16.
2. Open the model's performance report in Xcode, or use
   `coremltools`'s compute-plan API, and record which ops land on the ANE.
   If `grid_sample` or the blocks after it fall to GPU or CPU, stop here.
3. Run the existing ground-truth harness from INTERPOLATION.md on the fast
   and slow clips, from Python, with the same invented-frames-only scoring.
4. Measure engine throughput at 1080p and GPU utilisation with `powermetrics`
   to confirm the work is on the ANE and not just moved.

Go/no-go, both required:

| metric | threshold | why |
|---|---|---|
| ANE residency | warp and all four blocks on ANE | otherwise no structural gain over Vulkan |
| engine throughput | ≥ 40 out fps at 1080p | must beat `smooth-fps` end to end after encode |

VMAF is expected to be unchanged at about 57.6, because the weights are the
same. That is acceptable for the spike. It means the tool would be a *fast*
interpolator, not a *good* one, and its place would be previews and low-motion
footage, unless a heavier model also fits the ANE budget. Test v4.25-heavy in
the spike too if the first conversion goes on the ANE.

### Phase 1, only on a go: `utils/rife-ane`

- Rust, `objc2` + `objc2-core-ml`, no bundle, installer at
  `setup/install/install-rife-ane.sh`, following `battery-widget`'s pattern.
- ffmpeg decode to `rgb24` or `rgba` over stdin, ffmpeg encode from stdout,
  audio copied in the encoder invocation. No intermediate files at any point.
- Scene-cut detection from frame difference, so cuts are duplicated rather
  than interpolated across. `rife-60fps` never did this.
- Dedup done properly with timestamps: identical frames are dropped from the
  input sequence and the target frame times are interpolated within the
  correct surviving pair at the correct fraction. This fixes the timing warp
  that made `--dedup` unsafe on held frames.
- Output written by one encoder process for the whole file. No raw stream
  concatenation, no timescale rewriting, which avoids both defects found in
  `rife-60fps`.
- Tests in `src/tests/` for the frame scheduler (pair selection and fractions
  for a given source and target rate, with and without dedup), which is the
  only part with logic worth testing in isolation.

### Phase 2

Yazi opener beside the `smooth-fps` ones. Do not make it the default unless it
measures better than `smooth-fps` on the same clips.

## What the VapourSynth plugin will and will not tell us

It is being built anyway, because it is a few hours and the dependencies are
already installed. Expectations, so nobody is surprised by the result:

- It removes the four image-codec passes per frame, so the end-to-end figure
  should move from 8.6 toward the engine's 20 out fps. That is the whole gain.
- VMAF will be the same 57.6 on the fast clip. Confirm it, since a matching
  score is also the proof that the plugin is running the same model correctly.
- Its `sc` scene-change option and `fps_num`/`fps_den` timing are things
  `rife-60fps` lacked and are worth having for whatever RIFE is still used for.
- If the measured number lands near 20, that is the ceiling of ncnn on this
  GPU, and it is the final word on option A above.

Result, 2026-09-11: `rife-vapoursynth` built and measured on the 3-minute 720p clip at
30.0 out fps end to end, exact duration, against about 18 for `rife-60fps` on
the same file. A 1.6× gain from removing the image passes, and consistent
with a GPU-bound engine. Isolated through `vspipe` on a 30 s window at 720p:
decode and colour conversion run at 2192 fps, rife at 59.7 out fps for 2× and
44.8 for 3×, both ≈30 inferences/s, and the x264 encoder adds 3%. So the
non-engine overhead is now about 3% in total, which is the most a Rust
wrapper of the same engine could ever recover. This closes option A on
measurement, not just on argument. The 1080p harness number is still to be
taken.

## Risks and unknowns, ranked

1. `grid_sample` on the ANE. Decides everything. Answered by Phase 0 step 2.
2. FP16 on the ANE degrading flow precision. Check VMAF in the spike against
   the FP32 ncnn number, expect a small loss.
3. Fixed input shapes. Mitigated by converting per resolution on first use
   and caching under `~/.cache`.
4. `objc2-core-ml` API coverage for `MLMultiArray` input binding without
   copies. Worst case one copy per frame, which is cheap at 1080p.
5. The model is still RIFE. If the target is quality, the answer on this
   machine is Resolve Speed Warp, and no Rust work changes that.

## Decision log

- 2026-09-10: `rife-60fps` built on the PNG binary; assumed I/O bound.
  Wrong, and the assumption was never measured before the UI was added.
- 2026-09-10: nine models benchmarked against ground truth; v4.6 best and
  fastest; ncnn engine confirmed GPU-bound and ANE-blind.
- 2026-09-11: `smooth-fps` made the default. `rife-60fps` superseded.
- 2026-09-11: Rust versus VapourSynth judged equivalent in output. Rust
  ncnn port rejected for reproducing a measured loss. Core ML spike proposed
  as the only Rust path with a possible structural gain.

## Sources

- [INTERPOLATION.md](INTERPOLATION.md), the measurements this rests on
- [rife-ncnn-vulkan source](https://github.com/nihui/rife-ncnn-vulkan/tree/master/src), custom `Warp` layer and shaders
- [TNTwise/rife-ncnn-vulkan](https://github.com/TNTwise/rife-ncnn-vulkan), the fork with every model, used by `install-rife-arm64`
- [VapourSynth-RIFE-ncnn-Vulkan](https://github.com/styler00dollar/VapourSynth-RIFE-ncnn-Vulkan), models to v4.26, `sc`, `fps_num`/`fps_den`
- crates.io: `ncnnrs`, `ncnn-sys`, `ncnn-bind`, `ncnn-rs`, `ort`
