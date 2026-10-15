# src/utils — Apple-silicon video ML CLIs

| dir | does | Apple API | model |
|-----|------|-----------|-------|
| avinterp | frame interpolation | VideoToolbox VTFrameRateConversion | built into macOS |
| avupscale | super-resolution | VideoToolbox VTSuperResolutionScaler | downloaded on demand |
| rife-video | streaming RIFE frame interpolation + video encoding | VideoToolbox decode/encode + Metal/NCNN Vulkan | pinned RIFE models |
| avremove | per-frame object removal | Vision + Core ML | LaMa (convert_lama.py) |

The original three Apple-ML tools are Swift packages targeting macOS 26, with
swift-argument-parser as the only dependency. Build + install with `scripts/install/install-<name>.sh`
(→ `~/.local/bin`). `VideoIO.swift` is duplicated in each on purpose so the
folders stay independent — edit one, copy it to the other two.

`rife-video` is independent of the shared `VideoIO.swift`; it uses explicit
hardware decoding, rational timestamp scheduling and a pinned upstream RIFE
library. See [its README](rife-video/README.md) for build and integration checks.

Original Apple-ML tools status: every source file typechecks against the macOS
27 SDK headers. None has been built or run yet. See NOTES.md for what that does and doesn't cover.
