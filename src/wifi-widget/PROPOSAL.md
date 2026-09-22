# wifi-widget: macOS menu bar Wi-Fi status widget

Status: **native v3 panel, nearby scans, cached-first results, security labels, copyable addresses and click-to-join implemented. QR, session totals and router telemetry remain future work.**
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
| `designs/v3/` | the design being built: current graphite panel, Nearby Networks, scan interactions, six scenarios; future ideas described separately |
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
| path gateway + ARP + ICMP | gateway address, MAC resolved, echo RTT | verified; unprivileged ICMP works (`SOCK_DGRAM`/`IPPROTO_ICMP`) |
| `captive.apple.com/hotspot-detect.html` | whether one well-known HTTP endpoint answers as expected | evidence of reachability, not proof "the internet works" |
| GL.iNet JSON-RPC at `/rpc` | WAN up/down, uptime, uplink, byte counters | only the unauthenticated `challenge` is verified; method names unknown |
| `/usr/bin/networkQuality` | capacity and responsiveness | ~15 s and hundreds of MB; on demand only |
| Keychain | Wi-Fi password for the QR payload | System keychain, service `AirPort`, account = SSID; admin prompt per read; WPA3 verified, iPhone hotspot has an item |
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
    gateway: GatewayEvidence,  // gateway from the path, ARP resolved, ICMP echo RTT
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
| `BandFallback` | on 2.4GHz and a faster band of the same SSID seen (hysteresis below) | alert glyph + segments + `2.4GHz` | — |
| `Weak` | SNR tier poor | alert glyph + segments + `−81 dB` | — |
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

## 6. Menu structure (v3 — current approved panel)

```text
● Connected                      (dot with translucent halo)
┌─ graphite #17181b · thin #46474b outline ─────────────┐
│ [Wi-Fi] Studio (6 GHz)                                │
│         Wi-Fi 6E · ch 37 · 160 MHz                    │
│         ▬▬▬▬▬▬▬▬▬▬▬▬▬   41 dB · Excellent            │
│         0   15   25   40 50                          │
│   −52             −93             1201              │
│ signal · dBm    noise · dBm        Mbps              │
│ ─────────────────────────────────────────────────── │
│ Internet  ONLINE  18 ms                             │
│ ─────────────────────────────────────────────────── │
│ IP        192.168.1.24                              │
│ Gateway   192.168.1.1                               │
│ MAC       f6:1d:83:5a:c0:27  PRIVATE                 │
│ Node      3c:22:fb:9e:41:a0                          │
└─────────────────────────────────────────────────────┘
Nearby Networks                                  ↻
  ≈ Studio-IoT                        2.4 GHz   ▂▄▆█
                                      WPA2
  ⛓ iPhone                            5 GHz     ▂▄▆
                                      WPA2/3
Style ▸ · Open at Login · Wi-Fi Settings… · Quit
```

The heading contains status only. The 8 pt dot is inset slightly and surrounded
by a 14 pt translucent halo. No separator appears above Nearby Networks.
Graphite inset: #1e1f22. SSID and detail lines are compact. Band pills hug measured
text, with modest horizontal padding and optical vertical centering. Nearby-row
pill outlines have a fixed 14 pt height; their text is optically lowered 1.5 pt
inside the outline rather than moving the text and outline together.

The menu is 320 pt wide (reduced from 340). The SNR meter is 220 × 10 pt with marks at 15, 25 and 40 dB and labels at
0, 15, 25, 40 and 50. It ends 18 pt before the right inset, with its verdict above and right-aligned.
The full track uses red/amber/blue/green tier segments and a white 18 pt reading
marker with a dark outline. Color only the quality word by its tier. The SSID
is bold; the Wi-Fi glyph centers vertically in the upper graphite section. Hide
this scale for RSSI-only readings. The card has extra top/bottom padding. The three stats have 6 pt more padding
above and below and use bold 16 pt figures above
unchanged 9 pt labels, centered in equal columns. Internet is vertically
centered; only ONLINE is green and latency stays neutral. IP, Gateway, MAC and
Node share fixed label/value columns and copy on click. Hover reveals a separate
copy icon without moving the text. IP/Gateway come from the Wi-Fi service's IPv4
configuration, never an unrelated VPN or Ethernet service.

Nearby Networks contains visible scan results, including unsaved networks,
strongest first. Exclude the current SSID; combine duplicate SSIDs using their
strongest access point. Saved, in-range names and icons are pure white; unsaved
ones use secondary color. Indented rows highlight on hover. Each shows small grey band text immediately after the SSID,
with advertised security below the strength bars. Security includes mixed/enterprise/Open/Unknown modes.
An iPhone name gives a hotspot icon as an explicitly name-based hint.

Clicking a row attempts a background association. Failure offers a secure
password retry; enterprise authentication requires Wi-Fi Settings. The mockup
simulates this without changing network configuration.

Refresh is a 14 pt symbol centered in a 24 pt hover circle. At rest it has no
border or fill. Hover makes the symbol white and adds a translucent background
and thin outline. While scanning, replace it with an animated spinner and
prevent further clicks. Start a scan at launch; menu openings rescan only when
results are more than three minutes old. Manual Refresh bypasses the age check,
never the single-flight guard. macOS cached results may populate an empty list
first; identify them in tooltips and replace them with the full scan result.
Do not use saved profiles as evidence of visibility when Location prevents scans.

**Still planned:** observed session duration and received bytes after latency
(`ONLINE 13 ms · 18 hrs · 4.3 GB`), QR sharing, router telemetry and speed tests.
Session totals must reset with association changes and clearly include LAN
traffic; these are not implemented or shown as live facts in the current mockup.

## 7. AppKit specifics

- **Polling is required for a live meter.** Link-quality events proved sparse,
  so poll the interface every ~1 s while the menu is open and every ~5 s while
  it is closed; revisit only if events turn out to be reliable. Register the
  timers in the common run-loop modes so they keep firing during menu tracking
  (verified).
- **Scans are single-flight and cached.** On menu open, show cached results at
  once; start a background scan only if the cache is older than three minutes and no
  scan is running; merge results when it returns.
- **Custom views carry accessibility from the start**: every row, card fact and
  button sets its label and role when it is built, not in a later pass.
- **Submenus from view items** open with the keyboard (verified); mouse hover
  is still to be checked by hand.
- **Template vs colour**: the chip is a template image when monochrome, a
  coloured attributed title otherwise, as in `battery-widget`.

## 8. Risks and unknowns

- **The widget never joins networks.** A spike showed `associate` failing every
  time and `disassociate` leaving the Mac offline for minutes, so Rejoin and
  join-on-click were removed; the widget diagnoses and macOS joins.
- **Every QR reveal costs an admin prompt.** Wi-Fi passwords are in the System
  keychain; `SecItemCopyMatching` returns them only after an administrator
  prompt, and a second read prompted again. Enterprise networks are untested.
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
