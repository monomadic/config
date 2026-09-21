# wifi-widget tasks

The only build plan for the widget; [PROPOSAL.md](PROPOSAL.md) explains the
design and architecture, and this file wins wherever the two disagree.
The design to match is [designs/v3](designs/v3/wifi-widget-design.html).
Work top to bottom: each priority band assumes the one above it is done.
Items marked **spike** answer a question before code depends on the answer;
items marked **gated** wait for a named spike and are dropped if it fails.

Reference implementations in this repo: `src/battery-widget` (status item,
Style submenu, attributed-title bar) and `src/volume-control-widget` (custom
`NSView` menu rows, per-row controls, settings file, `--dump`).

---

## P0 — a widget that runs and tells the truth

Goal: a double-clickable app with a correct menu bar chip in every state and a
menu showing the connected network — built on the snapshot model from day one.
Nothing in P0 needs the keychain, the router or a network scan.

### 0.1 Completed spikes

Ran 2026-09-21 (macOS 26.5, arm64). Throwaway code is not kept in the repo.

- [x] **Location authorization.** SSID and BSSID are `nil` without it.
  - **Embedded `Info.plist` (`-sectcreate __TEXT __info_plist`) does not work.**
    CoreLocationAgent logs *"This process is not the executable of a bundle"*
    and *"client bundle is NULL. Skip showing AuthPrompt"*; no prompt appears.
  - **A plain binary cannot prompt either.**
  - **A generated `.app` (`LSUIElement`, `NSLocationWhenInUseUsageDescription`)
    shows the prompt** and receives `authorizedAlways`.
  - **It must be launched like a normal app.** Started by launchd pointing at
    `Contents/MacOS/<exe>`, SSID/BSSID stay `nil` even when authorized; started
    through LaunchServices (`open`, double-click, Login Items) they are present.
    So: a plain `.app`, "Open at Login" via `SMAppService.mainApp`, no LaunchAgent.
  - `ipconfig getsummary en0` redacts SSID and BSSID too; there is no
    permission-free source.
  - **Ad-hoc re-signing loses the grant**, so a rebuild re-prompts once.
    Accepted, as with other locally built apps here.
  - **`rssiValue` intermittently returns 0** on a live link, in every launch mode.
  - Consequence: this widget joins `job-monitor` as an exception to the
    no-bundle rule; `bundle.sh` generates the `.app`, nothing bundled is checked
    in. Update `AGENTS.md` when that lands.
- [x] **Bindings coverage.** All on the objc2 0.6 / framework-crate 0.3
  generation with no duplicate versions:
  - `objc2-core-wlan 0.3.2` (features `CWWiFiClient`, `CWInterface`, `CWChannel`,
    `CWConfiguration`, `CWNetwork`, `CWNetworkProfile`, `CoreWLANTypes`): reads
    work; `associate`/`disassociate` exist. `CWEventDelegate` works with
    `define_class!`; callbacks arrive **off the main thread**; the delegate
    property is **weak**, so keep a `Retained` delegate. `CWPHYMode` has no
    Wi-Fi 7 (`11be`) constant. **No link-quality events fired in a 30 s idle
    run** — live signal needs polling (0.5).
  - **`networkProfiles` SSIDs are readable without Location.**
  - **Scans without Location return `ssid = nil`**, and **a scan took 7.8 s,
    blocking its thread**.
  - Network framework: no usable crate; hand-declared `nw_path_monitor_*`,
    `nw_path_is_expensive`, `nw_path_is_constrained` with `block2` +
    `dispatch2 0.3` work. The path handle is only valid inside the callback.
  - `objc2-core-location 0.3.2`, `objc2-core-image 0.3.2` (QR: 31×31 output,
    needs nearest-neighbour scaling), `objc2-security 0.3.2` (compiles; never
    called).
- [x] **Live menu re-layout.** Growing a custom item's view with
  `NSMenu.itemChanged` resizes an open menu (88 → 208 px); timers in the common
  run-loop modes fire during menu tracking; a custom-view item opens its
  submenu on Right Arrow. **Mouse hover into that submenu is unverified** —
  synthetic events don't drive menu tracking; check by hand once.

### 0.2 Open spikes (run before the features they gate)

