use objc2::{MainThreadMarker, rc::autoreleasepool};
use objc2_app_kit::*;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::sync::{
    Mutex,
    mpsc::{self, Receiver},
};
use std::time::{Duration, Instant};
type JoinResult = (String, Result<(), String>);
static PENDING: Mutex<Option<Receiver<JoinResult>>> = Mutex::new(None);
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
        at.elapsed() >= if phase == Phase::Failed { LINGER_FAILED } else { LINGER_OK }
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
                .ok_or("Wi-Fi interface unavailable")?;
            let networks = interface
                .scanForNetworksWithName_error(Some(&NSString::from_str(&name)))
                .map_err(|e| e.localizedDescription().to_string())?;
            let network = networks
                .iter()
                .filter(|n| n.ssid().is_some_and(|s| s.to_string() == name))
                .max_by_key(|n| n.rssiValue())
                .ok_or("Network is no longer visible")?;
            let mut facts = Vec::new();
            if let Some(channel) = network.wlanChannel() {
                if let Some(band) = wifi_widget::wifi::band(channel.channelBand()) { facts.push(band.label().to_owned()); }
                facts.push(format!("ch {}", channel.channelNumber()));
            }
            facts.push(format!("{} dBm", network.rssiValue()));
            report(&name, Phase::Working, format!("Joining · {}", facts.join(" · ")));
            let password = password.as_deref().map(NSString::from_str);
            interface
                .associateToNetwork_password_error(&network, password.as_deref())
                .map_err(|e| e.localizedDescription().to_string())?;
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
            Ok::<(), String>(())
        });
        if let Err(error) = &result {
            report(&name, Phase::Failed, error.clone());
        }
        let _ = tx.send((name, result));
    });
}
/// Delivers a finished join. A failure closes the open menu before asking for
/// a password, since a modal alert can't sit on top of menu tracking.
pub fn tick(mtm: MainThreadMarker, menu: Option<&NSMenu>) {
    let result = {
        let mut pending = PENDING.lock().unwrap();
        match pending.as_ref().map(|rx| rx.try_recv()) {
            Some(Ok(result)) => {
                *pending = None;
                Some(result)
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                *pending = None;
                None
            }
            _ => None,
        }
    };
    if let Some((name, Err(error))) = result {
        if let Some(menu) = menu { menu.cancelTracking(); }
        let alert = NSAlert::new(mtm);
        alert.setMessageText(&NSString::from_str(&format!("Could not join {name}")));
        alert.setInformativeText(&NSString::from_str("Enter the network password to retry."));
        // The bundle ships no app icon, so the alert would otherwise show a blank tile.
        if let Some(icon) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("wifi.exclamationmark"),
            Some(&NSString::from_str(&error)),
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
        if alert.runModal() == NSAlertFirstButtonReturn {
            let password = field.stringValue().to_string();
            if !password.is_empty() {
                start(name, Some(password));
            }
        }
    }
}
