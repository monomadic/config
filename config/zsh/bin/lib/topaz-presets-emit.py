#!/usr/bin/env python3
"""Emit legacy tab-separated Topaz preset rows from the per-preset TOML tree.

The Topaz shell/lua tooling consumes presets as tab-separated rows (see
topaz-preset-catalog.zsh). Those rows now live as one TOML file per preset under
topaz-presets/<type>/, so they can be hand-edited with named fields, comments and
no shell-quoting. This script reads a type's TOML files and prints the exact rows
the old hardcoded catalog functions produced, so every consumer keeps working
unchanged.

Usage: topaz-presets-emit.py <type> <presets-dir>
  type: enhancement | insights | interpolation | output | transform | catalog
"""
import sys
import pathlib

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    sys.stderr.write(
        "topaz-presets-emit: needs python3 >= 3.11 (stdlib tomllib) or a tomli backport\n"
    )
    sys.exit(3)


def load_dir(directory):
    """Load every *.toml in `directory`, sorted by `order` then filename.

    The filename stem is exposed as `slug` (the preset's stable id)."""
    items = []
    if not directory.is_dir():
        return items
    for path in sorted(directory.glob("*.toml")):
        with open(path, "rb") as fh:
            data = tomllib.load(fh)
        data["slug"] = path.stem
        items.append(data)
    items.sort(key=lambda d: (d.get("order", 1_000_000), d["slug"]))
    return items


def s(value):
    """Render a scalar field as a string; None/missing -> empty."""
    return "" if value is None else str(value)


def emit(rows):
    out = sys.stdout
    for row in rows:
        out.write("\t".join(s(field) for field in row))
        out.write("\n")


def main():
    if len(sys.argv) != 3:
        sys.stderr.write("usage: topaz-presets-emit.py <type> <presets-dir>\n")
        sys.exit(2)

    kind = sys.argv[1]
    base = pathlib.Path(sys.argv[2])

    if kind == "enhancement":
        rows = []
        for d in load_dir(base / "enhancement"):
            if d.get("pseudo"):
                continue  # e.g. __original__ carries only an [insight], no row
            scales = ",".join(d.get("scales", []))
            rows.append([
                d.get("category", ""), d["display"], d["slug"],
                scales, d["filter"], d.get("blurb", ""), d.get("metadata", ""),
            ])
        emit(rows)

    elif kind == "insights":
        # Details-sheet prose, folded into each enhancement TOML under [insight].
        # Row order is irrelevant (the lua keys these by slug).
        rows = []
        for d in load_dir(base / "enhancement"):
            ins = d.get("insight")
            if not ins:
                continue
            slug = d["slug"]
            if ins.get("strategy"):
                rows.append([slug, "strategy", ins["strategy"]])
            for key, text in (ins.get("notes") or {}).items():
                rows.append([slug, "note", key, text])
            if ins.get("watch"):
                rows.append([slug, "watch", ins["watch"]])
            if ins.get("vs"):
                rows.append([slug, "vs", ins["vs"], ins.get("vs_note", "")])
        emit(rows)

    elif kind == "interpolation":
        rows = [
            [d["display"], d["slug"], d["filter"], d.get("metadata", "")]
            for d in load_dir(base / "interpolation")
        ]
        emit(rows)

    elif kind == "output":
        rows = [
            [d["display"], d["slug"], d.get("ext", ""), d["video_args"]]
            for d in load_dir(base / "output")
        ]
        emit(rows)

    elif kind == "transform":
        rows = [
            [d["display"], d.get("categories", ""), d["slug"], d["filter"], d.get("metadata", "")]
            for d in load_dir(base / "transform")
        ]
        emit(rows)

    elif kind == "catalog":
        # The combined encode/simple picker table: up to 8 fields, picker first.
        # Rows without an output profile historically omit the trailing
        # ext/video_args/metadata columns entirely, so trim trailing empties to
        # stay byte-identical (the picker's `read` fills missing fields anyway).
        rows = []
        for picker in ("encode", "simple"):
            for d in load_dir(base / picker):
                row = [
                    picker, d["display"], d.get("preset_name", d["display"]),
                    d.get("preset_flag", "--filter_complex"), d["filter"],
                    s(d.get("ext", "")), s(d.get("video_args", "")), s(d.get("metadata", "")),
                ]
                while len(row) > 5 and row[-1] == "":
                    row.pop()
                rows.append(row)
        emit(rows)

    else:
        sys.stderr.write(f"topaz-presets-emit: unknown preset type: {kind}\n")
        sys.exit(2)


if __name__ == "__main__":
    main()
