# avinterp

ML frame interpolation using Apple's on-device optical-flow model
(`VideoToolbox.VTFrameProcessor` + `VTFrameRateConversionConfiguration`).
The API is macOS 15.4+; the package targets macOS 26. Apple silicon only.
No model download needed — it's in the OS.

```sh
scripts/install/install-avinterp.sh  # → ~/.local/bin/avinterp
avinterp in.mp4 out.mp4              # 2x fps
avinterp in.mp4 out.mov -f 4 -c prores422
avinterp in.mp4 slow.mp4 -f 4 --fps 30   # 4x slow-motion at 30 fps
avinterp in.mp4 out.mp4 --precompute-flow   # separate VTOpticalFlow pass, slower
```

Notes
- Limits: 8192x4320 on macOS. Output timestamps are regenerated (CFR); VFR
  input is treated as CFR at its nominal rate — remux to CFR first if needed:
  `ffmpeg -i in.mp4 -vsync cfr -r 30 cfr.mp4`
- Audio is dropped (video-only writer). Mux back:
  `ffmpeg -i out.mp4 -i in.mp4 -map 0:v -map 1:a -c copy final.mp4`
- Compare against RIFE: `rife-ncnn-vulkan -i frames/ -o out/ -m rife-v4.6`

Sources
- https://developer.apple.com/documentation/videotoolbox/vtframerateconversionconfiguration
- WWDC25 session 300, "Enhance your app with machine-learning-based video effects"
