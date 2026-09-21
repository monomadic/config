# wifi-widget: macOS menu bar Wi-Fi status widget

Status: **design settled, P0 spikes done (see `TASKS.md` §0.1), no widget code written**. Mockups live in
`designs/` (`v1`, `v2`, `v3`, plus `alt-black-panel` and the
original `menu-bar-concepts` exploration). `v3` is the design to build.
Scope: macOS only, Rust against `objc2`, same family as `battery-widget`,
`free-disk-space-widget` and `volume-control-widget`.

## 1. Recommendation

Build `src/wifi-widget/` as a Rust binary with a real `NSMenu`, following
`volume-control-widget`'s module split, **wrapped in a plain `.app` that you
open like any other app** — no LaunchAgent, no installer. The spikes showed why:
macOS only reveals the SSID and BSSID to a process with Location permission,
only an app bundle can be granted it, and even then only when started through
LaunchServices (double-click, `open`, Login Items) rather than by launchd.
"Open at Login" replaces the LaunchAgent; restart-on-crash is given up.

The widget's job is not to restate what the system Wi-Fi icon already shows.
It exists to answer the three questions the system icon cannot:

1. **Is the link actually good?** Full bars mean nothing without the noise
   floor, so every meter is **SNR** (RSSI − noise), not RSSI.
2. **Is the internet working?** A perfect link with a dead uplink looks
   identical to a healthy one in the system icon.
3. **Why is it slow?** Band fallback to 2.4GHz, a sticky association to a far
   mesh node, or a silent fall back to the iPhone hotspot.

Keep the reading cheap and event-driven. `CWEventDelegate` supplies link, SSID,
BSSID and RSSI change events, so unlike `battery-widget` there is no polling
loop for the radio itself. Only the internet probe and the router/WAN poll run
on timers.

## 2. What already exists

| Thing | State |
|---|---|
| `src/wifi-widget/designs/v3/` | the design being built: graphite connected card, Known Networks list, QR codes, five menu bar styles, six scenarios |
| `src/wifi-widget/designs/alt-black-panel/` | rejected for now: needs a custom `NSPanel` instead of `NSMenu` |
| `scripts/wifi-qr` | **broken on current macOS** — finds the SSID with the removed `airport` CLI. The widget's QR button supersedes it; the script should be fixed or deleted separately |
| `src/battery-widget` | reference for status item, Style submenu, attributed-title bar drawing |
| `src/volume-control-widget` | reference for custom `NSView` menu rows, per-row controls, settings file, `--dump` |

## 3. Data sources

Verified on this machine (2026-09-20, macOS 26 / Darwin 25.5, GL-BE9300 Flint 3):

| Source | Gives | Status |
|---|---|---|
| `CWInterface` | `rssiValue`, `noiseMeasurement`, `transmitRate`, `wlanChannel` (band/width), `activePHYMode`, `ssid`, `bssid`, `hardwareAddress` | public API, needs Location authorization for SSID/BSSID |
| `CWEventDelegate` | link, SSID, BSSID, RSSI, power change events | public API |
| `CWConfiguration.networkProfiles` | the saved ("known") network list, in order | public API |
| `CWInterface.scanForNetworks` | nearby networks with RSSI/channel | **7.8 s measured**, blocks its thread, and returns `ssid = nil` without Location; run on menu open and Refresh only |
| `NWPathMonitor` | `isExpensive` (hotspot/cellular), `isConstrained` (Low Data Mode) | drives Metered detection |
| `captive.apple.com/hotspot-detect.html` | internet reachable / captive portal + redirect host | plain HTTP on purpose |
| GL.iNet JSON-RPC at `http://192.168.1.1/rpc` | WAN up/down, uptime, active uplink, byte counters | **verified the endpoint answers a `challenge` request**; method names for the WAN figures are **not yet verified** and need a logged-in session |
| `/usr/bin/networkQuality` | download/upload capacity, responsiveness | present on this machine; ~15 s, hundreds of MB |
| Keychain (`security find-generic-password` or `SecItemCopyMatching`) | Wi-Fi password for the QR payload | prompts for an administrator password |
| Core Image `CIQRCodeGenerator` | the QR image | no crate dependency needed |

Ruled out here: **UPnP IGD** (SSDP got no reply) and **NAT-PMP** (timed out).
Both are worth one cheap attempt at runtime for other networks, but no feature
should depend on them.

Not available, as far as I can establish: the Instant Hotspot metadata Apple's
own menu shows (phone battery, cellular bars). The design says
`Personal hotspot · metered` instead.

## 4. Module layout

