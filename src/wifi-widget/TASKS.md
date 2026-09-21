# wifi-widget tasks

Build checklist for the [technical proposal](PROPOSAL.md).
The design to match is [designs/v3](designs/v3/wifi-widget-design.html).
Work top to bottom: each priority band assumes the one above it is done.
Items marked **spike** answer a question before code depends on the answer.

Reference implementations in this repo: `src/battery-widget` (status item,
Style submenu, attributed-title bar, installer) and `src/volume-control-widget`
(custom `NSView` menu rows, per-row controls, settings file, `--dump`).

---

## P0 — a widget that runs and tells the truth

Goal: a double-clickable app, event-driven, correct menu bar chip in all six states, and a
menu that shows the connected network. Nothing in P0 needs the router, the
keychain or the network scan.

### 0.1 Spikes that can change the plan (do these first)

All three ran on 2026-09-21 (macOS 26.5, arm64). Throwaway code is not kept in
the repo; the results are recorded here.

- [x] **spike: Location authorization.** SSID and BSSID are `nil` without it,
  which disables Known Networks matching, Home detection and the Node row.
  Results:
  - **Embedded `Info.plist` (`-sectcreate __TEXT __info_plist`) does not work.**
    `locationd` forwards the request, but CoreLocationAgent logs *"This process
    is not the executable of a bundle"* and *"client bundle is NULL. Skip
    showing AuthPrompt"*. No prompt is ever shown.
  - **A plain binary cannot prompt either**: no request reaches CoreLocationAgent.
  - **A generated `.app` (with `LSUIElement` and `NSLocationWhenInUseUsageDescription`)
    shows the prompt** and receives `authorizedAlways`.
  - **It must be launched like a normal app.** Authorized and started by launchd
    pointing at `Contents/MacOS/<exe>`, SSID/BSSID stay `nil`; started through
    LaunchServices (`open`, double-click, Login Items) they are present. So there
    is **no LaunchAgent**: the widget is a plain `.app` you open, and it offers
    "Open at Login" via `SMAppService.mainApp` (unverified for an unsandboxed,
    self-signed app — fall back to asking the user to add it in Login Items).
    The only thing launchd added was restart-on-crash; that is given up.
  - `ipconfig getsummary en0` also redacts SSID and BSSID, so there is no
    permission-free way to read them.
  - **Ad-hoc re-signing loses the grant.** A rebuild changed the cdhash and the
    status reverted to `notDetermined`, so each rebuild re-prompts once.
    Accepted: that's how other locally built apps here behave. A stable
    self-signed identity would avoid it if it ever becomes annoying.
  - **`rssiValue` intermittently returns 0** for single reads in every launch
    mode. Treat 0 as "no sample" and keep the previous value (see 0.5).
  - Consequence: this widget joins `job-monitor` as an exception to the no-bundle
    rule. A build script wraps the release binary into a `.app`; nothing bundled
    is checked in. Update `AGENTS.md` when that lands.
- [x] **spike: bindings coverage.** Everything needed exists on the
  objc2 0.6 / framework-crate 0.3 generation, with no duplicate versions:
  - `objc2-core-wlan 0.3.2` (features `CWWiFiClient`, `CWInterface`, `CWChannel`,
    `CWConfiguration`, `CWNetwork`, `CWNetworkProfile`, `CoreWLANTypes`): all
    reads work; `associate`/`disassociate` exist. `CWEventDelegate` implements
    fine with `define_class!`; callbacks arrive **off the main thread**, and the
    delegate property is **weak**, so keep a `Retained` delegate alive.
    `CWPHYMode` stops at `11ax` — there is no Wi-Fi 7 (`11be`) constant.
  - **`networkProfiles` SSIDs are readable without Location permission**, so the
    Known Networks names work even when redacted.
  - **Scans without Location return `ssid = nil`** for every network (RSSI and
    channel are filled), and **a scan took 7.8 s, blocking its thread** — not
    the 1–3 s assumed in 1.2.
  - Network framework: no usable crate (`objc2-network` is a 0.0.0 placeholder).
    Hand-declared `nw_path_monitor_*`, `nw_path_is_expensive`,
    `nw_path_is_constrained` with `block2` + `dispatch2 0.3` work; the path
    handle is only valid inside the callback.
  - `objc2-core-location 0.3.2`, `objc2-core-image 0.3.2` (QR generator verified:
    31×31 output, needs nearest-neighbour scaling), `objc2-security 0.3.2`
    (`SecItemCopyMatching` compiles; never called).
