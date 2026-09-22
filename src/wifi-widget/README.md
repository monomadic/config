# WiFi Widget

A macOS Wi-Fi menu bar diagnostic, built in Rust against objc2. The intended
interface is in `designs/v3`; `TASKS.md` tracks implementation.

**Current milestone: native v3 card and saved networks.** The menu now has a
graphite connected card with a state-colored glyph, band pill, notched SNR meter,
word verdict, three signal/noise/rate columns and endpoint reachability. MAC and
Node rows copy their values, show a hover indicator and briefly confirm copying.
A private MAC is labelled PRIVATE. Node is omitted when unavailable. Empty signal
and disconnected states never reuse old measurements.

Known Networks shows up to eight saved macOS profiles (excluding the connected
SSID when readable), with compact rows that open Wi-Fi Settings. Names remain
available without Location. Rows say Saved: scanning and availability enrichment
are still pending. QR codes, IP/gateway, WAN and speed controls are not built yet.
The Smart Bar meter is narrower to reduce notch-related crowding.

## Build and open

Requires macOS 13 or newer, Rust with edition 2024 support, and Xcode Command Line
Tools. Build the relocatable app, then launch it through macOS:

```sh
./bundle.sh
open "target/release/WiFi Widget.app"
```

The script builds release code, generates the bundle and ad-hoc signs it. It does
not install or launch anything. You may copy the app to `~/Applications`, open
that copy, then select **Open at Login**. There is no LaunchAgent or Dotter
mapping. Rebuilding/re-signing may require granting Location again.

The app has no Dock icon. Quit from its menu. Its status item uses the same
saved-position approach as `menu-tidy`: first launch starts at the right-hand end,
and later launches preserve your Command-dragged position. This avoids placing a
new unnamed item among icons hidden by a menu-bar spacer. **Refresh** rereads the link and
reruns the endpoint check; it does not scan or change your connection. The Style
submenu saves atomically to `~/.config/wifi-widget/settings`. Login registration
is changed only by its menu action; if registration fails, it opens Login Items
settings and shows the error in the menu.

## Permissions

On first launch the app requests Location access to read the SSID and BSSID.
It does not request location coordinates. Without access it still displays band,
channel, PHY, signal, noise, rate, MAC and endpoint status, with the network name
labelled unavailable and Node hidden. Its Location menu action opens System
Settings after denial. Known-network names remain visible without permission; their availability is not yet scanned.

Always open the `.app` through LaunchServices (`open`, Finder or Login Items).
Running its executable directly cannot obtain the intended Location grant.
A sandbox may prevent CoreWLAN access entirely; that is shown as unavailable,
not as Wi-Fi Off. This widget never joins, disconnects, changes Wi-Fi power,
reads passwords, or runs speed tests in this milestone.

## Live measurements

- CoreWLAN events flag a main-thread read within 250 ms. Polling runs every second
  with the menu open, every five seconds closed, and every second for 30 seconds
  after association changes or wake. The timer uses common run-loop modes.
- Snapshots keep signal fresh below 10 seconds, stale at 10–20 seconds, unknown
  after 20 seconds. A zero RSSI never refreshes an older measurement.
- SNR uses measured noise. Without noise the meter uses separate RSSI tiers and
  the value is labelled dBm. Signal bars and weak-signal chip alarms use a 3 dB
  margin, 10-second degradation hold and 15-second recovery hold.
- A small HTTP request to Apple's captive-portal endpoint runs on a worker every
  30 seconds and two seconds after association changes. Two consecutive failures
  confirm an unreachable endpoint. Reachable describes that endpoint only.
- Probes are single-flight. Association generations discard old worker results,
  including on wake and redacted roaming events. DNS resolution may outlive its
  five-second timeout, but a guard prevents accumulating resolver threads.
- The main thread consumes worker results and renders from the snapshot at most
  four times per second. Network I/O never runs on the menu thread. The chip image
  is rebuilt only when its display changes.

Path flags, home inference, band-fallback evidence and gateway fields have model
types but no live adapters. No hotspot usage or router data is invented.

## Diagnostics and checks

```sh
cargo run -- --dump
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
bash -n bundle.sh
```

