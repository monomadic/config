# avupscale

Video super-resolution using Apple's on-device model
(`VTSuperResolutionScalerConfiguration`). macOS 26+, Apple silicon.
The model is an on-demand system asset; the CLI downloads it on first run.

```sh
scripts/install/install-avupscale.sh
avupscale --list in.mp4 /dev/null       # supported scale factors, model status
avupscale in.mp4 out.mov -s 2           # 2x, ProRes 422 (default)
avupscale in.mp4 out.mp4 -s 2 -c hevc --mbps 40
avupscale in.mp4 out.mov --no-temporal  # per-frame ("image") mode
```

Notes
- One quality level: the SDK only defines `.normal` for this scaler.
- Video mode feeds the previous input/output frame back in (temporal
  stabilisation). Use `--no-temporal` for content with hard cuts every few
  frames, or to A/B against Real-ESRGAN which is per-frame.
- Benchmark idea: same 10s 1080p clip → this, `realesrgan-ncnn-vulkan -s 2`,
  Topaz Proteus. Compare `ffmpeg -i a.mov -i b.mov -lavfi ssim` and eyeball
  temporal flicker at 400% zoom.
- Audio dropped; mux with ffmpeg as in avinterp/README.md.

Sources
- https://developer.apple.com/documentation/videotoolbox/vtsuperresolutionscalerconfiguration
