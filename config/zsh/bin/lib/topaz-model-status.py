#!/usr/bin/env python3
"""Report which Topaz models can actually run, and how expensive they are.

Topaz ships a JSON descriptor for every model it knows about, but the weights
(.tz3) are fetched on demand. So a descriptor existing says nothing about whether
a render will work: a model whose weights are absent produces no error, no
download message and no progress — the filter graph initialises and then sits
there. That failure is indistinguishable from a slow model, which is why this
exists.

Weight files are named `<stem>-v<version>-<...>.tz3` and belong to the model code
`<stem>-<version>` (prob-v4-... -> prob-4). A model runs only if its own weights
AND every model in its `dependencies` chain are present.

Usage:
  topaz-model-status.py <models-dir> --list [filter]
  topaz-model-status.py <models-dir> --check <code> [code...]

--check exits 1 if any named model is unavailable, printing what is missing.
"""
import json
import os
import re
import sys


def load(models_dir):
    """Return (descriptors, weight_counts) keyed by model code / stem-version."""
    descriptors = {}
    for name in os.listdir(models_dir):
        if not name.endswith(".json"):
            continue
        try:
            data = json.load(open(os.path.join(models_dir, name)))
        except Exception:
            continue
        if isinstance(data, dict):  # a few files hold bare lists
            descriptors[name[:-5]] = data

    weights = {}
    for name in os.listdir(models_dir):
        m = re.match(r"^(.+?)-v(\d+)-.*\.tz3$", name)
        if m:
            code = f"{m.group(1)}-{m.group(2)}"
            weights[code] = weights.get(code, 0) + 1
    return descriptors, weights


def runs_in_ffmpeg(code, descriptors):
    """Can `code` run through `ffmpeg -filter_complex tvai_up=...` at all?

    Topaz has two inference paths. The classic models (Proteus, Artemis, Iris,
    Gaia, Nyx, Theia, Rhea, Chronos, Apollo) declare a `backends.coreml` spec and
    are executed by the tvai_up filter — that is the path every script here uses.
    The newer generative models (Starlight Mini, Starlight Precise, Astra) declare
    no backend at all and are served instead by the separate `neuroserver`
    process the app spawns, out of reach of ffmpeg.

    tvai_up given a backend-less model does not error: it initialises the graph
    and then blocks forever, which is exactly what a Starlight preview looked like.
    """
    backends = (descriptors.get(code) or {}).get("backends")
    return isinstance(backends, dict) and bool(backends.get("coreml"))


def has_weights(code, weights):
    """Are this model's weights on disk?

    Exact `<stem>-v<version>` match — the naming is consistent across every model
    that runs in ffmpeg. (The multi-part Starlight family, whose parts are named
    inconsistently, is excluded earlier by runs_in_ffmpeg.)
    """
    return bool(weights.get(code))


def missing_parts(code, descriptors, weights, seen=None):
    """Model codes in `code`'s dependency closure that have no weights on disk."""
    seen = seen if seen is not None else set()
    if code in seen:
        return []
    seen.add(code)

    missing = []
    if not has_weights(code, weights):
        missing.append(code)
    for dep in (descriptors.get(code) or {}).get("dependencies", []) or []:
        missing.extend(missing_parts(dep, descriptors, weights, seen))
    return missing


def main():
    if len(sys.argv) < 3:
        sys.stderr.write(__doc__)
        return 2

    models_dir, mode = sys.argv[1], sys.argv[2]
    descriptors, weights = load(models_dir)

    if mode == "--check":
        bad = []
        for code in sys.argv[3:]:
            if code not in descriptors:
                bad.append(f"{code}: no such model")
                continue
            name = descriptors[code].get("displayName", code)
            if not runs_in_ffmpeg(code, descriptors):
                bad.append(f"{code} ({name}): has no coreml backend — it runs only "
                           "inside the Topaz app, via neuroserver, and cannot be "
                           "driven by ffmpeg at all")
                continue
            gaps = missing_parts(code, descriptors, weights)
            if gaps:
                bad.append(f"{code} ({name}): weights not downloaded for "
                           + ", ".join(sorted(gaps)))
        for line in bad:
            print(line)
        return 1 if bad else 0

    if mode != "--list":
        sys.stderr.write(f"unknown mode: {mode}\n")
        return 2

    pattern = sys.argv[3].lower() if len(sys.argv) > 3 else ""
    rows = []
    for code in sorted(descriptors):
        d = descriptors[code]
        display = d.get("displayName", "")
        if pattern and pattern not in code.lower() and pattern not in display.lower():
            continue
        scales = sorted({s for name in os.listdir(models_dir)
                         for s in re.findall(r"-(\d+)x-", name)
                         if name.startswith(code.rsplit("-", 1)[0] + "-v")}, key=int)
        if runs_in_ffmpeg(code, descriptors):
            gaps = missing_parts(code, descriptors, weights)
            status = "ready" if not gaps else "NOT DOWNLOADED"
        elif d.get("isNeuroserverModel"):
            # Served by the neuroserver process, not the tvai_up filter. Some of
            # these ship Windows-only nets, which no amount of downloading fixes
            # on a Mac — validate_install lists the platforms that have a build.
            platforms = list((d.get("validate_install") or {}).keys())
            if platforms and "macos" not in platforms:
                status = "windows only"
            else:
                status = "neuroserver (not ffmpeg)"
        else:
            status = "no backend"
        rows.append((
            code, display, str(d.get("frames", 1)),
            ",".join(s + "x" for s in scales) or "-", status,
        ))

    if not rows:
        return 0
    w = [max(len(r[i]) for r in rows) for i in range(4)]
    print(f"{'CODE':<{w[0]}}  {'NAME':<{w[1]}}  {'FRAMES':>6}  {'SCALES':<{w[3]}}  STATUS")
    for r in rows:
        print(f"{r[0]:<{w[0]}}  {r[1]:<{w[1]}}  {r[2]:>6}  {r[3]:<{w[3]}}  {r[4]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