`--dump` uses the same snapshot API, taking one interface reading and one probe
when associated. It includes sample ages and may contain local network identifiers,
but no passwords. A direct terminal invocation normally has redacted names.
DNS, connection, write and response phases each have five-second limits. Responses
are capped at 64 KiB; unsupported HTTP framing is unknown, not a claimed outage.

## Verification

The diagnostic agreed with macOS System Information on 5 GHz, channel 149, 80 MHz,
Wi-Fi 6, 1200 Mbps and noise −82 dBm. Both returned zero current RSSI on that sample;
the widget correctly omitted SNR. The endpoint answered successfully.

The generated bundle passes plist validation and strict signature verification.
The app launched through `open` and remained running. Automated UI inspection
could not complete because the computer-use service timed out. Visible layout,
Location approval/denial, login registration, keyboard navigation, VoiceOver,
live menu tracking and Quit still require a hands-on pass. No login item was
enabled during verification. Native card PNGs were rendered and inspected in light/dark appearance and across
connection/error/redaction states. Full v3 parity is still pending QR, scans and
additional facts; actual menu interaction and VoiceOver need a hands-on pass.

For a missing icon, launch with `--ui-diagnostics` to write its visibility,
image size and window/screen bounds once to stderr (no network identifiers):

```sh
open --stderr /tmp/wifi-widget-ui.log "target/release/WiFi Widget.app" --args --ui-diagnostics
```

Quit an existing instance first so LaunchServices delivers the arguments to a
new process. During the placement fix, its reported x-position changed from 477
to 1256 on a 1496-point screen, with the same valid 112-point status-item width.

To render deterministic native card fixtures without querying Wi-Fi or requesting
permissions, run:

```sh
cargo run -- --render-preview /tmp/wifi-widget-v3
```

This writes eleven PNGs, including light appearance, long names, zero signal,
Location-redacted identity, Wi-Fi off, and disconnected. These are native view
renders from fixture data, not screenshots of the user's current network.

Panel alignment refinements: Refresh is inline with Known Networks, saved names
use explicit foreground contrast, the outlined band pill follows the network
name, and the card has a subtle border. The green ONLINE label denotes the same
endpoint result as before; its tooltip explains the check. Preview rendering now
also writes `panel.png`, a composed native layout fixture with four saved rows.

Next visual pass prioritizes alignment, padding (especially Known Networks), font
weights and vertically centered pill text. The two-line signal/noise/Mbps stats
are user-approved and should stay. Planned row enrichment adds saved security
labels and scanned strength meters; Refresh becomes a small circular symbol button.

Nearby Networks shows only named networks detected by a recent scan, including
unsaved networks, sorted by signal strength. The connected network stays in the
main card. Scans start in the background at app launch. Menu openings reuse results for three minutes; Refresh requests a new scan. A disabled spinner replaces Refresh while
scanning, with only one
scan in flight. Each SSID shows its strongest scanned access point's band and
signal; duplicate access points are combined and readings expire after three minutes.
Hover the band for RSSI. The outlined Refresh control and network rows highlight
on hover.

IP and Gateway are copyable IPv4 values from the SystemConfiguration service
matching the Wi-Fi interface. Missing values display an em dash. Reads happen
in the background and refresh every five seconds.
Clicking a nearby network attempts a CoreWLAN association in the background.
If joining fails, a secure password field offers a retry; passwords are not
written to settings or passed on a command line. Enterprise authentication uses
Wi-Fi Settings. Refresh keeps its hover feedback without an outline.

Saved networks detected nearby use pure white names and icons; unsaved networks
use secondary text color.

Nearby rows show the security advertised by the strongest scanned access point
beneath the band (WPA2, WPA3, mixed modes, enterprise, OWE, WEP, or Open).
Unrecognized security remains Unknown rather than being treated as open.

At scan startup, available macOS cached scan results populate an empty list first;
these may be older and their tooltips identify them as cached. The spinner remains
active until the fresh scan replaces them. Saved profiles are not used as evidence
of visibility. The SNR meter shows 0/15/25/40/50 dB boundaries; this scale is hidden
when only RSSI is available.
