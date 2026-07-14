# Zsh Config — Outstanding Tasks

P0 (leak scrub) and the P1–P3 assessment (startup 109 ms → 27 ms, keybinding/history
conflicts, dead aliases) are done and pushed. The footgun aliases (`ga`/`gca`/`~=grep`),
the `dd`→`deploy` rename, and the stray `songs/`/`lua/` dirs are also cleared. What's
left is the P4 structural work, none of which is a clean unprompted win — notes below.

## P4 — structure

### Re-home `alias.zsh` (770 lines) — worth doing, needs a focused pass

Its yt-dlp / ffmpeg / rsync / kitty / mpv sections duplicate the domain files that
already exist. Verified conflict-free: no alias/function name in `alias.zsh` collides
with one in `ffmpeg.zsh`, `media.zsh`, `yt-dlp.zsh`, `kitty.zsh`, or `rsync.zsh`, so a
move overrides nothing, and aliases/functions load lazily so source order is safe.

The catch is categorization, not safety: the yt-dlp and rsync sections move cleanly
into their files, but the mpv/media/download/ffmpeg aliases blur together and
`media.zsh` is already 200+ lines — deciding where the fuzzy ones live is a taste call.
Best done as its own commit (easy to review/revert), unambiguous sections first.

### Slim `zshenv.zsh` — not worth it

Rationale was to stop interactive-only exports (media glob arrays, `JUMP_DIRS`, FZF
options) loading for every non-interactive zsh. But **zero `bin/` scripts read any of
them**, and startup is already 27 ms, so moving them is cosmetic with a small risk of
breaking a subshell that relies on the export. Recommend leaving as-is.
(One real bug to fix if touched: `ZSH_SCRIPT_PATHS` is declared as an exported array —
zsh can't export arrays, so it silently exports a scalar. `configure-bin` reads it.)

### Rename `autoload/` — skip

Nothing in it is fpath-`autoload`ed; it's all sourced, so the name is a misnomer. But
renaming is cosmetic, forces a `dotter` mapping change plus a redeploy that re-points a
live symlink (`~/.zsh/autoload`), and risks leaving a dangling old symlink. Risk > value.