```
src/wifi-widget/
  Cargo.toml          objc2, objc2-app-kit, objc2-foundation, block2 (match battery-widget's feature lists)
  src/
    main.rs           status item, menu assembly, state machine, timers
    wifi.rs           CoreWLAN FFI: interface state, scan, known networks, events
    probe.rs          internet + captive portal check, gateway reachability
    router.rs         GL.iNet RPC client, Keychain-stored credentials, WAN sample ring buffer
    speed.rs          networkQuality runner (on demand only)
    qr.rs             Wi-Fi join payload + CIQRCodeGenerator image
    bar.rs            menu bar chip: glyph, four segments, tag
    card.rs           the connected card view (row + inset, hover-to-copy, QR button)
    row.rs            compact known-network row view
    settings.rs       key=value store at ~/.config/wifi-widget/settings
```

## 5. State machine

One enum drives the header dot, the chip colour and the primary action:

| State | Entered when | Header | Chip (Smart Bar) | Primary action |
|---|---|---|---|---|
| `Healthy` | associated, SNR ≥ 25, probe OK | green | glyph + segments + band (`6GHz`) | — |
| `BandFallback` | on 2.4GHz **and** the same SSID is seen on 5/6GHz in the last scan | amber | alert glyph + segments + `2.4GHz` | Rejoin |
| `Weak` | SNR < 15 | red | alert glyph + segments + `−81 dB` | Rejoin |
| `NoInternet` | associated, probe fails, no portal redirect | red | alert glyph only, **no segments** | — (Refresh re-runs it) |
| `LoginRequired` | probe redirected or body mismatch | amber | alert glyph + segments + `Login` | Open Login Page |
| `MeteredFallback` | current path `isExpensive` | amber | link glyph + segments + data used | Disconnect Hotspot |

SNR tiers, used by both the menu bar segments and the in-menu meter:
`< 15` poor (1 segment, red), `15–25` fair (2), `25–40` good (3), `≥ 40`
excellent (4). Clamp the meter to 0–50 dB.

**Hysteresis matters more than precision.** RSSI moves constantly. Require a
state to hold for ~10 s (and 3 dB of margin around tier edges) before the menu
bar changes, otherwise the widget flickers between amber and monochrome while
the laptop sits still.

## 6. Detection rules (no user configuration)

- **Metered**: `NWPathMonitor.isExpensive || isConstrained`. Remember flagged
  SSIDs in the settings file so a known hotspot shows its dashed row before you
  join it. Disable the speed test on these networks.
- **Home**: the network whose router credentials are stored; otherwise the
  network with the most accumulated connected time, which the widget tallies
  itself. Home turns red **only** on an involuntary transition — Home
  disappears and macOS joins something else. Joining another network yourself
  leaves Home dimmed as "Not in range".

## 7. Menu structure (v3)

```
● Wi-Fi – Connected – excellent signal      ← one line, dot inset equally from top and left
┌─ connected card (graphite, state colour in glyph + meter) ─┐
│ [glyph]  Studio  (6 GHz)                        [QR]       │  ← QR top-right
│          Wi-Fi 6E · ch 37 · 160 MHz                        │
│          ▬▬▬▬▬▬▬▬ 41 dB                                    │
│   −52 signal      −93 noise      1201 Mbps                 │  ← three equal columns
│   Internet  ONLINE 18 ms                                   │
│   WAN       ↓3.2 ↑0.4 MB/s  ╭╴sparkline╶╮                  │  ← router-backed, home only
│   Speed     ↓412 ↑38 Mbps · 2 h ago         Test           │
│   IP / Gateway / MAC / Node                  ⧉ hover-copy  │
└────────────────────────────────────────────────────────────┘
Known Networks                                    [Refresh]
  ≈ Studio-IoT  (2.4 GHz)                34 dB    [QR]        ← one compact line each
  ⛓ iPhone            hotspot · metered           [QR]        ← dashed border
  ≈ Grind Coffee         Not in range             [QR]
Other Networks ▸
Style ▸ · Wi-Fi Settings… · Quit
```

Layout constants worth keeping (measured in the mockup): panel padding 7px;
card inset 9px; icon column 40px on the card, 26px in the list; rows carry a
9px right inset so every QR button sits on one axis; the header dot sits 13px
from the panel's top and left edges.

## 8. AppKit specifics to get right

- **Live updates while the menu is open.** `NSMenu` tracking runs the run loop
  in `NSEventTrackingRunLoopMode`; timers added only to the default mode stop
  firing. Add the refresh timer to the tracking mode or the WAN sparkline
  freezes exactly when the user is looking at it.
- **QR codes open in their own window**, centred on screen at ~300 px, not
  inside the menu: bigger codes scan more reliably and the menu never has to
  re-lay-out. Clicking anywhere or pressing Esc closes it.
- **View-based items and submenus.** Confirm that a custom-view item can still
  open a submenu before relying on it for `Other Networks ▸`, otherwise make it
  a plain item.