- [x] **spike: live menu re-layout.** Both answers are yes, for keyboard:
  - Growing a custom item's view from 30 to 150 px while the menu was open,
    then calling `NSMenu.itemChanged`, grew the menu window from 88 to 208 px
    live. No longer needed for QR codes, which now open in their own window
    (1.4), but the menu can grow rows if something else needs it.
  - Timers registered in `.common` run-loop modes fired during menu tracking.
  - A custom-view item with a submenu opened that submenu on Right Arrow, so
    `Other Networks ▸` (3.1) can be a view item. **Mouse hover is unverified**:
    synthetic mouse-moved events don't drive menu tracking. Check by hand once.

### 0.2 Scaffold

- [ ] `Cargo.toml`: edition 2024; `objc2 0.6`, `objc2-foundation 0.3`,
  `objc2-app-kit 0.3`, `block2 0.6`, plus CoreWLAN bindings per 0.1. Copy
  battery-widget's release profile (`opt-level = "s"`, `lto`, `strip`).
- [ ] `main.rs`: accessory activation policy (no Dock icon), variable-length
  `NSStatusItem`, an `NSMenu` with Quit.
- [ ] `bundle.sh` in the crate: `cargo build --release`, then wrap the binary
  into `target/release/WiFi Widget.app` with an `Info.plist` (`LSUIElement`,
  `NSLocationWhenInUseUsageDescription`, bundle ID `com.jayu.wifi-widget`) and
  ad-hoc sign it (a rebuild re-prompts for Location once; accepted). The `.app` can live anywhere; copying it to `~/Applications` is the
  whole "install". No LaunchAgent (see the Location spike).
- [ ] "Open at Login" menu item via `SMAppService.mainApp.register()`; if that
  refuses a self-signed app, show a hint to add it in Login Items instead.
- [ ] `README.md`: purpose, styles, data sources, build, permissions.
- *Done when* double-clicking the built `.app` shows the glyph with no Dock
  icon, Quit works, and `bash -n` passes on `bundle.sh`.

### 0.3 Reading the interface (`wifi.rs`)

- [ ] Plain-Rust `Link` snapshot from `CWWiFiClient.sharedWiFiClient.interface`:
  `powerOn`, `rssiValue`, `noiseMeasurement`, `transmitRate`,
  `wlanChannel` (`channelNumber`, `channelBand`, `channelWidth`),
  `activePHYMode`, `ssid`, `bssid`, `hardwareAddress`, `interfaceName`.
- [ ] Pure conversions, unit-tested: band enum → `2.4GHz`/`5GHz`/`6GHz`;
  width enum → MHz; PHY mode + band → `Wi-Fi 4/5/6/6E/7` (ax on 6GHz is 6E).
- [ ] Distinguish **off**, **disconnected**, **associated** and **redacted**
  (associated with a non-zero RSSI but `ssid == nil`).
- [ ] `--dump`: print every field plus derived values as `key=value`.
- *Done when* `--dump` agrees with `system_profiler SPAirPortDataType` for band,
  channel, width, PHY, RSSI, noise and rate on this Mac.

### 0.4 Events, not polling

- [ ] `CWEventDelegate` class via `define_class!`; start monitoring
  `linkDidChange`, `ssidDidChange`, `bssidDidChange`, `linkQualityDidChange`,
  `powerDidChange`.
- [ ] Callbacks arrive off the main thread: set an atomic dirty flag and hop to
  the main queue. Coalesce redraws to at most one per 250 ms.
- [ ] Safety-net poll every 5 s in case an event source goes quiet (for
  example after sleep/wake).
- [ ] Register the redraw timer in `NSRunLoopCommonModes`, not just the default
  mode — otherwise it stops firing while the menu is open.

### 0.5 Signal model (pure, no AppKit)

- [ ] `snr = rssi − noise`; if noise is 0/unknown, fall back to an RSSI-only
  mapping and mark the value as estimated.
- [ ] Treat `rssi == 0` as a dropped sample and keep the last good value — the
  spike saw CoreWLAN return 0 intermittently on an associated link.
- [ ] Tiers: `<15` poor, `15–25` fair, `25–40` good, `≥40` excellent →
  1–4 segments. Meter scale 0–50 dB.
- [ ] Hysteresis: 3 dB margin around each tier edge, and a new tier must hold
  10 s before the menu bar changes. Recoveries are slower than escalations.