- [ ] **spike: targeted reassociation** — gates Rejoin (1.7) and row-click join
  (1.2). Needs Location and briefly drops Wi-Fi, so run when that's harmless.
  1. scan the current SSID and list its BSSIDs; note the current BSSID;
  2. `disassociate`, then `associate(to:password:)` with a *different* scanned
     `CWNetwork` of the same SSID (password `nil`, then from Keychain if refused);
  3. read back the BSSID after association settles;
  4. repeat 10×; also try once on the WPA3 network.
  Record: does macOS honour the chosen BSSID, does auto-join race us, does it
  ask for credentials or admin rights? **If it isn't reliable, Rejoin is
  removed from the design** rather than shipped as a button that can make
  things worse.
- [ ] **spike: Wi-Fi password retrieval** — gates QR codes (1.3). Needs an admin
  prompt you answer yourself.
  - Which keychain item and attributes hold a Wi-Fi password on macOS 26
    (service `AirPort`? account = SSID? System vs login keychain)?
  - `SecItemCopyMatching` vs `security find-generic-password -wa`: what prompt
    appears, does Touch ID satisfy it, does a second read in the same session
    re-prompt?
  - WPA3 network, a network whose SSID collides with another, an enterprise
    network if one is saved, and the iPhone hotspot (likely no item at all).
  If only the current network works cleanly, QR is limited to the connected
  card.
- [ ] **spike: gateway evidence** — shapes `GatewayEvidence` (0.8). Check
  whether the default route + the gateway's ARP entry (`sysctl` route/ARP
  tables) reliably distinguish "router present" from "no route", and whether
  macOS allows unprivileged ICMP echo via `socket(AF_INET, SOCK_DGRAM,
  IPPROTO_ICMP)`. TCP 80/53 results are kept only as metadata, never as the
  verdict.

### 0.3 Scaffold

- [ ] `Cargo.toml`: edition 2024; `objc2 0.6`, `objc2-foundation 0.3`,
  `objc2-app-kit 0.3`, `block2 0.6`, `dispatch2 0.3`, `objc2-core-wlan 0.3`;
  battery-widget's release profile (`opt-level = "s"`, `lto`, `strip`).
- [ ] `main.rs`: accessory activation policy, variable-length `NSStatusItem`,
  an `NSMenu` with Quit.
- [ ] `bundle.sh`: `cargo build --release`, wrap the binary into
  `target/release/WiFi Widget.app` with an `Info.plist` (`LSUIElement`,
  `NSLocationWhenInUseUsageDescription`, bundle ID `com.jayu.wifi-widget`),
  ad-hoc sign. The `.app` can live anywhere; copying it to `~/Applications` is
  the whole install.
- [ ] Request Location on first launch; "Open at Login" menu item via
  `SMAppService.mainApp.register()`, falling back to a Login Items hint.
- [ ] `README.md`: purpose, build, permissions, what works without Location.
- *Done when* double-clicking the `.app` shows a glyph with no Dock icon, the
  Location prompt appears once, Quit works, and `bash -n bundle.sh` passes.

### 0.4 Reading the interface (`wifi.rs`)

- [ ] Plain-Rust `LinkReading` from `CWWiFiClient.sharedWiFiClient.interface`:
  power, RSSI, noise, tx rate, channel number/band/width, PHY mode, SSID,
  BSSID, MAC, interface name.
- [ ] Pure conversions, unit-tested: band → `2.4GHz`/`5GHz`/`6GHz`; width →
  MHz; PHY + band → `Wi-Fi 4/5/6/6E/7` (ax on 6GHz is 6E; Wi-Fi 7 has no
  constant — treat unknown modes as "Wi-Fi" rather than guessing).
- [ ] Distinguish **off**, **disconnected**, **associated** and **redacted**
  (associated, non-zero RSSI, `ssid == nil`).
- *Done when* the values agree with `system_profiler SPAirPortDataType` for
  band, channel, width, PHY, RSSI, noise and rate.

### 0.5 Store and snapshot (`model.rs`)

The architecture the rest of the widget hangs on (PROPOSAL §4–5).

- [ ] `Sample<T> { value, at: Instant }` with `fresh` / `stale` / `unknown` by
  age (10–20 s for link values). `rssi == 0` is a dropped sample: keep the last
  good value but let it age out.
- [ ] Fact types: `LinkHealth`, `ProbeStatus`, `PathFlags { expensive,
  constrained }`, `BandStatus`, `GatewayEvidence`, plus inference types
  `HomeGuess { ssid, confidence }` and `NetworkChange`.
