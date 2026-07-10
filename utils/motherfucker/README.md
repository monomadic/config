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
while the panel is up.

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
  `move-down`; `"none"` unbinds a default.
- `[style]` — `width`, `panel_background`/`panel_foreground`/`panel_opacity`/
  `panel_padding`/`panel_corner_radius`, `item_foreground`/`item_font_size`/
  `item_foreground_highlight`, `selected_item_background`/
  `selected_item_foreground`/`selected_item_opacity`/
  `selected_item_corner_radius`/`selected_item_foreground_highlight`,
  `input_font_size`. Colors are `"#rrggbb"`; the highlight keys color the
  query-matched characters.
- `[icons]` — `search`, `running_many`/`running_one`/`running_none`/
  `installed` (literal glyph strings; SF Symbols pasted as text work), and
  `utilities`/`system`/`applications`/`shortcut` (SF Symbol names for the
  tag pills).
- `[icons.apps]` — `"App Name" = "glyph"` per-app overrides for the
  row-state glyph (case-insensitive name match).
- `[shortcuts]` — `"Name" = "shell command"` custom entries, matched like
  apps and run via `sh -c` on activation.
- `[stats]` — `interval`, seconds between gauge refreshes while visible.

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
