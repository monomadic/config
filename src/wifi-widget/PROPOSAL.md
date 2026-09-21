# wifi-widget: macOS menu bar Wi-Fi status widget

Status: **design settled, P0 spikes done, no widget code written.**
This document is the *why* and the *shape*; [`TASKS.md`](TASKS.md) is the only
plan and the record of spike results — when the two disagree, `TASKS.md` wins.
Mockups live in `designs/`; `v3` is the design to build (`alt-black-panel` and
`menu-bar-concepts` are kept for reference). Scope: macOS only, Rust against
`objc2`, same family as `battery-widget`, `free-disk-space-widget` and
`volume-control-widget`.

## 1. Recommendation

Build a Rust binary with a real `NSMenu`, **wrapped in a plain `.app` that you
open like any other app** — no LaunchAgent, no installer. macOS only reveals
the SSID and BSSID to a process with Location permission, only an app bundle
can be granted it, and only when started through LaunchServices (double-click,
`open`, Login Items), not by launchd. "Open at Login" replaces the LaunchAgent;
restart-on-crash is given up. A rebuild re-prompts for Location once; accepted.

The widget exists to answer three questions the system Wi-Fi icon cannot:

1. **Is the link actually good?** Full bars mean nothing without the noise
   floor, so the meter is **SNR** (RSSI − noise), not RSSI.
2. **Is the internet reachable?** A perfect link with a dead uplink looks
   identical to a healthy one in the system icon.
3. **Why is it slow?** Band fallback to 2.4GHz, a sticky association to a far
   mesh node, a fall back to the iPhone hotspot.

**The widget tells the truth, and says how it knows.** Every value is either an
*observed fact* (RSSI, the probe result, `isExpensive`) or an *inference*
(Home, "this network change was unexpected"). The model keeps the two apart,
inferences carry a confidence, and the UI never presents a guess with the
alarm level of a measurement. This principle decides most of the edge cases
below.

The distinctive value is **local Wi-Fi interpretation**. Router telemetry is a
bonus for the one router in the house, not a product direction: no more vendor
integrations unless router monitoring becomes the point.

## 2. What already exists

| Thing | State |
|---|---|
| `designs/v3/` | the design being built: graphite connected card, Known Networks, QR lightbox, menu bar styles, six scenarios |
| `designs/alt-black-panel/` | rejected for now: needs a custom `NSPanel` instead of `NSMenu` |
| `scripts/wifi-qr` | **broken on current macOS** — finds the SSID with the removed `airport` CLI; superseded by the QR button |
| `src/battery-widget` | reference for status item, Style submenu, attributed-title bar drawing |
| `src/volume-control-widget` | reference for custom `NSView` menu rows, per-row controls, settings file, `--dump` |

## 3. Data sources

Verified on this machine (September 2026, macOS 26.5, GL-BE9300 Flint 3) unless
marked otherwise.

| Source | Gives | Notes |
|---|---|---|
| `CWInterface` | RSSI, noise, tx rate, channel/band/width, PHY mode, SSID, BSSID, MAC | SSID/BSSID need Location. **RSSI intermittently reads 0** on a live link — a dropped sample, not a value. `CWPHYMode` has no Wi-Fi 7 constant |
| `CWEventDelegate` | link, SSID, BSSID, link-quality, power events | callbacks off the main thread; delegate is weak. **No link-quality events fired in a 30 s idle run**, so live signal needs polling (§7) |
| `CWConfiguration.networkProfiles` | saved network names, in macOS's order | **readable without Location** |
| `CWInterface.scanForNetworks` | nearby networks, RSSI, channel | **7.8 s, blocks its thread**; SSID/BSSID are `nil` without Location |
| `nw_path_monitor` | `isExpensive`, `isConstrained`, status | hand-declared C FFI (no crate); two **separate** facts (§5) |
| default route + ARP table | gateway address; whether its MAC is resolved | link-level evidence the router is there; *not yet verified* |
| `captive.apple.com/hotspot-detect.html` | whether one well-known HTTP endpoint answers as expected | evidence of reachability, not proof "the internet works" |
| GL.iNet JSON-RPC at `/rpc` | WAN up/down, uptime, uplink, byte counters | only the unauthenticated `challenge` is verified; method names unknown |
| `/usr/bin/networkQuality` | capacity and responsiveness | ~15 s and hundreds of MB; on demand only |
| Keychain | Wi-Fi password for the QR payload | *unverified*: item fields, prompts and WPA3/enterprise behaviour — spike first |
| Core Image `CIQRCodeGenerator` | QR image | verified via `objc2-core-image` |

