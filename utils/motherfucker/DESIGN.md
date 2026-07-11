# motherfucker — design

Brand and visual spec. Rule zero mirrors the code: draw almost nothing,
and never at summon time. The brand is two colors and one glyph.

## Identity

- **Name.** Always lowercase: `motherfucker`. No title case, no "MF", no
  censored variant — where the name can't go, the mark stands alone.
- **Idea.** A lightning bolt on warning yellow: instant, a little loud,
  slightly rude. The app in one glyph.
- **Voice.** Terse imperative, lowercase. Says what it does ("no jank"),
  never how hard it tries.

## The mark — knockout bolt

Glass-black bolt knocked out of a volt-yellow squircle.

- Canvas 1024×1024, transparent. Squircle 824×824 centered (Apple's Big
  Sur icon grid), corner radius 185, volt fill.
- Bolt: one six-point polygon, glass fill —
  `573.8,275.1 388.4,553.2 491.4,553.2 439.9,748.9 645.9,460.5 532.6,460.5`.
- Flat only. Never outlined, never gradiented, never shadowed, never
  rotated.

Variants, in order of preference:

1. **Tile** — the full squircle. App icon contexts (Finder, Activity
   Monitor, Force Quit).
2. **Bare bolt, volt** — the polygon alone on any dark surface. Menu
   bar, inline, favicon-scale (< 16 px).
3. **Bare bolt, glass** — the polygon alone on volt or other light
   surfaces. Never volt-on-white.

## Wordmark

`mother⚡ucker` — lowercase monospace (SF Mono / Menlo), regular weight,
with the bolt occupying the f's character cell. The wordmark is typed,
not drawn; the bolt is the only special-cased glyph. Volt on dark
surfaces, glass on light ones, never mixed weights or sizes.

## Color

| name      | value               | role |
|-----------|---------------------|------|
| volt      | `#F1FF0F`           | brand yellow; matched query characters in the panel |
| glass     | `#0D0D10`           | bolt fill; the "black" of black glass |
| tint      | `#000000` @ 0.80    | wash over the HUD blur (`[style] tint`) |
| selection | `#0963F6` @ 0.40    | selected row (`[style] selection`) |

Rules: volt goes on dark only. Selection blue is UI state, never brand —
it does not appear in the mark or wordmark. Any one surface uses at most
two of these.

## Panel ("black glass, tint selection")

The running product, spec'd by `config/motherfucker/config.toml` defaults:

- Non-activating NSPanel over the system dark-HUD vibrancy material;
  the window server composites the blur, we draw only text. 620 pt wide.
- Input line 24 pt, rows 15 pt, system font.
- Selected row: selection blue at 0.40 over the glass.
- Matched query characters: volt. Everything else: system label colors
  under vibrancy.

## Assets

| file | what |
|------|------|
| `assets/icon.svg` | canonical vector of the mark |
| `assets/icon.png` | 1024 px raster, embedded via `include_bytes!` and self-stamped onto the installed binary at startup (`main.rs`) |
| `assets/genicon.swift` | regenerates icon.png: `swift assets/genicon.swift assets/icon.png` |
| `assets/wordmark.svg` | wordmark, volt-on-dark variant |

After changing the mark: edit icon.svg and genicon.swift together (same
coordinates), regenerate icon.png, then `setup/install/motherfucker.sh`.
The install overwrites the binary's Finder-icon metadata, and the first
launch re-stamps it from the embedded copy.
