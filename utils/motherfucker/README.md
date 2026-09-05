# motherfucker

Cache-free, single-binary, minimalist Spotlight replacement. App launcher and
switcher only. The core principle is **no jank**: whatever state the system is
in, this thing stays rock solid.

## Usage

Run the binary; it sits resident (a few MB, zero idle CPU) and waits for
**⌥Space**. Type to filter, `↑`/`↓` to select, `↩` to switch/open, `⌘↩` to
force-open (reopen event), `⌘R` to reveal in Finder, `⎋` or clicking away
dismisses. Summoning with an empty query lists running apps — it's an app
switcher by default. CPU/RAM gauges on running rows refresh every second
while the panel is up. Hold **⌘** to show a row-jump hint on each visible
row — a running app gets the first letter of its name (`⌘F` for Finder;
several running apps sharing a letter all show it, and repeated presses
cycle through them one at a time), everything else gets the next free
`⌘1`–`⌘9` in order. Pressing the hint opens that row directly (same as
selecting it and pressing `↩`); the hints disappear the moment ⌘ is
released. A hint never shadows a configured `[keys]` chord — `⌘R`/`⌘A`
stay Reveal/Select All even if a visible row starts with R or A.

Install: `setup/install/motherfucker.sh` — builds release, installs to
`~/.bin`, and loads a LaunchAgent (`com.nom.motherfucker`) so it starts at
login and relaunches if it ever dies. Re-run the script after code changes;
it reloads the agent. Uninstall: `launchctl bootout gui/$(id -u)
~/Library/LaunchAgents/com.nom.motherfucker.plist` and delete the plist.

## Configuration

`~/.config/motherfucker/config.toml` (in this repo: `config/motherfucker/`,
deployed by the Dotter `motherfucker` package). Read **once at startup** —
one file read for the process lifetime, so the disk never touches the
summon path. Apply changes with `launchctl kickstart -k
gui/$UID/com.nom.motherfucker`. Everything is optional; a missing or broken
file means built-in defaults (bad lines are reported on stderr and skipped).

- `[hotkeys]` — global triggers, `"chord" = "mode"`. Multiple triggers are
  supported and each carries a mode name (only `launcher` exists so far;
  the plumbing for more is in place). To take over **⌘Space**, disable
  Spotlight's shortcut first (see `spotlight-manager` in `config/zsh`),
  then add `"cmd+space" = "launcher"`.
- `[keys]` — in-panel bindings, `"chord" = "action"`. Actions: `open`,
  `launch-new`, `reveal`, `clear`, `dismiss`, `select-all`, `move-up`,
  `move-down`, `show-commands` (default `tab`; see `[commands.<App Name>]`
  below); `"none"` unbinds a default. The `⌘`-held row-jump hints above
  aren't configured here — they're computed live from whatever's on
  screen — but any `cmd+<letter>` bound here always wins over a hint that
  would otherwise land on the same letter.
- `[style]` — `width`, `panel_background`/`panel_foreground`/`panel_opacity`/
  `panel_padding`/`panel_corner_radius`, `border`/`border_width` (panel
  stroke, default 0), `item_foreground`/`item_font_size`/
  `item_foreground_highlight`, `icon_foreground` (glyph column + search
  icon), `item_info_foreground`/`item_info_background` (the inline tag
  pill; a background makes it filled instead of outlined),
  `selected_item_background`/`selected_item_foreground`/
  `selected_item_opacity`/`selected_item_corner_radius`/
  `selected_item_border`/`selected_item_border_width` (inset stroke,
  default 0)/`selected_item_foreground_highlight`, `input_font_size`,
  `cpu_alert`/`cpu_alert_background` (the ⚠ CPU badge), `running_dot`.
  Colors are `"#rrggbb"`; the highlight keys color the query-matched
  characters.
- **Themes** — `~/.config/motherfucker/themes/<name>.toml`, each holding a
  `[style]` section that overlays the base style (this repo ships a set in
  `config/motherfucker/themes/`). `theme = "name"` under `[style]` applies
  one at startup. Interactively: search for "theme" and open
  `Setting: Change Theme (…)` — the panel lists every theme with the
  active one selected, moving the selection restyles the live panel,
  `↩` keeps the theme for the session (the config file is never written),
  `⎋` reverts. Theme files are read at startup and on refresh-config
  only — never on the summon path.
- `[icons]` — `search`, `running_many`/`running_one`/`running_none`/
  `installed` (literal glyph strings; SF Symbols pasted as text work), and
  `utilities`/`system`/`applications`/`shortcut` (SF Symbol names for the
  tag pills).
- `[icons.apps]` — `"App Name" = "glyph"` per-app overrides for the
  row-state glyph (case-insensitive name match). Wrap the key in `*` for a
  substring match instead of exact — `"*downloads*" = "glyph"` matches any
  entry with "downloads" in its title; `"downloads*"`/`"*downloads"` match
  just the start/end. Exact keys always win over a pattern.
- `[shortcuts]` — `"Name" = "shell command"` custom entries, matched like
  apps and run via `sh -c` on activation.
