# Quarantine

Commands retired from PATH, kept so the code and its history stay recoverable.

**Nothing here is deployed, and nothing in live config may reference it.**
Dotter has no package for this directory, so these files are not on PATH on any
machine. If something here turns out to still be needed, move it back with
`git mv _quarantine/<dir>/<cmd> bin/` and deploy — don't copy it out.

- `bin/` — dropped from the old top-level `bin/` during the 2026-09 restructure.
  These were unreferenced by any live config and mostly last touched in 2024:
  the midjourney scrapers, the firefox cookie extractors, the one-off image and
  disk-speed scripts, superseded topaz patchers.
- `zsh-bin/` — the former `config/zsh/_bin_quarantine/`: superseded VirtualDJ
  stem generations (`mkstem`, `mk-vdjstem-*`, `vdjstems-*-v1`, the sidecar and
  demucs variants) kept while `vdjstems` settled.

Don't add to this directory as a habit — it is a graveyard, not a staging area.
Retiring one command is a `git mv` here; retiring a whole category is a delete.
