# Topaz presets

One TOML file per preset. These are the source of truth for every Topaz wrapper
(`topaz-encode`, `topaz-pick`, `topaz-workflow`, `topaz-simple-presets`,
`topaz-catalog`, and the mpv render menu in
`config/mpv/scripts/topaz-workflow-current.lua`).

`../topaz-presets-emit.py` renders these files into the legacy tab-separated rows
the tools consume, and `../topaz-preset-catalog.zsh` wraps it under the historical
function names. **Editing a `.toml` here changes the live config immediately** — no
build, no deploy (the whole `config/` tree is symlinked into place). A preset's
**slug is its filename** (`proteus-extreme.toml` → slug `proteus-extreme`).

Requires `python3 >= 3.11` on PATH (for stdlib `tomllib`). macOS system python
(3.9) is skipped automatically; set `TOPAZ_PRESETS_PYTHON` to force an interpreter.

## Editing tips

- Store ffmpeg filters as TOML **literal strings** (single quotes): `filter = '...'`.
  Literal strings take `:` and `\` verbatim — no escaping.
- The one exception is the grain sub-option, which needs its colon escaped for
  ffmpeg: `...parameters=grain_type=gaussian\\:grain_sigma=0.5...` (two backslashes,
  inside the single-quoted literal string). Copy an existing grain preset if unsure.
- `order` controls position within a type/menu group (ascending). Prose fields
  (`display`, `blurb`, `metadata`, `[insight]`) are ordinary double-quoted strings.
- After editing, sanity-check with:
  `python3 ../topaz-presets-emit.py <type> .` — it prints the rows or errors.

## Directories & schema

### `enhancement/` — mpv render-menu presets
```toml
order = 1
category = "detail"          # detail | repair | sharpen | focus-fix
display = "Proteus — Detail Max"
scales  = ["1", "2", "4k"]   # which output resolutions the model supports
blurb   = "Max invention + compression fix, minimal sharpen"
filter  = 'tvai_up=model=prob-4:@SCALE@:...'   # @SCALE@ is filled from the Output tab
metadata = "videoai=[Detail] ..."

[insight]                    # optional; powers the `d` details sheet
strategy = "..."
watch    = "..."
vs       = "proteus-max-regrain"
vs_note  = "..."
```
A file with `pseudo = true` (e.g. `__original__.toml`) contributes only its
`[insight]`, not a menu row.

### `interpolation/` — frame-rate stage
```toml
order = 1
display = "Apollo 60fps — best quality"
filter  = 'tvai_fi=model=apo-8:slowmo=1:...'
metadata = "videoai=[Interpolate] ..."
```

### `output/` — output format profiles
```toml
order = 1
display    = "HEVC constant bitrate 40mbps"
ext        = "mp4"
video_args = '-c:v hevc_videotoolbox ...'
```

### `transform/` — two-step workflow presets (topaz-workflow)
```toml
order = 1
display    = "[Cleanup] Proteus Compression Cleanup"
categories = "Cleanup"
filter     = 'tvai_up=...'
metadata   = "videoai=..."
```

### `encode/` and `simple/` — fzf picker presets (topaz-encode / topaz-simple-presets)
```toml
order = 1
display      = "HQ 1080p to clean sharp 4K HEVC"
# preset_name = "..."   # only if it differs from display (simple/ uses the slug)
filter       = 'tvai_up=...'
ext          = "mp4"        # omit for filter-only presets
video_args   = '-c:v ...'   # omit for filter-only presets
metadata     = "videoai=..."# omit for filter-only presets
```
`preset_flag` defaults to `--filter_complex`; omit unless a preset needs another.