- [ ] `Store` owned by the main thread; every source posts results to it via
  the main queue; each update produces a new immutable `Snapshot` and sets a
  dirty flag; the UI redraws from the latest snapshot at most every 250 ms.
- [ ] `--dump` prints the current `Snapshot` (after one poll and one probe) as
  `key=value`, with each sample's age — no separate diagnostic path.
- [ ] Unit tests for merging and ageing, using an injectable clock.

### 0.6 Polling and events

- [ ] `CWEventDelegate` (link, SSID, BSSID, link quality, power) as *triggers*
  for an immediate read; they are not the only source of freshness.
- [ ] Poll the interface every ~1 s while the menu is open and every ~5 s while
  it is closed (link-quality events proved sparse). Timers registered in the
  common run-loop modes.
- [ ] After wake and after association changes, poll at 1 s for 30 s.

### 0.7 Signal model (pure)

- [ ] `snr = rssi − noise`. **If noise is unavailable, show RSSI in dBm and
  derive segments from a separate RSSI tier table — never display an invented
  SNR.**
- [ ] SNR tiers `<15` poor, `15–25` fair, `25–40` good, `≥40` excellent → 1–4
  segments; meter 0–50 dB; word verdict for the card (`Excellent`, `Good`, …).
- [ ] Hysteresis: 3 dB margins at tier edges, 10 s hold before the menu bar
  changes; escalations faster than recoveries.
- [ ] Unit tests: tier edges, margins, hold timer, noise-missing path, stale
  samples.

### 0.8 Headline state (pure)

- [ ] `headline_state(&Snapshot) -> State` over `WifiOff, Disconnected,
  NoInternet, LoginRequired, Weak, MeteredFallback, BandFallback, Healthy`
  with the precedence in PROPOSAL §5. It reads facts; it never discards them.
- [ ] Table-driven tests: one case per mockup scenario plus collisions
  (weak + no internet + metered + 2.4GHz → `NoInternet`, with every fact still
  present in the snapshot).
- [ ] **design gap:** Wi-Fi off, disconnected and Location-denied have no
  mockup. Add them to `designs/v3` before building their UI.

### 0.9 Menu bar chip (`bar.rs`) — Smart Bar and Icon only

- [ ] SF Symbols via `NSImage(systemSymbolName:)`: `wifi` (with
  `variableValue` for lit arcs), `wifi.exclamationmark`, `personalhotspot`,
  `wifi.slash`, at a small symbol size matching the battery widget's bolt.
- [ ] Four segments drawn into an `NSImage`; lit count from the tier, colour
  from the state; template image when monochrome.
- [ ] **Smart Bar** (default) with tags `6GHz`, `2.4GHz`, `−81 dB` (U+2212),
  `Login`, data used; no segments in `NoInternet`. **Icon**: always white,
  exclamation glyph when not healthy.
- [ ] Style submenu (two entries for now), persisted.
- [ ] `--state <name>` debug flag for screenshots.
- *Done when* both styles match the mockup in every state on light and dark
  menu bars. The other three styles are P1 (1.8).

### 0.10 Probe (`probe.rs`)

- [ ] Background-thread HTTP/1.1 `GET /hotspot-detect.html` to
  `captive.apple.com:80` over a plain `TcpStream`, 5 s timeouts, no HTTP crate.
- [ ] Result is a `ProbeStatus`: `Reachable { latency }`, `Captive { host }`
  (3xx, or a 200 without `Success`), `DnsFailure`, `ConnectFailure`,
  `ReadFailure`, `UnexpectedResponse`. The UI collapses these; the snapshot
  and `--dump` keep them.
- [ ] Gateway: fill `GatewayEvidence` per the 0.2 spike (route, ARP, maybe
  ICMP); any TCP 80/53 attempt is recorded as `service_responded`, never as
  reachability.
- [ ] Schedule: every 30 s, 2 s after association changes, on Refresh.
  Two consecutive failures before `NoInternet`.
- [ ] Open Login Page: `NSWorkspace.openURL("http://captive.apple.com")`.
- [ ] Unit tests on recorded fixtures (success, redirect, hijacked body,
  truncated body, DNS failure, timeout).

### 0.11 Menu skeleton

- [ ] One-line header (state dot + `Wi-Fi – <status>`), dot 13 px from the
  panel's top and left edges.
