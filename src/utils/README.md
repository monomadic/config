# src/utils — Apple-silicon video ML CLIs

| dir | does | Apple API | model |
|-----|------|-----------|-------|
| avinterp | frame interpolation | VideoToolbox VTFrameRateConversion | built into macOS |
| avupscale | super-resolution | VideoToolbox VTSuperResolutionScaler | downloaded on demand |
| avremove | per-frame object removal | Vision + Core ML | LaMa (convert_lama.py) |

All three are Swift packages targeting macOS 26, with swift-argument-parser as
the only dependency. Build + install with `scripts/install/install-<name>.sh`
(→ `~/.local/bin`). `VideoIO.swift` is duplicated in each on purpose so the
folders stay independent — edit one, copy it to the other two.

Status: every source file typechecks against the macOS 27 SDK headers. None
has been built or run yet. See NOTES.md for what that does and doesn't cover.
