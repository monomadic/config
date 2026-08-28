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
| Boxed Text | `22H` set small inside a thin rounded rule |
| Day Progress | whole days over a progress bar filled to the part-day |
| Days and Bar | `2D` beside a progress bar filled to the part-day |
| Icon, Bar and Dots | glyph, the day's bar, and a mark per day passed below it |

The chosen style persists to `~/.config/system-uptime-widget/style` and is
picked from the `Style` submenu in the dropdown. Each row in that submenu
carries the image that style would install, drawn from the current uptime — the
preview is the thing itself, not a mock of it, and it re-renders as the value
ticks over.

`Day Progress` is the one style whose digits are not the compact value below:
it shows whole days once there is a day (`3D`), otherwise
hours or minutes, and puts the fraction the digits drop into the bar instead —
so `3D` half-filled is three and a half days up.

`Days and Bar` makes the same split laid along the menu bar instead of across
it, in the layout `battery-widget` uses: the digits carry the whole days and the
bar, at that widget's own 40pt track and 6pt rule, carries only the fraction
they drop. Under a day the digits fall back to hours or minutes and the bar
still shows the part-day, so the gauge means one thing at every scale.

`Icon, Bar and Dots` drops the digits entirely. The glyph leads, the bar is the
day in progress, and below it a ledger counts the days already behind it — a dot
to the day, a mark three dots wide to the week, weeks first so the row reads
coarse to fine. The ledger's height is reserved from the first minute, so the
bar sits on one line for the whole first day rather than stepping up when the
first dot lands. Past three weeks a countable row would be longer than the gauge
above it, so the style hands over to `Days and Bar` — the same reading, carried
by digits instead of marks.

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

The menu opens with the figure spelled out in full — "1 day, 2 hours, 43 mins"
— as a heading with no action of its own. Below `Style` sit `Reboot`,
`Shutdown` and `Quit`, in one section. Reboot and Shutdown put up a
confirmation dialog before telling System Events to perform the power action.

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

Days and Bar sets the value at the same 0.84em compact size and puts a 2.85em
track with a 0.42em rule beside it, one 0.35em gap apart — 40pt and 6pt at a
14pt menu bar font, which is `battery-widget`'s gauge exactly. Icon, Bar and
Dots keeps that track but thins the rule to 0.36em to leave the ledger room:
0.21em dots on 0.21em gaps, week marks three dots wide, a 0.14em row gap. All
three gauges are drawn by one routine, so a fill shorter than the rule is drawn
at the rule's width rather than as a sliver too short to take its rounded cap.

Boxed sets the value at 0.72em and pads it to a rounded rule one point thick,
drawn at 75% alpha so the box reads as a container rather than as another mark
competing with the number. Day Progress sets the value at 0.62em over a
0.16em-tall bar — faint track, solid fill — and widens the item to 2.4em when
the digits alone would leave a track too short to read as a gauge.

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

`cargo test` covers the uptime formatting, the boot-time read, and the day
ledger's carry and width.