- [ ] Connected card (`card.rs`), static layout: 40 px glyph column, name + band
  pill, spec line, SNR meter with notches and word verdict, QR button top-right.
- [ ] `Style ▸`, `Wi-Fi Settings…` (the Wi-Fi pane's `x-apple.systempreferences:`
  URL), `Quit`.
- [ ] **Accessibility is part of the view types from the start**: every custom
  view sets its accessibility label and role when built (card facts, buttons,
  rows); arrow keys must still move through the menu.
- *Done when* the menu opens instantly, matches the v3 card at rest, values
  stay live while it is open, and VoiceOver reads the header and card.

---

## P1 — the parts that make it worth opening

### 1.1 Connected card, complete

- [ ] State colour in glyph and meter only (graphite card); amber band pill in
  `BandFallback`; glyph badge for `NoInternet` / `Login`.
- [ ] Link line: signal, noise, rate in three equal, centred columns.
- [ ] Facts: Internet, WAN and Speed (the last two hidden until P2), IP,
  Gateway, MAC (`PRIVATE` when bit `0x02` of the first octet is set), Node.
- [ ] Hover-to-copy: `NSTrackingArea` per row, highlight + copy glyph, write to
  `NSPasteboard`, `Copied` for 1.1 s.
- [ ] IP and gateway from `getifaddrs` + the routing table or
  `State:/Network/Global/IPv4`.

### 1.2 Known Networks (`row.rs`)

