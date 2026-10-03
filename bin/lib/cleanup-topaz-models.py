#!/usr/bin/env python3
"""Inventory of Topaz model weights for `cleanup`: one TSV row per model.

usage: cleanup-topaz-models.py <models-dir> <presets-dir> [<archive-dir>]

Columns: kind, key, name, weight_bytes, cache_bytes, nfiles, uses

  model    key = model code (prob-4). Weights are every <stem>-v<ver>-*.tz3;
           cache_bytes is its compiled coreMLCache/<stem>-v<ver>-* dirs.
  store    key = neuroserver store dir under models/models (slp26, ...).
  restore-model / restore-store
           the same, found in <archive-dir> but absent locally.

`uses` is how many enabled presets under <presets-dir> reach the model: a
`model=<code>` in a tvai filter, or an `ns_store` for a store. A composite
model (Starlight Mini) is followed into the parts this Mac would load, so its
encoder/unet/decoder weights count as used, not as orphans.
"""
import importlib.util
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location(
    "topaz_model_status", os.path.join(HERE, "topaz-model-status.py"))
status = importlib.util.module_from_spec(spec)
spec.loader.exec_module(status)

WEIGHT = re.compile(r"^(.+?)-v(\d+)-.*")


def dir_bytes(path):
    total = 0
    for root, _, files in os.walk(path):
        for f in files:
            try:
                total += os.lstat(os.path.join(root, f)).st_size
            except OSError:
                pass
    return total


def weights_in(directory):
    """code -> [bytes, nfiles] for the .tz3 files directly in `directory`."""
    out = {}
    try:
        names = os.listdir(directory)
    except OSError:
        return out
    for name in names:
        m = WEIGHT.match(name)
        if not (m and name.endswith(".tz3")):
            continue
        code = f"{m.group(1)}-{m.group(2)}"
        entry = out.setdefault(code, [0, 0])
        entry[0] += os.path.getsize(os.path.join(directory, name))
        entry[1] += 1
    return out


def closure(code, descriptors, seen):
    if code in seen:
        return
    seen.add(code)
    d = descriptors.get(code) or {}
    parts = status.mac_model_set(d)
    if parts:
        for part in parts:
            closure(part, descriptors, seen)
        return
    for dep in d.get("dependencies", []) or []:
        closure(dep, descriptors, seen)


def preset_uses(presets_dir, descriptors):
    """({weight code: {preset}}, {store: {preset}}) from the enabled presets."""
    by_code, by_store = {}, {}
    for root, _, files in os.walk(presets_dir):
        for name in files:
            if not name.endswith(".toml"):
                continue
            path = os.path.join(root, name)
            text = open(path, encoding="utf-8", errors="replace").read()
            if text.lstrip().startswith("# Disabled"):
                continue
            direct = set(re.findall(r"model=([a-z0-9]+-\d+)", text))
            reached = set()
            for code in direct:
                closure(code, descriptors, reached)
            for code in reached:
                by_code.setdefault(code, set()).add(path)
                stem, _, ver = code.rpartition("-")
                if stem and stem[-1] in "sml":     # slmem-1 -> weights slme-1
                    by_code.setdefault(f"{stem[:-1]}-{ver}", set()).add(path)
            for store in re.findall(r'^ns_store\s*=\s*"([^"]+)"', text, re.M):
                by_store.setdefault(store, set()).add(path)
    return by_code, by_store


def display_name(code, descriptors):
    stem, _, ver = code.rpartition("-")
    for key in (code, *(f"{stem}{s}-{ver}" for s in "sml")):
        name = (descriptors.get(key) or {}).get("displayName")
        if name:
            return f"{name} {ver}" if key == code else name
    return code


def emit(*cols):
    print("\t".join(str(c) for c in cols))


def main():
    if len(sys.argv) < 3:
        sys.stderr.write(__doc__)
        return 2
    models, presets = sys.argv[1], sys.argv[2]
    archive = sys.argv[3] if len(sys.argv) > 3 else None

    descriptors, _ = status.load(models)
    by_code, by_store = preset_uses(presets, descriptors)
    local = weights_in(models)

    caches = {}
    cache_root = os.path.join(models, "coreMLCache")
    if os.path.isdir(cache_root):
        for name in os.listdir(cache_root):
            m = WEIGHT.match(name)
            if m:
                code = f"{m.group(1)}-{m.group(2)}"
                caches[code] = caches.get(code, 0) + dir_bytes(os.path.join(cache_root, name))

    for code, (size, n) in sorted(local.items()):
        emit("model", code, display_name(code, descriptors), size,
             caches.get(code, 0), n, len(by_code.get(code, ())))

    stores_dir = os.path.join(models, "models")
    local_stores = set()
    if os.path.isdir(stores_dir):
        for name in sorted(os.listdir(stores_dir)):
            path = os.path.join(stores_dir, name)
            if os.path.isdir(path) and not name.startswith("__"):
                local_stores.add(name)
                emit("store", name, name, dir_bytes(path), 0, len(os.listdir(path)),
                     len(by_store.get(name, ())))

    if archive and os.path.isdir(archive):
        for code, (size, n) in sorted(weights_in(archive).items()):
            if code not in local:
                emit("restore-model", code, display_name(code, descriptors), size, 0, n, 0)
        astores = os.path.join(archive, "models")
        if os.path.isdir(astores):
            for name in sorted(os.listdir(astores)):
                path = os.path.join(astores, name)
                if os.path.isdir(path) and not name.startswith("__") and name not in local_stores:
                    emit("restore-store", name, name, dir_bytes(path), 0, len(os.listdir(path)), 0)
    return 0


if __name__ == "__main__":
    sys.exit(main())