- [ ] Unit tests: tier edges, margin behaviour, hold timer, noise-unknown path.

### 0.6 State classification (pure, no AppKit)

- [ ] `enum State { Healthy, BandFallback, Weak, NoInternet, LoginRequired,
  MeteredFallback, Disconnected, WifiOff }`.
- [ ] `classify(inputs) -> State` with an explicit precedence:
  `WifiOff > Disconnected > NoInternet > LoginRequired > Weak >
  MeteredFallback > BandFallback > Healthy`.
- [ ] Table-driven tests: one case per mockup scenario, plus the precedence
  collisions (e.g. weak *and* metered).
- [ ] **design gap:** Wi-Fi off, disconnected and Location-denied have no
  mockup. Add them to `designs/v3` (slashed glyph, `Wi-Fi – Off` /
  `Not connected` header, no card) before building their UI.

### 0.7 Menu bar chip (`bar.rs`)

- [ ] Glyphs from SF Symbols via `NSImage(systemSymbolName:)` rather than
  hand-drawn paths: `wifi` (with `variableValue` for lit arcs),
  `wifi.exclamationmark`, `personalhotspot`, `wifi.slash`. Small symbol
  configuration so the glyph matches the battery widget's bolt.
- [ ] Four segments drawn into an `NSImage` with a drawing handler; lit count
  from the tier, colour from the state.
- [ ] Tag text exactly as designed: `6GHz`, `2.4GHz`, `−81 dB` (U+2212),
  `Login`, data used (`212 MB`, `1.4 GB`). No segments in `NoInternet`.
- [ ] Five styles: Smart Bar (default), Icon (always white, exclamation glyph
  when not healthy), Icon + dBm (`−52 dB`), Bar + Rate, Bar + Band.
- [ ] Template image when monochrome so it adapts to light and dark menu bars;
  coloured attributed title otherwise.
- [ ] Style submenu with a checkmark, persisted in `settings.rs`.
- [ ] Debug flag `--state <name>` forcing a state, for screenshots.
- *Done when* every style × state pair matches the mockup, on both a light and
  a dark menu bar.

### 0.8 Internet check (`probe.rs`)

- [ ] Background-thread HTTP/1.1 `GET /hotspot-detect.html` to
  `captive.apple.com:80` over a plain `TcpStream` with 5 s connect/read
  timeouts. No HTTP client crate — keep the binary small.
- [ ] Classify: body contains `Success` → online; 3xx → `LoginRequired` with the
  `Location` host; other 200 body → `LoginRequired` without a host;
  DNS/connect/read failure → offline.
- [ ] Gateway check: TCP connect to the default gateway on port 80 (then 53) to
  separate "router answers, internet doesn't" from "no route at all". No ICMP —
  it needs privileges.
- [ ] Record round-trip time for the `ONLINE 18 ms` row.
- [ ] Schedule: every 30 s, 2 s after any association change, and on Refresh.
  Require two consecutive failures before `NoInternet`.
- [ ] Open Login Page: `NSWorkspace.openURL("http://captive.apple.com")` so the
  portal's redirect happens in the default browser.
- [ ] Unit tests on recorded response fixtures (success, redirect, hijacked
  body, truncated body, timeout).

### 0.9 Menu skeleton

- [ ] `NSMenuDelegate.menuWillOpen`: kick a probe; later also a scan (1.2).
- [ ] Header item (custom view): state dot + `Wi-Fi – <status>` on one line;
  dot inset 13 px from the panel's top and left edges, as measured in v3.
- [ ] Connected card (`card.rs`), static layout first: 40 px glyph column,
  name + band pill, spec line, SNR meter with tier notches, QR button top-right.
- [ ] Items: `Style ▸`, `Wi-Fi Settings…` (opens the Wi-Fi pane via its
  `x-apple.systempreferences:` URL), `Quit`.
- *Done when* the menu opens instantly, matches the v3 card at rest, and live
  values update while it stays open.

---

## P1 — the parts that make it worth opening

### 1.1 Connected card, complete

- [ ] State colour in the glyph and meter only (graphite card); band pill
  filled amber in `BandFallback`; badge on the glyph for `NoInternet` / `Login`.
- [ ] Link line: signal, noise and rate in three equal, centred columns.
- [ ] Fact rows: Internet, WAN, Speed (the latter two hidden until P2), IP,
  Gateway, MAC, Node.
- [ ] Hover-to-copy: `NSTrackingArea` per copyable row, highlight + copy glyph
  on hover, `NSPasteboard` write on click, `Copied` for 1.1 s.
