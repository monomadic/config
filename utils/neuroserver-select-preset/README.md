# neuroserver-select-preset

TUI for the Topaz models that only the app's **neuroserver** process can run
(Starlight Precise 2.5 / 2.6 today — every enhancement preset in
`config/zsh/bin/lib/topaz-presets/` that declares `ns_model`). It is the
neuroserver counterpart of the mpv `z` menu and `topaz-pick`, which are
ffmpeg-only.

```
neuroserver-select-preset FILE [--time SECONDS]
```

Three lists on the left — preset, resolution (the same rows and fallback rules
as the mpv Output tab), output profile — and the preview on the right.
`Enter` renders the preview window at `t` through `topaz-preview-frame
--keep-window`: neuroserver refuses runs shorter than five frames, so a "still"
is really an 8-frame window, and this tool shows **every frame of it**, as a
kitty image (half-block cells in any other terminal), `←`/`→` to step,
`o`/`space` to flip to the matching source frame. The requested frame is the
ringed one in the strip. `,`/`.` and `<`/`>` move `t`; renders are cached per
preset, resolution and time, and neuroserver's own phase and percentage
(encode → DiT → decode) show in the status line while one runs; `Esc` cancels.

`e` confirms and runs `neuroserver-encode` on the whole clip with the chosen
preset, resolution and output profile (output lands beside the input as
`<stem> [Topaz - <preset>].<ext>`, the log in `~/Library/Logs/topaz-batch`).
`c` prints that command instead and quits.

Install: `setup/install/install-neuroserver-select-preset.sh` → `~/.local/bin`.
It calls the deployed `~/.zsh/bin/topaz-preview-frame` and
`~/.zsh/bin/neuroserver-encode`, so the `zsh` Dotter package must be deployed.
