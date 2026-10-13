#!/usr/bin/env python3
"""List the partial downloads in yt-dlp's temp dir, with the URL to resume each.

    cleanup-ytdlp-partials.py <temp dir> <info.json dir> [<download archive>]

One line per download, tab separated ("-" for unknown):

    kb  mtime  files  url  format_id  alias  source  state  base

`state` is `done` when the download archive already records the video — a
later run finished it and this partial is what the dead one left — else
`partial`.

A download is every file in one directory sharing a base name: `base.mp4.part`,
`base.mp4.ytdl`, `base.mp4.part-Frag12.part`, `base.f137.mp4.part`, and the
thumbnail or subtitle beside them. `base` is that absolute path without
extensions.

The .part holds no URL. It comes from the info.json yt-dlp leaves behind when
a download dies (--embed-info-json only deletes it after embedding), found by
the id in the file name; failing that, from the id itself on sites where the
URL can be rebuilt. `alias` is a guess at the config alias that produced the
name (porn | visuals | plain) — the caller checks it before downloading.
"""
import json
import os
import re
import sys

PARTIAL = re.compile(r"\.(?:part(?:-Frag\d+(?:\.part)?)?|ytdl)$")
EXT = re.compile(r"\.[A-Za-z0-9]{2,5}$")
FORMAT = re.compile(r"\.f[\w-]+$")
TOKEN = re.compile(r"[(\[]([^()\[\]]+)[)\]]")
CUT_TOKEN = re.compile(r"\(([^()]+)$")  # --trim-filenames cut the name mid-id
SUFFIX = ".info.json"

# Sites whose page URL is just the id: extractor (as it appears in the file
# name, lowercased) -> URL.
FROM_ID = {
    "faphouse": "https://faphouse.com/videos/{}",
}


def base_of(name):
    """`Title (x_1).f137.mp4.part-Frag3.part` -> `Title (x_1)`; None if not a partial."""
    stem, n = PARTIAL.subn("", name)
    if not n:
        return None
    return FORMAT.sub("", EXT.sub("", stem))


def find_json(tokens, cut, names):
    """The info.json for one of the ids in the file name, trying the last first."""
    for tok in reversed(tokens):
        if tok + SUFFIX in names:
            return tok + SUFFIX
        hits = [n for n in names if n.endswith("_" + tok + SUFFIX)]
        if len(hits) == 1:
            return hits[0]
    if cut:
        hits = [n for n in names if n.startswith(cut) and n.endswith(SUFFIX)]
        if len(hits) == 1:
            return hits[0]
    return None


def main():
    tmp, json_dir = sys.argv[1], sys.argv[2]
    try:
        names = set(os.listdir(json_dir))
    except OSError:
        names = set()

    archived, archived_ids = set(), set()
    if len(sys.argv) > 3:
        try:
            with open(sys.argv[3]) as fh:
                for line in fh:
                    archived.add(line.strip())
                    archived_ids.add(line.strip().partition(" ")[2])
        except OSError:
            pass

    for root, _, files in os.walk(tmp):
        bases = {b for b in map(base_of, files) if b is not None}
        for base in sorted(bases):
            group = [f for f in files if f.startswith(base + ".")]
            size = mtime = 0
            for f in group:
                try:
                    st = os.stat(os.path.join(root, f))
                except OSError:
                    continue
                size += st.st_size
                mtime = max(mtime, int(st.st_mtime))

            tokens = TOKEN.findall(base)
            cut = CUT_TOKEN.search(base)
            url = fmt = source = "-"
            state = "partial"
            found = find_json(tokens, cut.group(1) if cut else None, names)
            if found:
                try:
                    with open(os.path.join(json_dir, found)) as fh:
                        info = json.load(fh)
                    url = info.get("webpage_url") or "-"
                    fmt = info.get("format_id") or "-"
                    source = "info.json"
                    key = "{} {}".format(info.get("extractor_key", "").lower(), info.get("id"))
                    if key in archived:
                        state = "done"
                except (OSError, ValueError):
                    pass
            if tokens and base.endswith((")", "]")):
                extractor, _, vid = tokens[-1].partition("_")
                if (vid or extractor) in archived_ids:
                    state = "done"
                if url == "-" and vid and extractor.lower() in FROM_ID:
                    url, fmt, source = FROM_ID[extractor.lower()].format(vid), "-", "id"

            if root == tmp:
                alias = "visuals"
            elif base.endswith("]"):
                alias = "plain"
            else:
                alias = "porn"

            print(size // 1024, mtime, len(group), url, fmt, alias, source,
                  state, os.path.join(root, base), sep="\t")


if __name__ == "__main__":
    main()
