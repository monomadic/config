use objc2::{MainThreadMarker, rc::autoreleasepool};
use objc2_app_kit::*;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use std::sync::{
    Mutex,
    mpsc::{self, Receiver},
};
type JoinResult = (String, Result<(), String>);
static PENDING: Mutex<Option<Receiver<JoinResult>>> = Mutex::new(None);
pub fn start(name: String, password: Option<String>) {
    let mut pending = PENDING.lock().unwrap();
    if pending.is_some() {
        return;
    }
    let (tx, rx) = mpsc::channel();
    *pending = Some(rx);
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
            let password = password.as_deref().map(NSString::from_str);
            interface
                .associateToNetwork_password_error(&network, password.as_deref())
                .map_err(|e| e.localizedDescription().to_string())
        });
        let _ = tx.send((name, result));
    });
}
pub fn tick(mtm: MainThreadMarker) {
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
