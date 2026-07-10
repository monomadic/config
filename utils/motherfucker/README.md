# motherfucker

Cache-free, single-binary, minimalist Spotlight replacement. App launcher and
switcher only. The core principle is **no jank**: whatever state the system is
in, this thing stays rock solid.

## Usage

Run the binary; it sits resident (a few MB, zero idle CPU) and waits for
**⌥Space**. Type to filter, `↑`/`↓` to select, `↩` to switch/open, `⌘↩` to
force-open (reopen event), `⎋` or clicking away dismisses. Summoning with an
empty query lists running apps — it's an app switcher by default.

Install: `setup/install/motherfucker.sh` — builds release, installs to
`~/.bin`, and loads a LaunchAgent (`com.nom.motherfucker`) so it starts at
login and relaunches if it ever dies. Re-run the script after code changes;
it reloads the agent. Uninstall: `launchctl bootout gui/$(id -u)
~/Library/LaunchAgents/com.nom.motherfucker.plist` and delete the plist.

To take over **⌘Space**, disable Spotlight's shortcut first (see
`spotlight-manager` in `config/zsh`), then change `SUMMON_MODS` in
`src/main.rs` from `MOD_OPTION` to `MOD_CMD` and rebuild.

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
- `src/hotkey.rs` — Carbon global hotkey FFI
- `src/apps.rs` — readdir scan + fuzzy scorer (unit-tested: `cargo test`)

## Not yet

- multi-window pick on switch (needs AX permission)
- configurable hotkey (currently a constant)
- per-item commands on the selected row (⌘Q quit, ⌘R reveal, ...)