Dropped: **UPnP IGD** and **NAT-PMP** (no reply here; inconsistent everywhere;
weak data risks untrue status). Not available: the Instant Hotspot phone
battery and cellular bars Apple's own menu shows.

## 4. Architecture

```
src/wifi-widget/
  bundle.sh           cargo build --release → WiFi Widget.app
  src/
    main.rs           app, status item, menu assembly, timers
    model.rs          Store: merges sources into Snapshot; derives facts and headline
    wifi.rs           CoreWLAN: interface, scan, known networks, events
    path.rs           nw_path_monitor FFI: expensive / constrained / gateway
    probe.rs          HTTP probe → ProbeStatus
    router.rs         GL.iNet RPC (optional, isolated)
    speed.rs          networkQuality runner
    qr.rs             Wi-Fi join payload + CIQRCodeGenerator; lightbox window
    bar.rs            menu bar chip
    card.rs           connected card view
    row.rs            known-network row view
    settings.rs       persistence
```

**One store, one immutable snapshot.** Seven asynchronous sources — CoreWLAN
callbacks, the path monitor's dispatch queue, timers, background scans, TCP
probes, router RPC and a subprocess — never touch the UI. Each posts its result
to the store on the main thread; the store builds a new `Snapshot` and marks
the UI dirty; the UI redraws from the snapshot at most every 250 ms. `--dump`
prints the same snapshot, so diagnostics and UI can never disagree.

```
CoreWLAN ─┐
NWPath ───┤
Probe ────┤
Scan ─────┼─→ Store ─→ Snapshot { facts…, inferences…, headline } ─→ UI, --dump
Router ───┤
Speed ────┘
```

**Every measurement carries its age.** `Sample<T> { value, at: Instant }`, read
as *fresh*, *stale* or *unknown* by age (10–20 s for link values). After wake,
a run of zero RSSI must not keep showing the last good −52 dBm as current.

## 5. Facts, inferences and the headline

Facts are kept separately so nothing is lost when one headline wins:

```rust
struct Snapshot {
    link: LinkHealth,          // off / disconnected / associated; RSSI, noise, SNR, tier
    probe: ProbeStatus,        // Reachable{latency} | Captive{host} | DnsFailure | ConnectFailure | ReadFailure | UnexpectedResponse
    path: PathFlags,           // expensive, constrained (Low Data Mode) — two booleans, never merged
    band: BandStatus,          // current band; faster band of the same SSID seen, with RSSI
    gateway: GatewayEvidence,  // route present, ARP resolved; optional service response as metadata only
    home: Option<HomeGuess>,   // inference, with confidence
    change: Option<NetworkChange>, // UnexpectedNetworkChange{from, to} — observed, no claim about intent
    headline: State,
}
```

`headline_state(&Snapshot) -> State` picks one alarm for the menu bar; the
connected card still shows every fact. Precedence:
`WifiOff > Disconnected > NoInternet > LoginRequired > Weak > MeteredFallback >
BandFallback > Healthy`.

| State | From facts | Smart Bar chip | Primary action |
|---|---|---|---|
| `Healthy` | associated, SNR ≥ 25, probe reachable | glyph + segments + band (`6GHz`) | — |
| `BandFallback` | on 2.4GHz and a faster band of the same SSID seen (hysteresis below) | alert glyph + segments + `2.4GHz` | Rejoin *(if the spike keeps it)* |
| `Weak` | SNR tier poor | alert glyph + segments + `−81 dB` | Rejoin *(if kept)* |
| `NoInternet` | associated, probe DNS/connect/read failure twice | alert glyph only | — |
| `LoginRequired` | probe `Captive` | alert glyph + segments + `Login` | Open Login Page |
| `MeteredFallback` | `path.expensive` | link glyph + segments + data used | Disconnect Hotspot |

**Signal.** SNR tiers: `< 15` poor, `15–25` fair, `25–40` good, `≥ 40`
excellent → 1–4 segments; meter 0–50 dB; 3 dB margins and a 10 s hold before
the menu bar changes. If noise is unavailable, **show RSSI in dBm, never an
invented SNR**; segments come from a separate RSSI tier table.

**Band fallback** has its own hysteresis, because scan RSSI wanders: enter when
the faster band is seen at ≥ −68 dBm, leave below −75 dBm, using cached scan
results rather than forcing scans.

**Metered vs Low Data Mode.** `expensive` → metered (hotspot, cellular): amber,
data counter, speed test disabled, SSID remembered as a hotspot.
`constrained` → Low Data Mode: labelled as such, speed test off by default
with "Run anyway". Both → "Metered · Low Data Mode". Only `expensive` is ever
remembered as a hotspot.

