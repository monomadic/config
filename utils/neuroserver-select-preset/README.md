# neuroserver-select-preset

TUI for the Topaz models that only the app's **neuroserver** process can run
(Starlight Precise 2.5 / 2.6 today — every enhancement preset in
`config/zsh/bin/lib/topaz-presets/` that declares `ns_model`). It is the
neuroserver counterpart of the mpv `z` menu and `topaz-pick`, which are
ffmpeg-only.

```
neuroserver-select-preset FILE [--time SECONDS]
neuroserver-select-preset encode --input FILE --model M --store S …   # or: neuroserver-encode …
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

`e` confirms and encodes the whole clip with the chosen preset, resolution and
output profile (output lands beside the input as `<stem> [Topaz - <preset>].<ext>`,
the log in `~/Library/Logs/topaz-batch`). `c` prints the equivalent
`neuroserver-select-preset encode …` command instead and quits.

The encoder is part of this binary (`src/nsencode.rs`), not a separate script:
the TUI runs it in a thread, and the same code is the command-line encoder,
reached as the `encode` subcommand or through the `neuroserver-encode` symlink
the installer makes (the name it is invoked by picks the mode). `encode --help`
lists its options; `--dry-run` prints the neuroserver and mux commands.

The encode runs **inside** the TUI: a progress bar, the current phase, frame
count, elapsed time and a rate-based ETA, all read from neuroserver's JSON
progress lines as they arrive (every line is also kept in the log). `Esc` asks
before cancelling (neuroserver runs in its own process group and the whole group
is signalled, so nothing is orphaned); `↵` returns to the picker when it
finishes.

It also shows the **newest frame written to the output**, as a kitty image,
with one honest limit. Starlight works in chunks of about 100 source frames,
each going through encode → upscale → decode → post-processing, and neuroserver
hands a chunk's frames to its ffmpeg only at that last step. So the output grows
in bursts — about 80 minutes apart at 4K on an M4 Pro — and the live frame
advances one chunk at a time. Until the first chunk lands the pane shows the
preview still for the selection, labelled as not the encode. The
live frame depends on the output being written as *fragmented* MP4/MOV, which
the TUI adds to the codec arguments (the flags the Topaz app uses itself): a
plain MP4 has no index until it is closed and cannot be opened mid-write. The
mux pass rewrites it as an ordinary file at the end. The encoder adds those
flags unless the codec arguments already set `-movflags`.

**Resuming.** The same fragmented output is what makes an interrupted encode
recoverable: what neuroserver had written survives a crash or a hard kill.
Pressing `e` when such a partial exists for the chosen output says how many
frames it kept and offers `r` to resume (the default) or `x` to discard and
restart. Resuming keeps the surviving frames as a part file, renders only the
frames after them, and joins the parts losslessly before the audio mux. Because
Starlight writes a chunk only when the chunk is finished, an interruption costs
at most the chunk in flight. neuroserver's own `--resume` flag is not used: for
Starlight Precise 2.6 it is accepted and then ignored, re-rendering from frame
zero over the partial (tested).

Two limits worth knowing. Starlight Precise fails on sources smaller than its
640-pixel encoder tile (a 320-pixel clip crashes it; the encoder says so rather
than "no output"). And a resumed encode starts a fresh chunk at the resume
point, so the model's temporal context differs there from an uninterrupted run
— the same kind of boundary it already has between its own chunks.

neuroserver also needs a valid Topaz licence: without one it exits at start-up
with "No valid license or auth found", which the encoder reports as the reason.

Install: `setup/install/install-neuroserver-select-preset.sh` → `~/.local/bin`
(plus the `neuroserver-encode` symlink). Previews call the deployed
`~/.zsh/bin/topaz-preview-frame`, and the preset catalog is read through
`~/.zsh/bin/lib/topaz-preset-catalog.zsh`, so the `zsh` Dotter package must be
deployed.
