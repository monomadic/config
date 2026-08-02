# battery-widget

macOS menu bar battery widget in the same family as `free-disk-space-widget`
and `system-uptime-widget`. Rust, talking to AppKit directly via
[objc2](https://github.com/madsmtm/objc2) — no wrapper library, no vendored
forks, no `.app` bundle requirement. The dropdown is a real `NSMenu` assigned
to the status item, so macOS presents it natively (instant, anchored to the
menu bar).

## Layout styles

Pick from the Style submenu; persisted to `~/.config/battery-widget/style`.

| Style | Menu bar |
|---|---|
| Text | `87%` |
| Icon and Text | `􀛨 87%` (bolt while charging, empty battery when low) |
| Bar and Text | bar + `87%` |
| Icon and Bar | dim bolt + bar |
| Percentage and Bar | `87%` + bar |
| Bar | bar only |
| Bar and Power | bar + `8.4w` discharging / `+42w` charging |
| Smart Bar | state-aware (default) |
| Smart Bar and Timer | smart bar + `· 3:12` |

The smart bar colors by state: green + bolt while charging (text shows charge
rate and time to full in the dropdown), red below 10% while discharging,
yellow in Low Power Mode; otherwise default menu bar monochrome with power
draw as the text.

## Animation

A second timer re-renders the cached reading at 20fps, and only runs while
something is actually animating. Only one thing in the widget moves at a
time — the bar is what breathes, and the bolt stays solid unless the charge
is critical:

- **≤8% and discharging** — the bar breathes red, and the bolt turns red and
  throbs fast. The only state that animates the bolt.
- **>95% charge** — the bar breathes in the menu bar's own foreground colour,
  so it reads white on a dark menu bar and adapts on a light one. This is
  charge level, not the health percentage in the dropdown.
- **Plugged in** — the bar breathes in the same foreground colour; the bolt
  does not move. "Plugged in" means `on_ac`, not `state == Charging`: at 100%
  pmset reports `charged`, which is still on the charger.

The bolt itself is a charge-level readout rather than a charging indicator:
yellow above 60%, white at low alpha as it drains, red once critical.

`Icon and Bar` draws the bolt into the bar image rather than setting it as a
title run, so the button has a single image to centre. A title run's glyph
side bearing is what left the content sitting off-centre in its capsule.

The track behind the fill stays at constant alpha, so the pulse reads as the
charge level fading rather than the whole widget blinking.

The bar is drawn as strikethrough space runs in the attributed title — the
strike renders as one thin continuous line, vertically centered. Glyph bars
(`━`, `█`) leave per-cell gaps or seams, and run backgrounds always fill the
whole line height. One AppKit quirk: a strikethrough on the title's leading
run doesn't render, so a plain padding run is inserted when a bar leads.

## Data sources

- `pmset -g batt` — percent, charge state, time estimate
- `ioreg -rn AppleSmartBattery` — amperage/voltage (power draw), cycle count,
  health (NominalChargeCapacity ÷ DesignCapacity)
- `pmset -g` — Low Power Mode state; toggling it runs `pmset -a lowpowermode`
  via an osascript admin prompt

## Build and install

```bash
setup/install/install-battery-widget.sh
```

Builds are committed to `vendor/bin/battery-widget`; rebuild with
`cargo build --release` and copy, or let the installer's error hint guide
you. The installer copies to `~/.local/bin` and manages the
`com.jayu.battery-widget` LaunchAgent.