- **Template vs colour.** The chip is a template image when monochrome, and a
  coloured attributed title otherwise, as `battery-widget` does.
- **Redacted mode.** Without Location authorization the card shows
  `Wi-Fi · 6 GHz`, the Node row and the Known Networks list are hidden, and one
  item requests permission.

## 9. Prioritised tasks

**P0 — the widget exists and is honest** (target: usable daily)

1. Scaffold `src/wifi-widget` + `bundle.sh`, which wraps the release binary
   into `WiFi Widget.app` (see §1 on why it must be an app). No LaunchAgent.
   *Done when* double-clicking the `.app` shows a static glyph and no Dock icon.
2. `wifi.rs`: interface read + `CWEventDelegate`, plus a `--dump` flag printing
   every field. *Done when* `--dump` matches `system_profiler SPAirPortDataType`
   on this Mac, including the Location-denied case.
3. SNR model, tiers and hysteresis. *Done when* unit tests cover tier edges and
   the anti-flap rule; no AppKit needed for those tests.
4. `bar.rs`: five styles, four segments, per-state glyph and tag exactly as in
   v3. *Done when* every scenario in the mockup can be reproduced by forcing a
   state.
5. `probe.rs`: internet + captive check every 30 s and on link events, with the
   `NoInternet` / `LoginRequired` split and **Open Login Page**. *Done when*
   pulling the router's WAN cable produces `NoInternet` within ~30 s, and a
   café portal produces `LoginRequired` with a working browser launch.
6. Menu skeleton: one-line header, connected card, Known Networks rows,
   Refresh, Style, Wi-Fi Settings…, Quit.

**P1 — the parts that make it worth opening**

7. Hover-to-copy rows (IP, gateway, MAC with the `PRIVATE` tag, node + BSSID).
8. QR codes: `qr.rs` payload + Core Image image, inline panel, Keychain
   password fetch with the admin prompt, `nopass` path for open networks.
   *Done when* a phone joins Studio from the code.
9. Metered/Home detection via `NWPathMonitor` + the time tally, including the
   remembered-SSID case.
10. Band fallback detection (needs scan results correlated by SSID) and the
    **Rejoin** action, with the documented fallback to opening Wi-Fi Settings
    if macOS demands admin rights.

**P2 — the outside view**

11. `router.rs`: GL.iNet login (password in Keychain), WAN status, byte
    counters, sparkline ring buffer. **First task is a spike** to confirm the
    RPC method names; if they do not exist, degrade to gateway-reachability
    only and keep the row hidden.
12. `speed.rs`: `networkQuality -c`, on demand, disabled on metered networks.
13. Opportunistic UPnP IGD attempt for non-GL routers, behind a 2 s timeout.

**P3 — nice to have, explicitly deferred**

14. `Other Networks ▸` join flow (password entry is a system dialog; consider
    deferring to Wi-Fi Settings).
15. Fix or delete `scripts/wifi-qr` now that the widget covers it.
16. macOS 27 `NSStatusItem` expanded interface sessions + `NSGlassEffectView`,
    which would make `alt-black-panel` buildable with system-managed
    positioning, focus and dismissal. Revisit after upgrading; if it lands,
    build it as a shared crate for all the widgets, not just this one.

## 10. Risks and unknowns

- **GL.iNet RPC surface is unverified.** Only the unauthenticated `challenge`
  call has been confirmed. P2 must start with a spike, and the WAN row must be
  optional everywhere in the UI.
- **Keychain prompts.** Wi-Fi passwords live in the System keychain; expect an
  administrator prompt per reveal, with no "always allow". If that proves too
  intrusive, the QR button should be limited to the current network.
- **Instant Hotspot passwords** may not be in the keychain at all, since those
  joins go through iCloud. The hotspot QR may have nothing to show.
- **Rejoin may require admin rights** on current macOS. The fallback is
  specified; verify before promising the action in the UI.
- **Portals that hijack DNS** rather than redirecting still get caught by the
  body check, but the redirect host shown in the menu will be missing.
- **Scanning is disruptive.** Never scan on a timer; menu-open and Refresh only.

## 11. Verification

There is no CI. For this widget:

- `cargo test` in `src/wifi-widget` for the pure logic: SNR tiers, hysteresis,
  state classification, QR payload escaping, GL.iNet response parsing.
- `--dump` compared against `system_profiler SPAirPortDataType` and
  `ipconfig getsummary en0`.
- Manual scenario pass, since the six states are the product: unplug the WAN,
  force 2.4GHz on the Flint 3, walk to the far room, join a café portal, join
  the iPhone hotspot, and deny Location once.
- `bash -n` on `bundle.sh`; `scripts/setup/check.sh` if any Dotter manifest
  entry is added (none is needed — `src/**` is not deployed by Dotter).
