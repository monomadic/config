# menu-tidy

Minimal Bartender/HiddenBar-style menu bar tidier, in the same family as
`battery-widget`. Rust talking to AppKit directly via
[objc2](https://github.com/madsmtm/objc2) — no wrapper library, no `.app`
bundle, ~200 lines.

## How it works

macOS has no API to hide another app's status item. What it does have is a
layout rule: status items are packed right-to-left, and **any item pushed left
of the frontmost app's menus stops being rendered**. That suppression is the
hiding mechanism — the icons still exist and still run, they just aren't drawn.

Two status items:

- a **spacer**, invisible, sitting immediately left of the marker. Widening it
  shoves its left-hand neighbours past the boundary, which hides them. The
  spacer stops being drawn at that width too — expected, since it draws nothing.
- the **marker**, the thing you click. It keeps its natural width, so it is
  always drawn, and because a status item's position depends only on the items
  to its *right*, it never moves when icons on its left come and go.

That split is not incidental. A single item cannot do both jobs: to hide its
neighbours it must overflow the renderable region, but to stay visible itself it
must fit inside that same region. Measured on a 1728pt screen, one item at
10,000pt landed at `x = -4052..964` and macOS stopped drawing it — the app
looked like it had never launched. Narrowing it enough to stay drawn made macOS
re-pack the bar instead, dragging icons round from the right and shifting the
marker by hundreds of points on every toggle. With two items the marker holds
its exact position: `1139..1170` collapsed and expanded alike.

The marker is drawn as a template image on a canvas sized for the largest frame
of the animation (and for the wider of the style's two glyphs), so the item
keeps one width throughout. Animating the font size of a plain title instead
would resize the item 30 times a second and drag every icon beside it around.
Dimming is alpha in that template image rather than a colour, which keeps it
correct in both light and dark menu bars.

A zero-length spacer still reserves ~16pt, which shows as a small gap between
the revealed icons and the marker; it reads as a separator, so it is left alone.
`setVisible(false)` removes the gap but makes macOS re-place the spacer on the
way back, stranding icons between it and the marker.

- **Left click** the marker: toggle. Collapsed = icons hidden.
- **Appearance**: at rest — icons hidden, pointer elsewhere — the marker is
  drawn at 40% alpha so it sits quietly in the bar. Hovering brings it to full
  strength and kicks off a bounce: a fast elastic overshoot to ~1.25x that
  settles into a gentle 1.1Hz throb. It keeps throbbing while the icons are out
  and goes back to dim and still once they hide again.
- **Auto-hide**: while expanded, a 0.25s poll of `NSEvent.mouseLocation` checks
  whether the pointer is in the top 30pt strip of any screen; once it has been
  away for 5 seconds, it collapses again. No permissions needed (unlike global
  event monitors).
- **Right click / ctrl-click**: menu with a **Marker** submenu for picking the
  glyph, plus Quit. The menu is attached only for that click and detached in
  `menuDidClose`, so left clicks keep toggling.

The marker starts expanded on launch and auto-collapses via the same timer.

## Saved state

Everything lives in `~/Library/Preferences/menu-tidy.plist`; there is no config
file of its own.

| Key | Written by | Meaning |
|---|---|---|
| `NSStatusItem Preferred Position menu-tidy` | macOS (`autosaveName`) | marker position |
| `NSStatusItem Preferred Position menu-tidy-spacer` | seeded once, then macOS | spacer position |
| `marker-style` | the Marker submenu | chosen glyph |

Seven marker designs are offered: Triangle, Chevron, Chevron (bold), Angle,
Arrow, Dots and Bars.

Preferred position counts **up leftwards**, so the spacer is seeded at the
marker's value + 1 to sit on its left. Collapsed/expanded state is not
persisted — it always starts expanded.

## Setup

One-time: **⌘-drag** the menu bar icons you want tucked away so they sit to the
*left* of the marker. Only icons left of the spacer hide, and the spacer sits
immediately left of the marker, so drag past both. If the marker is already the
leftmost item there is nothing to its left to hide — drag it rightwards, or drag
icons across it.

## Build and install

```bash
setup/install/install-menu-tidy.sh
```

Builds with cargo, installs to `~/.local/bin`, and manages the
`com.jayu.menu-tidy` LaunchAgent (start at login, keep alive).

## Debugging

`MENU_TIDY_DEBUG=1 menu-tidy` logs screen geometry at startup, then the style,
expanded/hover state, last rendered (scale, alpha) and both items' positions and
draw state on every change and every 2s. The frame read immediately after
`setLength` is pre-relayout and misleading — the polled value is the real one.

The frame alone will not tell you whether the item is being *drawn*. For that,
read `kCGWindowIsOnscreen` from `CGWindowListCopyWindowInfo` for layer-25
windows: every menu bar item, including other apps', shows up there with its
bounds and on/off state, which is how the suppression behaviour above was
found. Note that all of them read `OFF` whenever a fullscreen window is
frontmost and the menu bar is hidden — compare against a neighbour rather
than reading one item in isolation.
