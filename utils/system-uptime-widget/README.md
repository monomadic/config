# system-uptime-widget

macOS menu bar widget for system uptime, in the same family as `battery-widget`
and `free-disk-space-widget`. Rust, talking to AppKit directly via
[objc2](https://github.com/madsmtm/objc2) — no wrapper library, no vendored
fork, no `.app` bundle. The dropdown is a real `NSMenu` assigned to the status
item, so macOS presents it natively.

## Menu bar

A clock glyph and the uptime beside it, drawn as one template image:

```text
􂝔 42m
􂝔 5h
􂝔 2d
􂝔 33d 3h
```

Only the two largest non-zero units, so the item keeps a steady width. Under an
hour it counts in minutes. The tooltip spells the same figure out —
"System uptime is 2 days, 1 hour".

The menu holds `Reboot`, `Shutdown` and `Quit`. Reboot and Shutdown put up a
confirmation dialog before telling System Events to perform the power action.

## Sizing

No hardcoded point sizes. The glyph is set in the menu bar font
(`NSFont::menuBarFontOfSize(0.0)`) and the value in the same 0.84em compact
size — monospaced digits, so the width doesn't jitter — that
`free-disk-space-widget` uses for its Icon and Text style, on a canvas the
height of `NSStatusBar::thickness()`. Change the system text size and the widget
follows, and the two widgets stay visually matched.

Both marks go into a single image rather than leaving the value in the button's
title: a status item button carrying both a title and an image pads generously
around each, and the glyph's own left side bearing lands on top of that. The
glyph is measured by its ink (`CTLineGetBoundsWithOptions` with glyph-path
bounds), not its advance, so macOS is left to pad the item evenly.

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