- **App commands** — every app row (installed or running) is `⇥`-able: it
  lists **Open, Reveal, Info** when cold, or **Focus, Reveal, Info, Close,
  Kill** when it's the one running (Open becomes Focus since it already is
  one; Close sends a normal quit via `NSRunningApplication.terminate`, Kill
  force-quits via `forceTerminate`). `↩` runs the highlighted one; `⎋`
  dismisses the panel outright, backspace on an empty field steps back to
  the launcher, like a sigil mode. A `[shortcuts]` entry has no bundle
  behind it, so it's `⇥`-able only once it has commands of its own (below).
- `[commands.<App Name>]` — `"label" = "shell command"` extras appended
  after the built-ins above, matched case-insensitively by row name; `⇥`
  on a plain `[shortcuts]` entry needs at least one of these to do anything.
  Bind a different chord under `[keys]` with the `show-commands` action if
  `⇥` doesn't suit. Example:
  ```toml
  [commands.Switchblade]
  "downloads" = "$HOME/.cargo/bin/switchblade --fast-fullscreen ~/Movies/Downloads"
  ```
- `[animation]` — `fade`, default `false`. An `NSPanel` left at AppKit's
  default animation behavior gets the window server's utility-panel fade on
  every summon and dismiss; off, the panel is on screen the frame the hotkey
  lands. Motion rather than appearance, so it sits outside `[style]` and no
  theme can flip it.
- `[stats]` — `interval`, seconds between gauge refreshes while visible.
- `[modes]` — sigil assignment for the first-character modes: `math = "="`,
  `web = "!"` (the defaults); `"none"` disables one. The sigil is lifted out
  of the field into a colored box; backspace on an empty field returns to the
  launcher. `=4% of 100` shows `= 4` (`↩` copies); an empty `=` rests at
  `= 0`. `!yt cat videos` opens a YouTube search (`↩` opens).
- `[modes.web]` — `"prefix" = "https://…{q}"` web shortcuts for the `!`
  mode; the row title is the site name, derived from the domain unless given
  explicitly as `"prefix" = "Name | https://…{q}"`. Defaults: `g` (Google),
  `yt` (YouTube), `w` (Wikipedia).
- `[modes.currency]` — `targets`, a comma list of currency codes the `$`
  mode converts into (default `usd, eur, gbp, aud, btc`). `$500,000 php`,
  `$3k usd`, `$1.4btc` all work (`k`/`m`/`b` multipliers; no code = USD).
  Rates are Coinbase's keyless USD endpoint, fetched via `curl` in the
  background and cached at `~/.cache/motherfucker/rates.json`; the panel
  always renders from cache (never blocks) and the top row shows its age.

Math and currency also autodetect without their sigil, Spotlight-style:
every keystroke the bare query is reclassified, so `2+2` or `580 php` or
`$100` shows mode rows and the mode's sigil badge, and deleting back to a
non-match restores app results — the badge follows the classification,
the typed text stays in the field. Autodetection is stricter than the sigils
(math needs an operator, currency a typed symbol or a code from `targets`
or the symbol table), so `42` and `1Password` stay app queries; the sigil
remains the explicit override for input autodetection declines, like a
code-less `100`. Disabling a mode's sigil (`"none"`) disables its
autodetection too.

The parser is a ~100-line hand-rolled TOML subset (sections + `key =
value`) so the binary stays dependency-free.

## Why it can't jank

- **Rendering**: a non-activating `NSPanel` holding one `NSVisualEffectView`
  (dark HUD material + black tint). The blur is composited by the window
  server — same machinery as every native panel. We draw nothing but text;
  no icons, no images, no custom render loop.
- **Hotkey**: Carbon `RegisterEventHotKey`. No Accessibility or Input
  Monitoring permission, and unlike a CGEventTap it cannot stall system-wide
  key delivery if this process hangs.
- **Discovery is cache-free by design**: every summon (and every keystroke)
  re-readdirs `/Applications`, `/System/Applications`, their `Utilities`
  subdirs, and `~/Applications`. Names come from bundle filenames — file
  *contents* (Info.plist, .icns) are never read, so there is nothing to
  cache and nothing to go stale. Warm cost is microseconds.
- **Running apps** come from `NSWorkspace.runningApplications` (in-memory).
  Switching uses `NSRunningApplication.activate`. No AX API, no extra
  permissions.
- **Matching**: a ~80-line subsequence scorer (prefix > word boundary >
  camelCase > consecutive; gap penalty; running apps boosted). Microseconds
  over a few hundred names.

## Design

"Black glass": dark vibrancy at ~68% black, no bold text anywhere, hierarchy
by size and opacity only (24 px input → 15.5 px rows at 68% white → 12 px
metadata at 42%), selection is a 14% white tint, 7% white hairline border,
no divider lines, bordered hint chips (`↩ switch · ⌘↩ open · ⎋ dismiss`).

## Files

- `src/main.rs` — AppKit layer: panel, views, delegate, event handling
- `src/hotkey.rs` — Carbon global hotkey FFI (multi-hotkey, id dispatch)
- `src/config.rs` — config file: chords, actions, style (unit-tested)
- `src/apps.rs` — readdir scan + fuzzy scorer (unit-tested: `cargo test`)

## Not yet

- multi-window pick on switch (needs AX permission)
- more per-item commands on the selected row (⌘Q quit, ...)
- modes beyond `launcher` for extra hotkeys
