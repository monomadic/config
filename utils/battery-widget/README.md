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
