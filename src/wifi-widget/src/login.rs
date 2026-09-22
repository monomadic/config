//! The main app login service (macOS 13+); never installs a LaunchAgent.
use objc2::{class, msg_send, rc::Retained, runtime::AnyObject};
use objc2_foundation::NSError;
#[link(name = "ServiceManagement", kind = "framework")]
unsafe extern "C" {}
pub fn enabled() -> bool {
    unsafe {
        let service: Retained<AnyObject> = msg_send![class!(SMAppService), mainAppService];
        let status: isize = msg_send![&service, status];
        status == 1
    }
}
pub fn toggle() -> Result<(), String> {
    unsafe {
        let service: Retained<AnyObject> = msg_send![class!(SMAppService), mainAppService];
        let mut error: *mut NSError = std::ptr::null_mut();
        let ok: bool = if enabled() {
            msg_send![&service, unregisterAndReturnError: &mut error]
        } else {
            msg_send![&service, registerAndReturnError: &mut error]
        };
        if ok {
            Ok(())
        } else {
            Err(error
                .as_ref()
                .map(|e| e.localizedDescription().to_string())
                .unwrap_or_else(|| {
                    "Open System Settings → General → Login Items to allow WiFi Widget.".into()
                }))
        }
    }
}
