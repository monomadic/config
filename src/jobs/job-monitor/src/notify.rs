//! Notification Centre banners for things that happen on someone else's
//! machine — the whole point of watching a folder you can't see.
//!
//! `UNUserNotificationCenter` needs a real bundle: asking for the current
//! centre from a bare executable throws, which is why `job-monitor` installs as
//! a generated `.app` rather than a binary in `~/.local/bin`. Run straight from
//! `cargo run` and there is no bundle identifier, so notifications quietly turn
//! themselves off instead of taking the app down.

use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::{NSBundle, NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationRequest,
    UNUserNotificationCenter,
};

static AUTHORIZED: AtomicBool = AtomicBool::new(false);

/// Holds no Objective-C object of its own, so it can be shared across the
/// poller threads: the notification centre is fetched per post. Adding a
/// request is safe from any thread — the completion handler comes back on
/// whichever queue the framework chooses, and we don't pass one.
pub struct Notifier {
    enabled: bool,
}

impl Notifier {
    /// Asks for permission once, at startup. The answer arrives asynchronously;
    /// until it does, posts are dropped rather than queued — a banner about a
    /// job that finished several minutes ago is worse than no banner.
    pub fn new() -> Self {
        if !is_bundled() {
            eprintln!("job-monitor: no bundle identifier, notifications disabled");
            return Self { enabled: false };
        }

        let handler = RcBlock::new(|granted: Bool, error: *mut NSError| {
            AUTHORIZED.store(granted.as_bool(), Ordering::Relaxed);
            if !error.is_null() {
                eprintln!("job-monitor: notification authorization failed");
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .requestAuthorizationWithOptions_completionHandler(
                UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
                &handler,
            );

        Self { enabled: true }
    }

    pub fn post(&self, title: &str, body: &str) {
        if !self.enabled || !AUTHORIZED.load(Ordering::Relaxed) {
            return;
        }

        let center = UNUserNotificationCenter::currentNotificationCenter();
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(title));
        content.setBody(&NSString::from_str(body));

        // A fresh identifier each time: reusing one replaces the banner
        // already on screen, and two jobs finishing back to back should
        // produce two.
        let id = NSString::from_str(&format!("job-monitor-{}", next_id()));
        let request =
            UNNotificationRequest::requestWithIdentifier_content_trigger(&id, &content, None);
        center.addNotificationRequest_withCompletionHandler(&request, None);
    }
}

/// Running from a `.app` — the precondition for the whole framework.
fn is_bundled() -> bool {
    NSBundle::mainBundle().bundleIdentifier().is_some()
}

fn next_id() -> u64 {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
