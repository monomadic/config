use objc2::{MainThreadMarker, rc::autoreleasepool};
use objc2_app_kit::*;
use objc2_core_wlan::CWSecurity;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::sync::{
    Mutex,
    mpsc::{self, Receiver},
};
use std::time::{Duration, Instant};
/// Why a join ended, and whether a password could plausibly fix it.
struct Failure {
    message: String,
    ask_password: bool,
}
fn failed(message: impl Into<String>) -> Failure {
    Failure { message: message.into(), ask_password: false }
}
static PENDING: Mutex<Option<Receiver<()>>> = Mutex::new(None);
static PROGRESS: Mutex<Option<Progress>> = Mutex::new(None);
/// How long the outcome stays on the row before it settles into the card.
const LINGER_OK: Duration = Duration::from_secs(4);
const LINGER_FAILED: Duration = Duration::from_secs(8);
const DHCP_WAIT: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Working,
    Connected,
    Failed,
    PasswordRequired,
}
/// What the row under the pointer shows while a join runs: the network, how
/// it is going, and one line of what the system reported last.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Progress {
    pub name: String,
    pub phase: Phase,
    pub detail: String,
    finished: Option<Instant>,
}
fn report(name: &str, phase: Phase, detail: impl Into<String>) {
    let finished = (phase != Phase::Working).then(Instant::now);
    *PROGRESS.lock().unwrap() = Some(Progress { name: name.to_owned(), phase, detail: detail.into(), finished });
}
/// The current join, if any; an outcome expires after its linger time.
pub fn progress() -> Option<Progress> {
    let mut slot = PROGRESS.lock().unwrap();
    let expired = slot.as_ref().and_then(|p| Some((p.phase, p.finished?))).is_some_and(|(phase, at)| {
        at.elapsed() >= if matches!(phase, Phase::Failed | Phase::PasswordRequired) { LINGER_FAILED } else { LINGER_OK }
    });
    if expired {
        *slot = None;
    }
    slot.clone()
}
fn ipv4(interface: &str) -> Option<String> {
    let out = std::process::Command::new("/usr/sbin/ipconfig").args(["getifaddr", interface]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (out.status.success() && !text.is_empty()).then_some(text)
}
pub fn start(name: String, password: Option<String>) {
    let mut pending = PENDING.lock().unwrap();
    if pending.is_some() {
        return;
    }
    let (tx, rx) = mpsc::channel();
    *pending = Some(rx);
    report(&name, Phase::Working, format!("Looking for {name}…"));
    std::thread::spawn(move || {
        let result = autoreleasepool(|_| unsafe {
            let interface = objc2_core_wlan::CWWiFiClient::new()
                .interface()
                .ok_or_else(|| failed("Wi-Fi interface unavailable"))?;
            let networks = interface
                .scanForNetworksWithName_error(Some(&NSString::from_str(&name)))
                .map_err(|e| failed(e.localizedDescription().to_string()))?;
            let network = networks
                .iter()
                .filter(|n| n.ssid().is_some_and(|s| s.to_string() == name))
                .max_by_key(|n| n.rssiValue())
                .ok_or_else(|| failed("Network is no longer visible"))?;
            let mut facts = Vec::new();
            if let Some(channel) = network.wlanChannel() {
                if let Some(band) = wifi_widget::wifi::band(channel.channelBand()) { facts.push(band.label().to_owned()); }
                facts.push(format!("ch {}", channel.channelNumber()));
            }
            facts.push(format!("{} dBm", network.rssiValue()));
            report(&name, Phase::Working, format!("Joining · {}", facts.join(" · ")));
            let password = password.as_deref().map(NSString::from_str);
            // Only a network with a shared passphrase can be retried with one.
            // Open, OWE and enterprise networks report their error on the row.
            let personal = [
                CWSecurity::WEP,
                CWSecurity::WPAPersonal,
                CWSecurity::WPAPersonalMixed,
                CWSecurity::WPA2Personal,
                CWSecurity::WPA3Personal,
                CWSecurity::WPA3Transition,
            ]
            .into_iter()
            .any(|kind| network.supportsSecurity(kind));
            interface
                .associateToNetwork_password_error(&network, password.as_deref())
                .map_err(|e| Failure { message: e.localizedDescription().to_string(), ask_password: personal })?;
            let ifname = interface.interfaceName().map(|n| n.to_string()).unwrap_or_else(|| "en0".into());
            report(&name, Phase::Working, "Associated · waiting for an IP address…");
            let deadline = Instant::now() + DHCP_WAIT;
            let address = loop {
                if let Some(address) = ipv4(&ifname) { break Some(address); }
                if Instant::now() >= deadline { break None; }
                std::thread::sleep(Duration::from_millis(300));
            };
            let rate = interface.transmitRate();
            match address {
                Some(address) if rate > 0. => report(&name, Phase::Connected, format!("Connected · {address} · {rate:.0} Mbps")),
                Some(address) => report(&name, Phase::Connected, format!("Connected · {address}")),
                None => report(&name, Phase::Connected, "Associated, but no IP address yet (DHCP)"),
            }
            Ok::<(), Failure>(())
        });
        if let Err(failure) = &result {
            let (phase, detail) = if failure.ask_password {
                (
                    Phase::PasswordRequired,
                    format!("Click again to enter password · {}", failure.message),
                )
            } else {
                (Phase::Failed, failure.message.clone())
            };
            report(&name, phase, detail);
        }
        let _ = tx.send(());
    });
}
/// Releases the worker slot after the join finishes.
pub fn tick() {
    let mut pending = PENDING.lock().unwrap();
    if pending.as_ref().is_some_and(|rx| !matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty))) {
        *pending = None;
    }
}

pub fn ask_password(mtm: MainThreadMarker, name: String) {
    let alert = NSAlert::new(mtm);
    alert.setMessageText(&NSString::from_str(&format!("Join {name}")));
    alert.setInformativeText(&NSString::from_str("Enter the network password to retry."));
    // The bundle ships no app icon, so the alert would otherwise show a blank tile.
    if let Some(icon) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str("wifi.exclamationmark"),
        None,
    ) {
        icon.setSize(NSSize { width: 64., height: 64. });
        unsafe { alert.setIcon(Some(&icon)) };
    }
    let field = NSSecureTextField::new(mtm);
    field.setFrame(NSRect {
        origin: NSPoint { x: 0., y: 0. },
        size: NSSize {
            width: 280.,
            height: 24.,
        },
    });
    field.setPlaceholderString(Some(&NSString::from_str("Network password")));
    alert.setAccessoryView(Some(&field));
    alert.addButtonWithTitle(&NSString::from_str("Join"));
    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    // An accessory app is never frontmost on its own: without this the
    // alert opens behind other windows and the typing goes elsewhere.
    NSApplication::sharedApplication(mtm).activate();
    alert.window().setInitialFirstResponder(Some(&field));
    if alert.runModal() == NSAlertFirstButtonReturn {
        let password = field.stringValue().to_string();
        if !password.is_empty() {
            start(name, Some(password));
        }
    }
}