- [ ] MAC tagged `PRIVATE` when bit `0x02` of the first octet is set.
- [ ] IP and gateway from `getifaddrs` + the routing table (`sysctl`
  `NET_RT_DUMP` or `SCDynamicStore` `State:/Network/Global/IPv4`).

### 1.2 Known Networks (`row.rs`)

- [ ] Source: `CWConfiguration.networkProfiles` (saved networks, in macOS's
  order), minus the connected one.
- [ ] Scan with `scanForNetworks(withSSID: nil)` on a background thread, **only**
  on menu open and Refresh; cache results for 60 s. A scan measured 7.8 s, so
  the menu must open with cached results and fill in when the scan returns.
- [ ] Row: glyph with lit arcs from scanned SNR, name, band pill, right-hand
  `34 dB` or `Not in range`, QR button. One compact 30 px line; 4 px left
  inset, 26 px icon column, 9 px right inset so QR buttons share the card's axis.
- [ ] Ordering: in range first (by SNR), then not in range; cap at 8 rows.
- [ ] Dashed border for remembered metered SSIDs; red row for Home after an
  involuntary drop (1.6).
- [ ] Click a row: associate via `CWInterface.associate(to:password:)` with the
  scanned `CWNetwork`; on error, open Wi-Fi Settings.
- [ ] Refresh button in the section header (bordered, dark fill, same
  treatment as the QR buttons), showing `Scanning…` while busy; also reruns
  the probe.

### 1.3 Hotspot data counter

- [ ] Per-interface byte counters from `sysctl` `NET_RT_IFLIST2` (`if_msghdr2`,
  64-bit `ifi_ibytes`/`ifi_obytes`). The 32-bit `getifaddrs` counters wrap at
  4 GB and would corrupt long sessions.
- [ ] Baseline at association; show the delta as `212 MB` / `1.4 GB` in the
  card and the Smart Bar tag while metered.

### 1.4 QR codes (`qr.rs`)

- [ ] Payload builder: `WIFI:T:WPA;S:<ssid>;P:<pass>;;`, `T:nopass` for open
  networks; escape `\ ; , : "`. Unit tests for escaping and open networks.
- [ ] Image: `CIFilter` `CIQRCodeGenerator` (`inputMessage` UTF-8 data,
  `inputCorrectionLevel = "M"`), scaled with nearest-neighbour, rendered into
  an `NSImage` on a white quiet zone.
- [ ] Password: `SecItemCopyMatching` for the AirPort item (System keychain);
  fall back to `/usr/bin/security find-generic-password -wa <ssid>`. Expect an
  admin prompt. Hold the password in memory for the menu session only — never
  log it, never write it to settings.
- [ ] No saved password (likely for Instant Hotspot joins): panel says so
  instead of showing a code.
- [ ] Show the code as a **full-screen lightbox**, the way macOS Large Type
  and 1Password's "Show in Large Type" work: a borderless window covering the
  screen under the pointer, dimmed backdrop (black ~50 %), and a dark rounded
  panel in the middle with the code at ~300 px, the network name and the
  security type. Any click or any key dismisses it; fade 0.15 s unless Reduce
  Motion is on.
  - Window: `NSPanel`, borderless, `.nonactivatingPanel` off, level above the
    menu bar (`.screenSaver` or `.popUpMenu`), `collectionBehavior`
    `[.canJoinAllSpaces, .fullScreenAuxiliary]`, frame = that screen's full frame.
  - Clicking the QR button closes the menu first, then shows the lightbox;
    activate the app so key presses reach it, and close on `resignKey` too.
- *Done when* a phone joins Studio and the café network from the codes.

### 1.5 Metered detection

- [ ] `nw_path_monitor` on a private dispatch queue; read
  `nw_path_is_expensive` and `nw_path_is_constrained` for the Wi-Fi path.
- [ ] Remember flagged SSIDs (`metered=` in settings) so a known hotspot is
  dashed before you join it.
- [ ] While metered: amber state, data counter visible, speed test disabled.

### 1.6 Home detection

- [ ] Tally connected seconds per SSID (`seconds.<ssid>=` in settings, flushed
  every 60 s).
- [ ] Home = SSID with stored router credentials (P2), else the highest tally.
- [ ] Involuntary drop: Home was connected, the link dropped, and a different
  SSID associated within 60 s without a join initiated from this menu. Mark
  Home red until it is back or the user joins something themselves.
