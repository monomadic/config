# Verification notes

Originally written without a compiler. On 2026-09-23 each tool was typechecked
with `swiftc -typecheck` against the Command Line Tools' macOS 27 SDK (with a
local stand-in for ArgumentParser's surface), and these guesses were corrected:

- `VTSuperResolutionScalerConfiguration` is **macOS 26**, not 15.4; the
  packages now target `.macOS(.v26)` (tools-version 6.2, Swift 5 language mode).
- Its `QualityPrioritization` has only `.normal`, so avupscale's `--quality` is gone.
- `downloadConfigurationModel` has no progress handler; progress is polled
  from `configurationModelPercentageAvailable`.
- `supportedScaleFactors` is `[Int]`; `isSupported` is a class property.
- `frameSupportedPixelFormats` is deprecated → `supportedPixelFormats: [OSType]`.
- Optical flow takes its own `VTOpticalFlowConfiguration.QualityPrioritization`.
- Vision: `generateScaledMaskForImage(forInstances:from:)`.
- `MLModel.compileModel(at:)` is async.
- `processor.process(parameters:)` resolves to the awaited completion-handler
  form, not macOS 26's AsyncSequence overload (checked explicitly).

Also changed: source color tags (primaries/transfer/matrix) now pass through to
the writer, the probe reader is cancelled, avremove skips frames whose crop
window collapses below 8 px, and `-Ounchecked` was dropped.

## Not yet verified (needs a real build + a run)
- The swift-argument-parser package resolve/build itself.
- Runtime behaviour: pixel-format negotiation, IOSurface requirements on the
  reader's buffers, interpolation timestamps, SR temporal feedback.
- avremove: whether `CIContext.render(_:toBitmap:)` rows are top-down (the
  bbox flip in `nonzeroBounds` assumes so — check with `--debug-mask`), and
  whether rendering the mask into a OneComponent8 buffer works.
- convert_lama.py: the simple-lama-inpainting import path, and whether
  coremltools converts LaMa's FFT layers on the installed version.
