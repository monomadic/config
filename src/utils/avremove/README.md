# avremove

Per-frame object removal: Vision instance segmentation + tracking for the
mask, LaMa (Core ML) for the fill. Deterministic feed-forward inpainting, so
frame-to-frame flicker is far lower than a generative fill — but it is still
non-temporal. For true temporal coherence use ProPainter.

## 1. Build the model (once)

```sh
uv venv && source .venv/bin/activate
uv pip install torch coremltools simple-lama-inpainting pillow
python convert_lama.py --size 512 --out LaMa512.mlpackage
# python convert_lama.py --size 1024 --out LaMa1024.mlpackage   # more detail, slower
```

## 2. Build + run

```sh
scripts/install/install-avremove.sh
# check the mask/tracking first
avremove in.mp4 mask-check.mov --point 640,360 --debug-mask
# then remove
avremove in.mp4 out.mov --point 640,360 --model LaMa512.mlpackage
# static region (burned-in logo etc.)
avremove in.mp4 out.mov --mask logo-mask.png --dilate 4
```

Options: `--dilate N` grows the mask (default 12 px, hides subject halo),
`--feather N` softens the paste-back edge, `--margin F` controls how much
context around the mask the model sees (0.6 = window is 2.2x the bbox).

## How it works
1. Frame 0: `VNGenerateForegroundInstanceMaskRequest`, pick the instance
   under `--point`, seed `VNTrackObjectRequest` with its bbox.
2. Each frame: track → re-segment → take the instance at the tracked centre.
3. Square window around the mask bbox → resample to the model tile → LaMa →
   scale back → `CIBlendWithMask` into the untouched frame.

Only the window is resampled; everything outside is byte-identical to input.

## Known limits
- Tracking loss (occlusion, fast motion) leaves frames untouched; check with
  `--debug-mask`. A `--mask-dir frames/%06d.png` input from Resolve's magic
  mask would be the next upgrade.
- `--point` is in the file's stored (unrotated) pixel grid — for portrait
  phone footage that is the landscape frame before its rotation tag.
- Foreground instance masks only segment salient subjects; for small
  background objects use a static `--mask`.

Sources
- LaMa: https://github.com/advimman/lama (Suvorov et al., WACV 2022)
- Vision instance masks: https://developer.apple.com/documentation/vision/vngenerateforegroundinstancemaskrequest
