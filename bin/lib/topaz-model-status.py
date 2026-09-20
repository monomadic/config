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


def mac_model_set(descriptor):
    """The part list ffmpeg loads for a composite model on this Mac.

    Starlight Mini is not one network but an encoder, unet and decoder, chosen
    per machine from `modelSet` by `modelSelector.mac` on system RAM in GB (the
    app logs "Selected model set: slmm" on a 48 GB machine). Returns None for an
    ordinary single-network model.
    """
    sets = descriptor.get("modelSet")
    if not isinstance(sets, dict) or not sets:
        return None
    selector = (descriptor.get("modelSelector") or {}).get("mac") or {}
    try:
        ram_gb = os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES") / 2**30
    except (ValueError, OSError):
        ram_gb = 0
    for name, bounds in selector.items():
        lo, hi = bounds.get("min"), bounds.get("max")
        if (lo is None or ram_gb >= lo) and (hi is None or ram_gb < hi) and name in sets:
            return sets[name]
    # No selector matched (or none for mac): take the smallest set, which is
    # what every machine can at least attempt.
    return sets[sorted(sets, key=lambda k: len(sets[k]))[0]]


def runs_in_ffmpeg(code, descriptors):
    """Can `code` run through `ffmpeg -filter_complex tvai_up=...` at all?

    The classic models (Proteus, Artemis, Iris, Dione, Nyx, Gaia, Theia, Rhea,
    Chronos, Apollo) declare a `backends.coreml` spec and are executed by the
    tvai_up filter — that is the path every script here uses. A composite model
    (Starlight Mini) declares no backend of its own but a `modelSet` of parts
    that do; tvai_up loads the parts, so it runs in ffmpeg too — the app itself
    drives slm-1 through tvai_up, never through neuroserver.

    The generative models served by the app's separate `neuroserver` process
    (Starlight Precise, Astra, Hyperion 2) declare `isNeuroserverModel` and are
    out of reach of ffmpeg.

    tvai_up given a backend-less model does not error: it initialises the graph
    and then blocks forever, which is exactly what a Starlight preview looked
    like before its parts were downloaded.
    """
    d = descriptors.get(code) or {}
    backends = d.get("backends")
    if isinstance(backends, dict) and backends.get("coreml"):
        return True
    parts = mac_model_set(d)
    if not parts:
        return False
    return all(runs_in_ffmpeg(part, descriptors) for part in parts)


def has_weights(code, weights):
    """Are this model's weights on disk?

    Exact `<stem>-v<version>` match — the naming is consistent across every
    single-network model. A Starlight Mini part is the exception: its descriptor
    is `slmem-1` (encoder, medium set) but its weight file is `slme-v1-...`, the
    set letter dropped from the stem — so try that too.
    """
    if weights.get(code):
        return True
    stem, _, ver = code.rpartition("-")
    return bool(stem and stem[-1] in "sml" and weights.get(f"{stem[:-1]}-{ver}"))


def missing_parts(code, descriptors, weights, seen=None):
    """Model codes in `code`'s dependency closure that have no weights on disk."""
    seen = seen if seen is not None else set()
    if code in seen:
        return []
    seen.add(code)

    d = descriptors.get(code) or {}
    parts = mac_model_set(d)
    if parts:
        # A composite model has no weights of its own: only the parts of the set
        # this machine would load matter (the `dependencies` list names every set).
        missing = []
        for part in parts:
            missing.extend(missing_parts(part, descriptors, weights, seen))
        return missing

    missing = []
    if not has_weights(code, weights):
        missing.append(code)
    for dep in d.get("dependencies", []) or []:
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
            # Served by the neuroserver process rather than the tvai_up filter, so
            # unreachable from ffmpeg — but still perfectly usable in the app.
            #
            # Do NOT read platform support out of `validate_install`: several of
            # these list only a `windows` key yet run fine on macOS (Hyperion 2
            # and Starlight Precise both do). It is a download-integrity block —
            # zip hash and file count — not an availability matrix.
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