- [ ] Known limitation, document in README: joins made from the system Wi-Fi
  menu look involuntary to the widget.

### 1.7 Band fallback and Rejoin

- [ ] `BandFallback` only when the scan shows the same SSID on 5 or 6GHz at
  ≥ −70 dBm while associated on 2.4GHz.
- [ ] Rejoin: disassociate, then associate to the best scanned BSSID for the
  SSID, on a background thread; on permission error open Wi-Fi Settings.
- [ ] Same action for `Weak` when a stronger BSSID of the same SSID is visible
  (the "stuck on far node" case); name that node in the header.

---

## P2 — seeing past the router

### 2.1 GL.iNet router (`router.rs`)

- [ ] **spike: authenticate and map the RPC.** `challenge` (confirmed working
  on 192.168.1.1) → derive the login hash from `alg`/`salt`/`nonce` per GL.iNet
  4.x docs → `login` → session ID. Then find the calls for WAN status, uptime,
  active uplink and interface byte counters. Write the verified method names
  into `README.md`. If they don't exist, stop here and keep WAN hidden.
- [ ] Credentials: widget-owned generic password in the **login** keychain
  (service `wifi-widget.router`), so no admin prompt. Entry via a
  `Connect Router…` item using an `NSAlert` with a secure text field.
- [ ] Poll every 5 s while the menu is open, 30 s otherwise; 60-sample ring
  buffer; throughput from byte deltas over elapsed time.
- [ ] WAN row: live `↓3.2 ↑0.4 MB/s` + sparkline (`NSBezierPath`, 22 % area
  fill, emphasised endpoint); `DOWN · 4 min` in red when the uplink is down.
- [ ] Feed WAN-down into `NoInternet` so the header reads `No internet – WAN down`.
- [ ] Session expiry and wrong-password handling: back off, show nothing rather
  than stale data.

### 2.2 Speed test (`speed.rs`)

- [ ] Run `/usr/bin/networkQuality -c` (verified present) on click only; parse
  the JSON download/upload throughput and responsiveness.
- [ ] `Testing…` while running (~15 s); cancel on menu close or quit.
- [ ] Store the last result per SSID with its time; show `2 h ago`.
- [ ] Disabled on metered networks with the explanation in the row.

### 2.3 Other routers

- [ ] Opportunistic UPnP IGD: SSDP `M-SEARCH` with a 2 s timeout, then
  `GetCommonLinkProperties` and `GetTotalBytes*`. Verified **not** to answer on
  this network, so treat as best-effort only.
- [ ] Hide WAN and Speed rows on networks with no router data and no Home
  status (the café case in the mockup).

### 2.4 Accessibility

- [ ] `NSAccessibility` labels and roles for every custom view: card facts,
  copy buttons, QR buttons, Refresh, rows. VoiceOver must read the state.
- [ ] Keyboard: custom views must not break arrow-key navigation through the
  menu; verify Return activates rows.

---

## P3 — deferred on purpose

- [ ] `Other Networks ▸` submenu listing scanned SSIDs not in the known list;
  open networks join directly, secured ones open Wi-Fi Settings.
- [ ] Location-denied polish: a single `Allow Location Access…` item that
  deep-links to the Privacy pane.
- [ ] Fix or delete `scripts/wifi-qr` (it relies on the removed `airport` CLI);
  the QR button supersedes it.
- [ ] macOS 27: `NSStatusItem` expanded interface session + `NSGlassEffectView`
  to build `designs/alt-black-panel` with system-managed positioning, focus and
  dismissal. If pursued, build it as a shared crate for all the widgets.

---

## Cross-cutting

- **Performance budget:** ~0 % CPU at idle, < 25 MB resident. Never scan on a
  timer. Profile with Instruments once P1 lands.
- **Secrets:** Wi-Fi and router passwords stay in the keychain and in memory
  for the menu session only; audit logs and `--dump` output for leaks.
- **Settings:** `key=value` at `~/.config/wifi-widget/settings`, same format
  and atomic-write approach as `volume-control-widget`.
- **Verification** (there is no CI): `cargo test` for the pure modules
  (signal, classify, probe parsing, QR payload, router parsing, networkQuality
  parsing); `--dump` against `system_profiler`; `bash -n` on `bundle.sh`;
  and a manual pass through every scenario — unplug the WAN, force 2.4GHz on
  the Flint 3, walk to the far room, join a café portal, join the iPhone
  hotspot, deny Location once, turn Wi-Fi off.
