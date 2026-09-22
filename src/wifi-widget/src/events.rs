//! CoreWLAN calls these on arbitrary threads. Only atomic flags cross into the
//! UI; the main run loop consumes them within 250 ms, including menu tracking.
use objc2::{AnyThread, define_class, msg_send, rc::Retained};
use objc2_core_wlan::{CWEventDelegate, CWEventType, CWWiFiClient};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};
use std::sync::atomic::{AtomicU8, Ordering};
const READ: u8 = 1;
const INVALIDATE: u8 = 2;
static PENDING: AtomicU8 = AtomicU8::new(0);
pub fn changed() {
    PENDING.fetch_or(READ | INVALIDATE, Ordering::Release);
}
pub fn take() -> (bool, bool) {
    let bits = PENDING.swap(0, Ordering::AcqRel);
    (bits & READ != 0, bits & INVALIDATE != 0)
}
define_class!(
    #[unsafe(super(NSObject))]
    pub struct Events;
    unsafe impl NSObjectProtocol for Events {}
    unsafe impl CWEventDelegate for Events {
        #[unsafe(method(powerStateDidChangeForWiFiInterfaceWithName:))]
        unsafe fn power(&self, _name: &NSString) {
            changed();
        }
        #[unsafe(method(ssidDidChangeForWiFiInterfaceWithName:))]
        unsafe fn ssid(&self, _name: &NSString) {
            changed();
        }
        #[unsafe(method(bssidDidChangeForWiFiInterfaceWithName:))]
        unsafe fn bssid(&self, _name: &NSString) {
            changed();
        }
        #[unsafe(method(linkDidChangeForWiFiInterfaceWithName:))]
        unsafe fn link(&self, _name: &NSString) {
            changed();
        }
        #[unsafe(method(linkQualityDidChangeForWiFiInterfaceWithName:rssi:transmitRate:))]
        unsafe fn quality(&self, _name: &NSString, _rssi: isize, _rate: f64) {
            PENDING.fetch_or(READ, Ordering::Release);
        }
        #[unsafe(method(clientConnectionInterrupted))]
        unsafe fn interrupted(&self) {
            changed();
        }
        #[unsafe(method(clientConnectionInvalidated))]
        unsafe fn invalidated(&self) {
            changed();
        }
    }
);
pub struct Monitor {
    client: Retained<CWWiFiClient>,
    _delegate: Retained<Events>,
}
impl Monitor {
    pub fn start() -> Self {
        unsafe {
            let delegate: Retained<Events> = msg_send![Events::alloc(), init];
            let client = CWWiFiClient::sharedWiFiClient();
            client.setDelegate(Some(delegate.as_ref()));
            for event in [
                CWEventType::PowerDidChange,
                CWEventType::SSIDDidChange,
                CWEventType::BSSIDDidChange,
                CWEventType::LinkDidChange,
                CWEventType::LinkQualityDidChange,
            ] {
                if let Err(error) = client.startMonitoringEventWithType_error(event) {
                    eprintln!("Wi-Fi events unavailable: {error}");
                }
            }
            Self {
                client,
                _delegate: delegate,
            }
        }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe {
            let _ = self.client.stopMonitoringAllEventsAndReturnError();
            self.client.setDelegate(None);
        }
    }
}
