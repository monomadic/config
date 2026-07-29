# battery-widget

macOS menu bar battery widget in the same family as `free-disk-space-widget`
and `system-uptime-widget`, built on [menuet](https://github.com/caseymrm/menuet)
(native NSStatusItem bridge — no vendored systray fork).

## Layout styles

Pick from the Style submenu; persisted via NSUserDefaults
(`com.jayu.battery-widget`).

| Style | Menu bar |
|---|---|
| Text | `87%` |
| Icon and Text | `􀛨 87%` (bolt while charging, empty battery when low) |
| Bar and Text | `━━━━━━━╌ 87%` |
| Icon and Bar | dim bolt + bar |
| Percentage and Bar | `87% ━━━━━━━╌` |
| Bar | bar only |
| Bar and Power | bar + `8.4w` discharging / `+42w` charging |
| Smart Bar | state-aware (default) |
| Smart Bar and Timer | smart bar + `· 3:12` |

The smart bar colors by state: green + bolt while charging (text shows charge
rate), red below 10% while discharging, yellow in Low Power Mode; otherwise
default menu bar monochrome with power draw as the text. Bars are drawn as
box-drawing text runs (menuet status-item images are template-only and cached
by name, so pixel-drawn PNG bars like the disk widget's aren't possible here).

## Data sources

- `pmset -g batt` — percent, charge state, time estimate
- `ioreg -rn AppleSmartBattery` — amperage/voltage (power draw), cycle count,
  health (NominalChargeCapacity ÷ DesignCapacity)
- `pmset -g` — Low Power Mode state; toggling it runs `pmset -a lowpowermode`
  via an osascript admin prompt

## Install

```bash
setup/install/install-battery-widget.sh
```

Builds are committed to `vendor/bin/battery-widget`; the installer copies to
`~/.local/bin` and manages the `com.jayu.battery-widget` LaunchAgent.
