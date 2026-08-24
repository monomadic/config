# volume-control-widget

macOS menu bar widget for output devices, mute, and a gig guard — in the same
family as `battery-widget` and `free-disk-space-widget`. Rust, talking to
AppKit via [objc2](https://github.com/madsmtm/objc2) and to the CoreAudio HAL
directly through a handful of hand-declared calls — no wrapper library, no
vendored fork, no `.app` bundle.

The premise inverts the usual volume widget: **for a DJ, muted is the safe
state.** The widget's job is not to help you hear the volume — it is to make
it impossible to not notice when the machine can make sound, and to make sure
the internal speakers *can't*.

Design mockups this implements: `designs/` (the "guard synthesis" iteration).

## Rules

Every device row trails a circular lock button (the disk widget's eject
anatomy) that cycles the device's rule:

| Button | Rule | Meaning |
|---|---|---|
| open padlock, translucent | none | plain device: click row to route, click speaker to mute, click bar to set level |
| filled padlock, solid disc | **Always Mute** | held muted. A property listener re-mutes the instant anything unmutes it — an app, a HUD key, a reroute — from the HAL's own callback thread, with no polling window. The row wears a dashed border. |
| shield, blue disc | **Never Mute** | the performance output (the DJM). Read-only: the app never writes mute or volume to it. Its absence is a warning — the row stays listed in red, the menu bar goes red. |

Rules follow the CoreAudio device **UID**, not the transient device ID, so
they survive unplug/replug and reboots. Ruled devices are remembered
(name + transport) in the settings file so a disconnected device keeps its row.

The pre-gig ritual is two clicks, once: padlock the internal speakers, shield
the DJM. From then on: menu bar shows `DJM` when the route is confirmed
(header status in green), red warning + `DJM` when the mixer disappears, and
amber `INT` when macOS falls back to the (already muted, so silent) speakers.

## Menu bar styles

Picked from the Style submenu; persisted to
`~/.config/volume-control-widget/settings`.

| Style | Menu bar |
|---|---|
| Icon and Route *(default)* | speaker glyph + route tag (`DJM` / `INT`) |
| Icon | glyph only; colour still carries state |
| Icon and Bar | glyph + pill bar of the active output's volume (dim when muted) |
| Icon, Bar and Route | everything |
| Icon and Percent | glyph + `65%` |

Normal states render as a template image (menu bar monochrome). Alert states
opt out: red for an expected device missing, orange for fallback caught.

## The menu

A native `NSMenu`, rebuilt each time it opens. Header: "Audio Outputs" plus a
status line — green "DJM is active", red "missing", orange "fallback caught",
or the plain default-device summary. Then one custom row per output device:

- bare device glyph (laptop / headphones / AirPods / hifispeaker / display,
  from transport + name), state-tinted
- name, with a bus pill (`USB`, `BT`, `HDMI`, `DP`, `TB`, `AIRPLAY`) —
  built-in devices wear no pill
- spec line: `24bit 48khz – 12 in, 10 out` (physical stream format, nominal
  sample rate, stream-configuration channel counts)
- volume line: mute speaker at the head (click toggles; the macOS Sound
  slider's own anatomy), bar to the right (click sets the level)
- the lock button

Row click targets: lock cycles the rule and speaker toggles mute, both
leaving the menu open; anywhere else routes output to that device and closes
it. The blue row is the current output; ruled-but-disconnected devices sit
dimmed (or red, if shielded) at the bottom.

## Plumbing

Everything is event-driven CoreAudio property listeners — device list,
default output, and per-device mute/volume. Listeners set a dirty flag; a 1 s
timer repaints the status item only when something changed (plus a 30 s
belt-and-braces full refresh). Always-Mute enforcement runs inside the
listener callback itself, so it does not wait for the UI tick.

The widget never plays audio, never taps audio, and shows no realtime levels
by design — the bar is the volume *setting*. Transport version numbers
(`USB 2.0`, `BT 5.2` in the mockups) are not in the HAL, so pills carry the
bus name only.

## Build and install

```bash
cargo build --release --manifest-path utils/volume-control-widget/Cargo.toml
cp utils/volume-control-widget/target/release/volume-control-widget vendor/bin/
setup/install/install-volume-control-widget.sh   # ~/.local/bin + LaunchAgent
```

`volume-control-widget --dump` prints the HAL's device snapshot and exits —
for checking what the widget sees without launching the status item. Logs
land in `~/Library/Logs/volume-control-widget.err.log` under the LaunchAgent.