**Home (inference).** Automatic, no pin buttons:
- *strong*: the network whose router credentials are stored;
- *weak*: the network with the most **recency-weighted** connected time (recent
  weeks dominate, so a new home or a finished office job wins out over time);
- a hidden right-click **Set as Home** on a row overrides both.

**Unexpected network change (observed).** Association moved away from Home and
this widget did not initiate it. The model records exactly that and nothing
about intent — joins made from the system Wi-Fi menu look the same. The Home
row turns **red only when Home confidence is strong**; with a weak guess it is
simply dimmed as "Not in range".

## 6. Menu structure (v3)

```
● Wi-Fi – Connected – excellent signal
┌─ connected card (graphite; state colour in glyph + meter) ─┐
│ [glyph]  Studio  (6 GHz)                           [QR]    │
│          Wi-Fi 6E · ch 37 · 160 MHz                        │
│          ▬▬▬▬▬▬▬▬ 41 dB · Excellent                        │
│   −52 signal      −93 noise      1201 Mbps                 │
│   Internet  ONLINE 18 ms                                   │
│   WAN       ↓3.2 ↑0.4 MB/s  sparkline      (router only)   │
│   Speed     ↓412 ↑38 Mbps · 2 h ago   Test                 │
│   IP / Gateway / MAC / Node            hover to copy       │
└────────────────────────────────────────────────────────────┘
Known Networks                                     [Refresh]
  ≈ Studio-IoT  (2.4 GHz)                 34 dB    [QR]
  ⛓ iPhone             hotspot · metered           [QR]
  ≈ Grind Coffee          Not in range             [QR]
Other Networks ▸
Style ▸ · Wi-Fi Settings… · Quit
```

The card leads with the diagnosis — name, band, SNR with a word verdict,
internet — and the engineering detail sits below it in the inset. Layout
constants (measured in the mockup): panel padding 7 px, card inset 9 px, icon
column 40 px on the card and 26 px in the list, 9 px right inset so every QR
button shares one axis, header dot 13 px from the top and left edges.

**QR codes open as a full-screen lightbox**, like macOS Large Type: the screen
dims, a ~300 px code sits on a dark panel, any click or key dismisses it.

**Without Location permission**: the card shows `Wi-Fi · 6 GHz` in place of the
name and hides the Node row; Known Networks **keeps its names** (they don't need
Location) but shows no in-range status, signal or band, since those come from
scans; one item asks for access.

## 7. AppKit specifics

- **Polling is required for a live meter.** Link-quality events proved sparse,
  so poll the interface every ~1 s while the menu is open and every ~5 s while
  it is closed; revisit only if events turn out to be reliable. Register the
  timers in the common run-loop modes so they keep firing during menu tracking
  (verified).
- **Scans are single-flight and cached.** On menu open, show cached results at
  once; start a background scan only if the cache is older than 60 s and no
  scan is running; merge results when it returns.
- **Custom views carry accessibility from the start**: every row, card fact and
  button sets its label and role when it is built, not in a later pass.
- **Submenus from view items** open with the keyboard (verified); mouse hover
  is still to be checked by hand.
- **Template vs colour**: the chip is a template image when monochrome, a
  coloured attributed title otherwise, as in `battery-widget`.

## 8. Risks and unknowns

- **Rejoin may not honour a chosen BSSID**, may race macOS auto-join, or may need
  credentials again. It gets a spike before any UI; if unreliable, the action
  is removed rather than shipped half-working.
- **Wi-Fi password retrieval** is the weak link of QR sharing: item fields,
  admin prompts, Touch ID, WPA3 and enterprise networks are all unknown.
  Spike first; if only the current network works cleanly, limit QR to it.
- **Instant Hotspot joins** may have no saved password at all.
- **GL.iNet RPC** is unverified beyond `challenge`; the WAN row stays optional
  everywhere and the feature is abandoned if the spike fails.
- **Home is a guess** until router credentials exist; the design keeps its
  alarm level proportional.
- **Portals that hijack DNS** are still detected, but without a redirect host.

## 9. Verification

There is no CI. `cargo test` covers the pure logic (tiers, hysteresis, headline
precedence, probe parsing, QR escaping, settings escaping, router parsing);
`--dump` is checked against `system_profiler SPAirPortDataType`; and a manual
pass walks every scenario: unplug the WAN, force 2.4GHz on the Flint 3, walk to
the far room, join a café portal, join the iPhone hotspot, turn on Low Data
Mode, deny Location, turn Wi-Fi off.
