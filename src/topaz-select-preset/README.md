# topaz-select-preset

TUI for the Topaz models the app's **ffmpeg** runs (`tvai_up` / `tvai_fi`) —
every enhancement preset in `bin/lib/topaz-presets/` that does *not*
declare `ns_model`, plus the interpolation and output profiles. It is the
terminal counterpart of the mpv `z` menu and a visual `topaz-pick`, and the
ffmpeg sibling of `neuroserver-select-preset`, whose layout and keys it shares.

```
topaz-select-preset FILE [--time SECONDS] [--window FRAMES]
```

Four lists on the left — enhancement (grouped by what is wrong with the source,
as topaz-pick groups them), resolution (the mpv Output tab's rows and fallback
rules), interpolation, output format — and the preview on the right. The filter
is composed exactly as topaz-pick composes it (`@SCALE@` filled, interpolation
before the lanczos tail), and `d` shows it with the preset's `[insight]` notes.

`Enter` renders a **window** of `--window` source frames (default 8) around `t`
through `topaz-preview-frame --keep-window`, and shows every frame as a kitty
image (half-block cells in any other terminal): `←`/`→` step, `o`/`space` flip
to the source frame, `s` puts them side by side. With an interpolation selected
the window keeps it, so the frames it *invents* are there to inspect — drawn as
`·` in the strip, between the source-aligned `○`s. `z` zooms 2×/4×/8× into the
same region of both frames (nearest-neighbour, so rendered pixels stay crisp)
and `H J K L` pan; that is what makes a detail preset judgeable in a terminal.
Renders are cached per preset, resolution, interpolation and time, and moving
onto a cached one shows it at once — render a few, then flick through them with
`j`/`k` at the same frame and zoom. `,`/`.` and `<`/`>` move `t`; `Esc` cancels.

`e` runs `topaz-encode` on the whole clip with the chosen preset, **inside the
TUI**: phase, frame count, fps, speed and an ETA from ffmpeg's own progress,
and the **newest frame written to the output**, refreshed every few seconds.
That works because topaz-encode always writes a fragmented `NAME.frag.EXT`
while it runs, which trails the encoder by at most a GOP (about a second with
the HEVC profiles, one frame with ProRes). `p` pauses the whole process group
in place and continues it, `o` opens the output so far in mpv, `Esc` stops.

**Resuming.** A stopped or crashed encode leaves the `.frag` file behind. `e`
then says how much of the clip it holds and offers `r` to resume (the default:
topaz-encode encodes only the rest and joins it losslessly) or `x` to move it to
the Trash and start again. A complete output offers `x` to replace it. `c`
prints the plain `topaz-encode` command for the selection instead and quits.

The output is named as topaz-encode names it — `<stem> [Topaz - <preset>].<ext>`
beside the input — so an encode started here and one started from `topaz-pick`
find each other's partials. The TUI passes `--output` anyway, and follows the
encode through `--progress-file` (ffmpeg's raw `-progress` stream, which log
mode otherwise reduces to 5% steps).

Install: `scripts/install/install-topaz-select-preset.sh` → `~/.local/bin`.
It calls the deployed `~/.local/bin/topaz-preview-frame` and
`~/.local/bin/topaz-encode`, and reads presets straight from the TOML tree at
`~/.local/bin/lib/topaz-presets` (`TOPAZ_PRESETS_DIR` overrides it), so the
`zsh` Dotter package must be deployed.
