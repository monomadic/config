#!/usr/bin/env python3
"""
Convert big-lama (LaMa inpainting, Suvorov et al. 2021) to a Core ML package
with image-typed inputs so the Swift CLI can feed CVPixelBuffers directly.

    uv venv && source .venv/bin/activate
    uv pip install torch coremltools simple-lama-inpainting
    python convert_lama.py --size 512 --out LaMa512.mlpackage

Weights: simple-lama-inpainting's download helper fetches its TorchScript
big-lama.pt release (override with the LAMA_MODEL_URL env var). LaMa is a
feed-forward FFC network — deterministic, no diffusion — which is exactly why
it holds up per-frame far better than a generative fill like Photos' Clean Up.

Inputs must be multiples of 8. 512 fits comfortably on the ANE; 1024 works on
M-series but drops to GPU for some layers.

Model card: https://github.com/advimman/lama
"""
import argparse
import numpy as np
import torch
import coremltools as ct


class LamaWrapper(torch.nn.Module):
    """image: (1,3,H,W) in [0,1]; mask: (1,1,H,W) in [0,1] (1 = remove).
    Returns (1,3,H,W) in [0,255] so coremltools can expose it as an RGB image."""

    def __init__(self, model):
        super().__init__()
        self.model = model

    def forward(self, image, mask):
        mask = (mask > 0.5).float()
        out = self.model(image, mask)          # [0,1]
        # Composite: keep original pixels outside the mask, exactly like the
        # reference inference. Lets the Swift side skip a blend pass.
        out = out * mask + image * (1.0 - mask)
        return (out.clamp(0.0, 1.0) * 255.0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--size", type=int, default=512, help="square input size (multiple of 8)")
    ap.add_argument("--out", default="LaMa512.mlpackage")
    ap.add_argument("--fp32", action="store_true", help="keep fp32 weights (2x size, marginally better)")
    args = ap.parse_args()
    assert args.size % 8 == 0, "size must be a multiple of 8"

    # The helper has lived at both paths across releases.
    try:
        from simple_lama_inpainting.utils import download_model
    except ImportError:
        from simple_lama_inpainting.utils.util import download_model
    from simple_lama_inpainting.models.model import LAMA_MODEL_URL

    pt_path = download_model(LAMA_MODEL_URL)
    model = torch.jit.load(pt_path, map_location="cpu").eval()
    wrapped = LamaWrapper(model).eval()

    s = args.size
    ex_img = torch.rand(1, 3, s, s)
    ex_mask = (torch.rand(1, 1, s, s) > 0.8).float()
    with torch.no_grad():
        traced = torch.jit.trace(wrapped, (ex_img, ex_mask))

    mlmodel = ct.convert(
        traced,
        convert_to="mlprogram",
        inputs=[
            ct.ImageType(name="image", shape=(1, 3, s, s), color_layout=ct.colorlayout.RGB, scale=1 / 255.0),
            ct.ImageType(name="mask", shape=(1, 1, s, s), color_layout=ct.colorlayout.GRAYSCALE, scale=1 / 255.0),
        ],
        outputs=[ct.ImageType(name="output", color_layout=ct.colorlayout.RGB)],
        compute_precision=ct.precision.FLOAT32 if args.fp32 else ct.precision.FLOAT16,
        compute_units=ct.ComputeUnit.ALL,
        minimum_deployment_target=ct.target.macOS15,
    )
    mlmodel.short_description = f"big-lama inpainting, {s}x{s}, image in [0,255] RGB, mask 0/255 gray"
    mlmodel.save(args.out)
    print(f"saved {args.out}")

    # Smoke test (macOS only — coremltools can't predict on Linux).
    try:
        from PIL import Image
        img = Image.fromarray((np.random.rand(s, s, 3) * 255).astype(np.uint8))
        msk = Image.fromarray((np.random.rand(s, s) > 0.9).astype(np.uint8) * 255).convert("L")
        out = mlmodel.predict({"image": img, "mask": msk})["output"]
        print("predict ok:", out.size, out.mode)
    except Exception as e:  # pragma: no cover
        print("predict skipped:", e)


if __name__ == "__main__":
    main()
