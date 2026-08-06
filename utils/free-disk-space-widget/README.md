# free-disk-space-widget

macOS menu bar widget for free disk space, in the same family as
`battery-widget`, `job-monitor` and `menu-tidy`. Rust, talking to AppKit directly
via [objc2](https://github.com/madsmtm/objc2) — no wrapper library, no vendored
fork, no `.app` bundle. The dropdown is a real `NSMenu` assigned to the status
item, so macOS presents it natively, and it is rebuilt from the mount table every
time it opens.

## Menu bar

| Style | Menu bar |
|---|---|
| Icon, Text and Bar | icon + compact `137gb` over bar |
| Text | `137 GB` |
| Icon and Text | icon + compact `137gb` |
| Bar and Text | bar + `137 GB` (default) |
| Icon and Bar | `􀤂` + bar |
| Bar | bar only |

`Show In Menu Bar` switches the number between free space and a percentage.
Below 10% free the glyph swaps to the warning drive and the title and bar turn
red. The bar fills with *free* space, so it drains as the disk fills.

Icon and Text and the stacked Icon, Text and Bar style use a rounded integer
with a lowercase, unspaced unit (`3gb`, `137gb`, `2tb`) to stay narrow. They
also share the same smaller text size. Other styles and the volume menu retain
standard macOS-like formatting.

Both submenu selections persist to `~/.config/free-disk-space-widget/settings`
as `key=value` lines.

## Volumes

Every user-visible mounted volume is listed at the top of the menu — startup disk
first, then alphabetical — as a Finder-sidebar-style row: an outline SF Symbol
for the volume kind (`internaldrive` / `externaldrive` / `network`, from the
`NSURLVolumeIsInternalKey`/`IsLocalKey` resource values) on a dark disc, its name
with the free amount small and right-aligned, and a capacity bar underneath that
fills with *used* space and turns red past 90%. Rows are custom `NSView`s; a menu
item's own title can do none of that.

The two discs are deliberately opposite polarities — dark behind the icon, light
behind the eject mark — so the ends of the row balance in weight while reading as
different kinds of control.

Two click targets per row, as in Finder. Clicking the row opens the volume in
Finder (the row tracks the mouse with an `NSTrackingArea` and draws its own
menu-style highlight). The eject button — a translucent circle that brightens
under the pointer, drawn and hit-tested by the row itself — sits in its own
always-visible column at the far right, for everything except the startup disk
and volumes mounted under `/System` or `/private`; it closes the menu and calls
`NSFileManager unmountVolumeAtURL:` with no options, which lets macOS put up its
own "disk in use" dialog exactly as Finder's eject does. Failures land on stderr
(`~/Library/Logs/free-disk-space-widget.err.log` under the LaunchAgent).

## Sizing

In Icon and Bar the glyph is drawn *into* the bar image rather than left in the
button's title: a status item button carrying both a title and an image pads
generously around each, and the glyph's own left side bearing lands on top of
that. The canvas is measured from the glyph's ink (`CTLineGetBoundsWithOptions`
with glyph-path bounds), not its advance, which leaves the item padded evenly on
both sides by macOS alone — 9pt each way at the default text size.

Icon, Text and Bar also draws the whole layout as one image. Its value is
left-aligned directly above the bar, beside the disk glyph, so it uses available
height instead of adding another horizontal run.

No hardcoded point sizes. The title uses the menu bar font
(`NSFont::menuBarFontOfSize(0.0)`, with monospaced digits so the width doesn't
jitter as the number changes), menu rows use the menu font
(`NSFont::menuFontOfSize(0.0)`), and the drawn bar is expressed as ratios of
those fonts' metrics, clipped to `NSStatusBar::thickness()`. Change the system
text size and the widget follows.

## Data source

`NSFileManager` and per-URL resource values, not `df`:

- `mountedVolumeURLsIncludingResourceValuesForKeys:options:` with
  `SkipHiddenVolumes` — the same set Finder shows
- `NSURLVolumeAvailableCapacityForImportantUsageKey` — the number Finder and
  System Settings quote, purgeable space included. Only local APFS/HFS+ volumes
  report it, so exFAT drives and network shares fall back to
  `NSURLVolumeAvailableCapacityKey`
- `NSURLVolumeTotalCapacityKey`, `NSURLVolumeNameKey`,
  `NSURLVolumeIsBrowsableKey`, `NSURLVolumeIsRootFileSystemKey`

## Build and install

```bash
setup/install/install-free-disk-space-widget.sh
```

Builds are committed to `vendor/bin/free-disk-space-widget`; rebuild with
`cargo build --release` and copy, or let the installer's error hint guide you.
The installer copies to `~/.local/bin` and manages the
`com.jayu.free-disk-space-widget` LaunchAgent.

`cargo test` covers the mount table and the capacity formatting.