- [ ] Names from `networkProfiles` (macOS's order, minus the connected one) —
  **always shown, with or without Location.**
- [ ] With Location: enrich from scans — in-range, signal, band. Without it:
  no enrichment and a single "Allow Location for signal and availability" line;
  never hide the names.
- [ ] Scanning: on menu open, show the cache at once; start a background scan
  only if the cache is older than 60 s **and no scan is running** (single
  flight); merge when it returns (~8 s). Refresh forces a scan unless one is
  running.
- [ ] Row: glyph with lit arcs, name, band pill, `34 dB` (or dBm if noise is
  unavailable) or `Not in range`, QR button; 30 px line, 4 px left inset,
  26 px icon column, 9 px right inset.
- [ ] Ordering: in range by signal, then not in range; cap at 8 rows.
- [ ] Row click joins that network — **gated** on the reassociation spike;
  otherwise it opens Wi-Fi Settings.
- [ ] Refresh button in the section header, `Scanning…` while busy; also
  reruns the probe.

### 1.3 QR codes (`qr.rs`) — gated on the password spike

- [ ] Payload `WIFI:T:WPA;S:<ssid>;P:<pass>;;`, `T:nopass` for open networks;
  escape `\ ; , : "`; unit tests.
- [ ] `CIQRCodeGenerator` (`inputCorrectionLevel = "M"`), nearest-neighbour
  scaling, white quiet zone.
- [ ] Password via whatever the spike proves; held in memory for the menu
  session only, never logged, never in `--dump`, never in settings. No saved
  password (e.g. Instant Hotspot): the lightbox says so instead of a code.
- [ ] **Full-screen lightbox** like macOS Large Type: borderless `NSPanel` over
  the whole screen under the pointer, dimmed backdrop (~50 % black), dark
  rounded panel with the ~300 px code, network name and security type. Any
  click, any key or losing focus dismisses it; 0.15 s fade unless Reduce Motion.
  Level above the menu bar, `[.canJoinAllSpaces, .fullScreenAuxiliary]`; the
  menu closes before it appears; activate the app so keys reach it.
- *Done when* a phone joins Studio and the café network from the codes.

### 1.4 Path flags (`path.rs`)

- [ ] `nw_path_monitor` on a private queue → `PathFlags { expensive,
  constrained }`. **Two facts, never merged.**
- [ ] `expensive` → `MeteredFallback`: amber, data counter, speed test off, SSID
  remembered as a hotspot (dashed row before you join it next time).
- [ ] `constrained` alone → "Low Data Mode" label; speed test off by default
  with "Run anyway"; **not** remembered as a hotspot.
- [ ] Both → "Metered · Low Data Mode".

### 1.5 Hotspot data counter

- [ ] 64-bit byte counters from `sysctl` `NET_RT_IFLIST2` (`if_msghdr2`); the
  32-bit `getifaddrs` counters wrap at 4 GB.
- [ ] Baseline at association; `212 MB` / `1.4 GB` in the card and Smart Bar
  tag while metered.

### 1.6 Home and network changes (inferences)

- [ ] Recency-weighted connected-time tally per SSID (e.g. exponential decay,
  ~30-day half-life), flushed every 60 s.
- [ ] `HomeGuess`: `strong` = router credentials stored for that SSID (P2);
  `weak` = highest weighted tally; a hidden right-click **Set as Home** on a row
  overrides both.
- [ ] `NetworkChange::UnexpectedNetworkChange { from, to }` when association
  moves away from Home without this widget initiating it. The model records
  only that — no claim about intent.
- [ ] Home row red only for an unexpected change **and** a strong guess;
  otherwise dimmed "Not in range".

### 1.7 Band fallback and Rejoin

- [ ] `BandFallback` enter when the same SSID is seen on 5/6GHz at ≥ −68 dBm,
  leave below −75 dBm, from cached scans (no extra scanning).
- [ ] **Rejoin — gated** on the reassociation spike, for `BandFallback` and for
  `Weak` with a stronger BSSID of the same SSID visible. If the spike fails,
  delete Rejoin from the design and the mockup.

### 1.8 Remaining menu bar styles

- [ ] Icon + dBm (`−52 dB`), Bar + Rate, Bar + Band, on the same `bar.rs`
  primitives. *Done when* they match the mockup in every state.

---

## P2 — seeing past the router

### 2.1 GL.iNet router (`router.rs`) — isolated and optional

- [ ] **spike: authenticate and map the RPC** (needs the admin password, which
  you enter): `challenge` → login hash → `login` → session; find the calls for
  WAN status, uptime, uplink and byte counters. If they don't exist, stop and
  keep WAN hidden.
- [ ] Credentials in the **login** keychain (service `wifi-widget.router`);
  entry via a `Connect Router…` item.
- [ ] Poll every 5 s with the menu open, 30 s otherwise; 60-sample ring buffer;
  throughput from byte deltas.
- [ ] WAN row with sparkline; `DOWN · 4 min` when the uplink is down, feeding
  `NoInternet – WAN down`.
- [ ] Session expiry and bad password: back off and show nothing, never stale
  data.
- [ ] No other vendors unless router monitoring becomes the product.

### 2.2 Speed test (`speed.rs`)

- [ ] `/usr/bin/networkQuality -c` on click only; parse throughput and
  responsiveness; `Testing…` (~15 s); cancel on menu close or quit.
- [ ] Last result per SSID with its time.
- [ ] Disabled when `expensive`; off by default with "Run anyway" when only
  `constrained`.

### 2.3 Accessibility pass

- [ ] Full VoiceOver pass over every state; Return activates rows; the
  lightbox is announced and dismissable from the keyboard.

---

## P3 — deferred on purpose

- [ ] `Other Networks ▸` submenu for scanned SSIDs not in the known list.
- [ ] Fix or delete `scripts/wifi-qr` (relies on the removed `airport` CLI).
- [ ] macOS 27: `NSStatusItem` expanded interface session + `NSGlassEffectView`
  for `designs/alt-black-panel`; if pursued, as a shared crate for all widgets.
- Dropped: UPnP IGD and NAT-PMP router telemetry (no reply here, inconsistent
  elsewhere, and weak data would undermine the widget's accuracy).

---

## Cross-cutting

- **Truthfulness:** facts and inferences are separate types; inferences carry a
  confidence; units always match what was measured.
- **Performance:** ~0 % CPU idle and < 25 MB resident; the 5 s closed-menu poll
  is the only idle work besides the 30 s probe. Never scan on a timer.
- **Secrets:** Wi-Fi and router passwords stay in the keychain and in memory for
  the menu session only; audit logs and `--dump` for leaks.
- **Settings** at `~/.config/wifi-widget/settings`, atomic writes. **SSIDs are
  values, never keys**, and are escaped (`=`, newlines, non-ASCII, very long
  names): e.g. `home=<escaped>`, `hotspot=<escaped>`, `tally=<escaped>:<seconds>`.
  Tests for round-tripping awkward SSIDs.
- **Verification** (no CI): `cargo test` for the pure modules; `--dump` against
  `system_profiler`; `bash -n bundle.sh`; a manual pass through every scenario —
  unplug the WAN, force 2.4GHz on the Flint 3, walk to the far room, join a
  café portal, join the iPhone hotspot, turn on Low Data Mode, deny Location,
  turn Wi-Fi off.
