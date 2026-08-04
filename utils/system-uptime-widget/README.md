# system-uptime-widget

macOS menu bar widget for system uptime, in the same family as `battery-widget`
and `free-disk-space-widget`. Rust, talking to AppKit directly via
[objc2](https://github.com/madsmtm/objc2) — no wrapper library, no vendored
fork, no `.app` bundle. The dropdown is a real `NSMenu` assigned to the status
item, so macOS presents it natively.

## Menu bar

| Style | Menu bar |
|---|---|
| Icon above Text | clock glyph stacked over `22H` (default) |
| Icon and Text | `􂝔 22H` |
| Text | `22H` |

The chosen style persists to `~/.config/system-uptime-widget/style` and is
picked from the `Style` submenu in the dropdown.

The value is always one unit, capitalised — at menu bar size a lowercase `d` or
`h` hangs off the digits' x-height where a cap sits flush with them:

```text
42M
5H
2D
3.5D
33.1D
```

Under an hour it counts in minutes, then in whole hours, and past a day the
hours become a decimal rather than a second unit — one number is less to read
across in a menu bar than two. The decimal is truncated, not rounded, so the
figure never runs ahead of the machine. The tooltip still spells the full figure
out — "System uptime is 3 days, 12 hours".

Below `Style` the menu holds `Reboot`, `Shutdown` and `Quit`. Reboot and
Shutdown put up a confirmation dialog before telling System Events to perform
the power action.

## Sizing

No hardcoded point sizes. Everything is a ratio of the menu bar font
(`NSFont::menuBarFontOfSize(0.0)`) or of `NSStatusBar::thickness()`, so changing
the system text size takes the widget with it.

Side by side, the glyph is set at menu bar size and the value at the same 0.84em
compact size `free-disk-space-widget` uses for its Icon and Text style, which
keeps the two widgets visually matched. Stacked, both runs have to share the
height of the bar, so both come down — the glyph to 0.72em, the value to 0.58em
— and if that pair still overruns the bar it is scaled down until it fits rather
than being clipped. Digits are monospaced in every style, so the width doesn't
jitter as the number changes.

Both marks go into a single image rather than leaving the value in the button's
title: a status item button carrying both a title and an image pads generously
around each, and the glyph's own left side bearing lands on top of that. The
glyph is measured and placed by its ink (`CTLineGetBoundsWithOptions` with
glyph-path bounds), not its advance — stacked, the advance box's line height
alone would push the value out of the bar. Text-only skips the image and sets
the button's title, at full menu bar size.

## Data source

`kern.boottime` via `sysctlbyname`, not `NSProcessInfo systemUptime`. The
sysctl is a wall-clock instant, so the elapsed time it yields includes any
stretch the machine spent asleep — which is what "uptime" means to someone
reading a menu bar.

## Build and install

```bash
setup/install/install-system-uptime-widget.sh
```

Builds are committed to `vendor/bin/system-uptime-widget`; rebuild with
`cargo build --release` and copy, or let the installer's error hint guide you.
The installer copies to `~/.local/bin` and manages the
`com.jayu.system-uptime-widget` LaunchAgent.

`cargo test` covers the uptime formatting and the boot-time read.
